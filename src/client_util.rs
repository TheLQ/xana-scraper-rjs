use crate::CommsOp;
use std::io::{Read, Write};
use std::net::SocketAddr;
use xana_commons_rs::tracing_re::trace;
use xana_commons_rs::{MapNetIoError, SimpleNetResult};

pub fn client_send_job_to_server<S>(stream: &mut S, job: &CommsOp) -> SimpleNetResult<()>
where
    S: Read + Write + GetPeerAddr,
{
    let target = stream.peer_addr();

    let op = serde_json::to_string(&job).unwrap();
    trace!("sending {op}");
    stream.write_all(op.as_bytes()).map_net_error(target)?;

    stream.write(&[0]).map_net_error(target)?;
    stream.flush().map_net_error(target)?;
    let mut dummy_u64 = [0; 8];
    stream.read_exact(&mut dummy_u64).map_net_error(target)?;
    trace!("received ack");

    Ok(())
}

pub trait GetPeerAddr {
    fn peer_addr(&self) -> SocketAddr;
}

impl GetPeerAddr for tokio::net::TcpStream {
    fn peer_addr(&self) -> SocketAddr {
        self.peer_addr().unwrap()
    }
}

impl GetPeerAddr for std::net::TcpStream {
    fn peer_addr(&self) -> SocketAddr {
        self.peer_addr().unwrap()
    }
}
