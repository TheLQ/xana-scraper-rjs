use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::ExitCode;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{XanaCommonsLogConfig, log_init_trace, pretty_format_error};
use xana_scraper_rjs::{CommsOp, MapNetIoError, ScrapeJob, ScrapeResult};

fn main() -> ExitCode {
    log_init_trace(XanaCommonsLogConfig::default());

    if let Err(e) = client() {
        error!("🛑🛑🛑 failed main {}", pretty_format_error(&e));
        ExitCode::FAILURE
    } else {
        info!("exiting cleanly");
        ExitCode::SUCCESS
    }
}

fn client() -> ScrapeResult<()> {
    let target: SocketAddr = "127.0.0.1:42000".parse().unwrap();
    let mut stream = TcpStream::connect(target).map_net_error(target)?;
    info!("connected to {target}");

    for url in ["https://blog.xana.sh/".to_string()] {
        let op = CommsOp::Job(ScrapeJob {
            url,
            referer: "https://xana.sh".into(),
            output: "tmpout/xanatest.jpg".into(),
        });

        let line = serde_json::to_string(&op)?;
        info!("sending {line}");
        stream.write_all(line.as_bytes()).map_net_error(target)?;

        stream.write(&[0]).map_net_error(target)?;
        stream.flush().map_net_error(target)?;
        let mut dummy_u64 = [0; 8];
        stream.read_exact(&mut dummy_u64).map_net_error(target)?;

        info!("received ack");
    }

    Ok(())
}
