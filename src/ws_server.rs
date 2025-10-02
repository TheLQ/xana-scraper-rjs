use futures_util::{SinkExt, StreamExt};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{ServerBuilder, WebSocketStream};
use xana_commons_rs::SimpleIoResult;
use xana_commons_rs::tracing_re::info;

pub async fn start_web_socket_server(port: u16) -> SimpleIoResult<()> {
    let addr = SocketAddr::from((IpAddr::from_str("0.0.0.0").expect("bad ip"), port));
    let listener = TcpListener::bind(addr).await.expect("failed to bind");
    info!("listening on {addr}");

    while let Ok((stream, _)) = listener.accept().await {}

    Ok(())
}

async fn client_connection(tcp_stream: TcpStream) {
    let (_request, mut server) = ServerBuilder::new()
        .accept(tcp_stream)
        .await
        .expect("failed to accept");

    // Just an echo server, really
    while let Some(msg_raw) = server.next().await {
        let msg = msg_raw.expect("failed to unwrap msg");
        if msg.is_text() || msg.is_binary() {
            server.send(msg).await.expect("failed to send");
        }
    }
}
