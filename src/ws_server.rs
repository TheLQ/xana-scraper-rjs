use crate::comms_server::WsScrapeJob;
use crate::err::{ScrapeError, ScrapeResult};
use crate::jobs::ScrapeConfig;
use crate::utils::{ANY_ADDR, format_duration, split_once};
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::process::exit;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_websockets::{Config, Limits, Message, ServerBuilder, WebSocketStream};
use xana_commons_rs::tracing_re::{debug, error, info};
use xana_commons_rs::{MapNetIoError, pretty_format_error};

pub async fn start_browser_scraper_server(
    ws_port: u16,
    config: ScrapeConfig,
    job_receiver: UnboundedReceiver<WsScrapeJob>,
    shutdown: Arc<Notify>,
) -> ScrapeResult<()> {
    let ws_addr = SocketAddr::from((ANY_ADDR, ws_port));
    let ws_listener = TcpListener::bind(ws_addr).await.map_net_error(ws_addr)?;
    info!("Websocket listening on {ws_addr}");

    tokio::spawn(async move {
        if let Err(e) = ws_server(config, ws_listener, job_receiver).await {
            error!("comms failed {}", pretty_format_error(&e));
        } else {
            info!("comms terminated");
        }
        shutdown.notify_one();
    });

    Ok(())
}

async fn ws_server(
    config: ScrapeConfig,
    server: TcpListener,
    job_receiver: UnboundedReceiver<WsScrapeJob>,
) -> ScrapeResult<()> {
    // one-shot scraper based on single config
    let (stream, _addr) = server
        .accept()
        .await
        .map_net_error(server.local_addr().unwrap())?;
    if let Err(e) = client_connection(stream, config, job_receiver).await {
        error!("🛑🛑🛑 client failed, crashing {}", pretty_format_error(&e));
        exit(1);
    }

    Ok(())
}

async fn client_connection(
    tcp_stream: TcpStream,
    config: ScrapeConfig,
    mut job_receiver: UnboundedReceiver<WsScrapeJob>,
) -> ScrapeResult<()> {
    info!(
        "ws client connected from {}",
        tcp_stream.peer_addr().unwrap()
    );
    let (_request, mut server) = ServerBuilder::new()
        .config(Config::default().frame_size(usize::MAX))
        .limits(Limits::unlimited())
        .accept(tcp_stream)
        .await?;

    let mut active_job = job_receiver.recv().await.unwrap();
    info!("starting initial job {:?}", active_job.job);

    let mut expect_download_content = false;
    let mut last_status = None;
    while let Some(response_res) = server.next().await {
        let message_raw = response_res?;

        if expect_download_content {
            assert!(message_raw.is_binary());
            let content: BytesMut = message_raw.into_payload().into();
            debug!(
                "received content size {} status {}. Sleeping for {}",
                content.len(),
                last_status.unwrap(),
                format_duration(config.request_throttle)
            );
            active_job.job.write_content(&content)?;
            active_job.on_complete.notify_one();

            tokio::time::sleep(config.request_throttle).await;

            // if let Some(next_job) = jobs.next() {
            active_job = job_receiver.recv().await.unwrap();
            info!("rotating to new job {:?}", active_job.job);
            expect_download_content = false;
            last_status = None;

            send_job(&active_job, &mut server).await?;
            // } else {
            //     info!("no more jobs, exiting");
            //     server
            //         .send(Message::text(
            //             ServerOp::Debug {
            //                 text: "thanks".into(),
            //             }
            //             .encode(),
            //         ))
            //         .await?;
            //     break;
            // }
        } else {
            let message = match message_raw.as_text() {
                Some(e) => e,
                None => {
                    return Err(ScrapeError::ContentNotText {
                        raw: message_raw.into_payload().into(),
                        backtrace: Backtrace::capture(),
                    });
                }
            };
            match ClientOp::parse(message)? {
                ClientOp::Init { current_page } => {
                    info!("client init at {}", current_page);
                    send_job(&active_job, &mut server).await?;
                }
                ClientOp::Content { headers_raw } => {
                    let (status_raw, headers) = split_once(&headers_raw, '\0')?;
                    let Ok(status) = status_raw.parse::<u16>() else {
                        return Err(ScrapeError::InvalidStatus {
                            raw_str: headers_raw,
                            backtrace: Backtrace::capture(),
                        });
                    };

                    active_job.job.write_response_headers(headers)?;
                    active_job.job.write_status(status)?;
                    last_status = Some(status);

                    expect_download_content = true;
                }
            };
        }
    }
    Ok(())
}

async fn send_job(
    active_job: &WsScrapeJob,
    stream: &mut WebSocketStream<TcpStream>,
) -> ScrapeResult<()> {
    let op = ServerOp::Scrape {
        url: active_job.job.url.clone(),
        referer: active_job.job.referer.clone(),
    };
    info!("requesting {}", op);
    stream.send(Message::text(op.encode())).await?;
    Ok(())
}

enum ClientOp {
    Init { current_page: String },
    Content { headers_raw: String },
}

impl ClientOp {
    fn parse(raw: &str) -> ScrapeResult<Self> {
        let (op_str, data_str) = split_once(raw, '\0')?;
        let data = data_str.to_string();

        let op = match op_str {
            "init" => Self::Init { current_page: data },
            "content" => Self::Content { headers_raw: data },
            unknown => panic!("unknown op {}", unknown),
        };
        Ok(op)
    }
}

enum ServerOp {
    Scrape { url: String, referer: String },
    Debug { text: String },
}

impl ServerOp {
    fn encode(&self) -> String {
        match self {
            Self::Scrape { url, referer } => {
                format!("scrape\0{url}\0{referer}")
            }
            Self::Debug { text } => {
                format!("debug\0{text}")
            }
        }
    }
}

impl Display for ServerOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scrape { url, referer } => write!(f, "ScrapeOp for {url} referer {referer}"),
            Self::Debug { text } => write!(f, "DebugOp for {text}"),
        }
    }
}
