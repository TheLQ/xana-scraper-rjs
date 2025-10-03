use crate::err::{ScrapeError, ScrapeResult};
use crate::jobs::ScrapeConfig;
use crate::utils::{format_duration, split_once};
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::process::exit;
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{Config, Limits, Message, ServerBuilder};
use xana_commons_rs::pretty_format_error;
use xana_commons_rs::tracing_re::{debug, error, info};

pub async fn start_browser_scraper_server(port: u16, config: ScrapeConfig) -> ScrapeResult<()> {
    let addr = SocketAddr::from((IpAddr::from_str("0.0.0.0").expect("bad ip"), port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ScrapeError::NetIo {
            err,
            backtrace: Backtrace::capture(),
        })?;
    info!("listening on {addr}");

    // while let Ok((stream, _addr)) = listener.accept().await {
    //     tokio::spawn(async move {
    //     });
    // }

    // one-shot scraper based on single config
    let (stream, _addr) = listener.accept().await.map_err(|err| ScrapeError::NetIo {
        err,
        backtrace: Backtrace::capture(),
    })?;
    if let Err(e) = client_connection(stream, config).await {
        error!("🛑🛑🛑 client failed, crashing {}", pretty_format_error(&e));
        exit(1);
    }

    Ok(())
}

async fn client_connection(tcp_stream: TcpStream, config: ScrapeConfig) -> ScrapeResult<()> {
    let mut jobs = config.jobs.into_iter();

    info!("client connected from {}", tcp_stream.peer_addr().unwrap());
    let (_request, mut server) = ServerBuilder::new()
        .config(Config::default().frame_size(usize::MAX))
        .limits(Limits::unlimited())
        .accept(tcp_stream)
        .await?;

    let mut active_job = jobs.next().unwrap();
    info!("starting initial job {:?}", active_job);

    let mut expect_download_content = false;
    while let Some(response_res) = server.next().await {
        let message_raw = response_res?;

        if expect_download_content {
            assert!(message_raw.is_binary());
            let content: BytesMut = message_raw.into_payload().into();
            debug!("received content of size {}", content.len());
            active_job.write_content(&content)?;

            if let Some(next_job) = jobs.next() {
                active_job = next_job;
                info!(
                    "rotating to new job {active_job:?} in {}",
                    format_duration(config.request_throttle)
                );
                expect_download_content = false;

                tokio::time::sleep(config.request_throttle).await;
            } else {
                info!("no more jobs, exiting");
                break;
            }
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
            let op = match ClientOp::parse(message)? {
                ClientOp::Init { current_page } => {
                    info!("client init at {}", current_page);

                    Some(ServerOp::Scrape {
                        url: active_job.url.clone(),
                    })
                }
                ClientOp::Content { headers_raw } => {
                    let (status_raw, headers) = split_once(&headers_raw, '\0')?;
                    let status: u16 = match status_raw.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(ScrapeError::InvalidStatus {
                                raw_str: headers_raw,
                                backtrace: Backtrace::capture(),
                            });
                        }
                    };

                    active_job.write_response_headers(headers)?;
                    active_job.write_status(status)?;

                    expect_download_content = true;
                    None
                }
            };

            if let Some(op) = op {
                info!("requesting {}", op);
                server.send(Message::text(op.encode())).await?;
            } else {
                // info!("no follow up op");
            }
        }
    }
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
    Scrape { url: String },
}

impl ServerOp {
    fn encode(&self) -> String {
        match self {
            Self::Scrape { url } => {
                format!("scrape\0{}", url)
            }
        }
    }
}

impl Display for ServerOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scrape { url } => write!(f, "ScrapeOp for {url}"),
        }
    }
}
