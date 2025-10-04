use std::net::{SocketAddr, TcpStream};
use std::process::ExitCode;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{MapNetIoError, XanaCommonsLogConfig, log_init_trace, pretty_format_error};
use xana_scraper_rjs::{CommsOp, ScrapeJob, ScrapeResult, client_send_job_to_server};

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
        client_send_job_to_server(&mut stream, &op)?;
    }

    Ok(())
}
