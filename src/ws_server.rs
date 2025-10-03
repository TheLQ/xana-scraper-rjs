use crate::err::{ScrapeError, ScrapeResult};
use crate::utils::split_once;
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
use xana_commons_rs::tracing_re::{error, info};

pub async fn start_web_socket_server(port: u16) -> ScrapeResult<()> {
    let addr = SocketAddr::from((IpAddr::from_str("0.0.0.0").expect("bad ip"), port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ScrapeError::NetIo {
            err,
            backtrace: Backtrace::capture(),
        })?;
    info!("listening on {addr}");

    while let Ok((stream, _addr)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = client_connection(stream).await {
                error!("🛑🛑🛑 client failed, crashing {}", pretty_format_error(&e));
                exit(1);
            }
        });
    }

    Ok(())
}

async fn client_connection(tcp_stream: TcpStream) -> ScrapeResult<()> {
    info!("client connected from {}", tcp_stream.peer_addr().unwrap());
    let (_request, mut server) = ServerBuilder::new()
        .config(Config::default().frame_size(usize::MAX))
        .limits(Limits::unlimited())
        .accept(tcp_stream)
        .await?;

    let mut expect_download_content = false;
    while let Some(response_res) = server.next().await {
        let message_raw = response_res?;

        if expect_download_content {
            assert!(message_raw.is_binary());
            let content: BytesMut = message_raw.into_payload().into();
            info!("received content of size {}", content.len());
            expect_download_content = false;
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
                        url: "https://xana.sh/Aelitasupercomputer.jpg".into(),
                    })
                }
                ClientOp::Content { headers_raw } => {
                    info!("client headers: {headers_raw}");
                    expect_download_content = true;
                    None
                }
            };

            if let Some(op) = op {
                info!("requesting {}", op);
                server.send(Message::text(op.encode())).await?;
            } else {
                info!("no follow up op");
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
        let (op_str, data_str) = split_once(raw, ':')?;
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
                format!("scrape:{}", url)
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
