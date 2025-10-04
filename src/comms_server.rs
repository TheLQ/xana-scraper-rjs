use crate::ScrapeJob;
use crate::err::{MapNetIoError, ScrapeResult};
use crate::utils::ANY_ADDR;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use xana_commons_rs::pretty_format_error;
use xana_commons_rs::tracing_re::{debug, error, info, warn};

pub async fn start_comms_server(
    comms_port: u16,
    shutdown: Arc<Notify>,
) -> ScrapeResult<UnboundedReceiver<WsScrapeJob>> {
    let comms_addr = SocketAddr::from((ANY_ADDR, comms_port));
    let comms_listener = TcpListener::bind(comms_addr)
        .await
        .map_net_error(comms_addr)?;
    info!("Comms listening on {comms_addr}");

    let (comms_sender, comms_receiver) = tokio::sync::mpsc::unbounded_channel::<WsScrapeJob>();

    tokio::spawn(async move {
        loop {
            if let Err(e) = comms_server(&comms_listener, &comms_sender).await {
                error!("comms failed {}", pretty_format_error(&e));
            } else {
                info!("comms terminated");
            }
        }
        // shutdown.notify_one();
    });

    Ok(comms_receiver)
}

async fn comms_server(
    server: &TcpListener,
    comms_sender: &UnboundedSender<WsScrapeJob>,
) -> ScrapeResult<()> {
    info!("waiting for comms client");
    let (client_stream, _addr) = server
        .accept()
        .await
        .map_net_error(server.local_addr().unwrap())?;
    comms_client(client_stream, comms_sender).await?;
    Ok(())
}

async fn comms_client(
    client_raw: TcpStream,
    comms_sender: &UnboundedSender<WsScrapeJob>,
) -> ScrapeResult<()> {
    let client_addr = client_raw.peer_addr().unwrap();
    info!("comms client connected from {client_addr}");
    let mut client = BufReader::new(client_raw);

    loop {
        let mut message = Vec::new();
        client
            .read_until(0x0, &mut message)
            .await
            .map_net_error(client_addr)?;
        if message.is_empty() {
            warn!("comms client disconnected");
            break;
        }
        // remove null
        message.pop();
        debug!("comms received {}", str::from_utf8(&message).unwrap());

        let op: CommsOp = serde_json::from_slice(&message)?;
        match op {
            CommsOp::Job(job) => {
                let on_complete = Arc::new(Notify::new());
                comms_sender
                    .send(WsScrapeJob {
                        job,
                        on_complete: on_complete.clone(),
                    })
                    .unwrap();
                on_complete.notified().await;

                info!("ack job complete");
                client
                    .write_u64(u64::MAX)
                    .await
                    .map_net_error(client_addr)?;
                client.flush().await.map_net_error(client_addr)?;
            }
            CommsOp::Terminate => {
                info!("received comms terminate");
                break;
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub enum CommsOp {
    Job(ScrapeJob),
    Terminate,
}

pub struct WsScrapeJob {
    pub job: ScrapeJob,
    pub on_complete: Arc<Notify>,
}
