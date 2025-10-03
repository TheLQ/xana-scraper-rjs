use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{Config, Limits, Message, ServerBuilder, WebSocketStream};
use xana_commons_rs::SimpleIoResult;
use xana_commons_rs::tracing_re::info;

pub async fn start_web_socket_server(port: u16) -> SimpleIoResult<()> {
    let addr = SocketAddr::from((IpAddr::from_str("0.0.0.0").expect("bad ip"), port));
    let listener = TcpListener::bind(addr).await.expect("failed to bind");
    info!("listening on {addr}");

    while let Ok((stream, _addr)) = listener.accept().await {
        tokio::spawn(async move { client_connection(stream).await });
    }

    Ok(())
}

async fn client_connection(tcp_stream: TcpStream) {
    info!("client connected from {}", tcp_stream.peer_addr().unwrap());
    let (_request, mut server) = ServerBuilder::new()
        .config(Config::default().frame_size(usize::MAX))
        .limits(Limits::unlimited())
        .accept(tcp_stream)
        .await
        .expect("failed to accept");

    let mut expect_download_content = false;
    while let Some(response_res) = server.next().await {
        let message_raw = response_res.expect("failed to unwrap msg");

        if expect_download_content {
            assert!(message_raw.is_binary());
            let content: BytesMut = message_raw.into_payload().into();
            info!("received content of size {}", content.len());
            expect_download_content = false;
        } else {
            let message = message_raw.as_text().expect("expected text message");
            let op = match ClientOp::parse(message) {
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
                server
                    .send(Message::text(op.encode()))
                    .await
                    .expect("failed to send");
            } else {
                info!("no follow up op");
            }
        }
    }
}

enum ClientOp {
    Init { current_page: String },
    Content { headers_raw: String },
}

impl ClientOp {
    fn parse(raw: &str) -> Self {
        let (op, data_str) = raw.split_once(":").expect("bad op");
        let data = data_str.to_string();

        match op {
            "init" => Self::Init { current_page: data },
            "content" => Self::Content { headers_raw: data },
            unknown => panic!("unknown op {}", unknown),
        }
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
