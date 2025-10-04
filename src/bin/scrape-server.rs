use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::Notify;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{XanaCommonsLogConfig, log_init_trace, pretty_format_error};
use xana_scraper_rjs::{
    ScrapeConfig, ScrapeResult, start_browser_scraper_server, start_comms_server,
};

fn main() -> ExitCode {
    log_init_trace(XanaCommonsLogConfig {
        map_huge_crate_names: [("xana_scraper_rjs", "xs")].into_iter().collect(),
        ..Default::default()
    });

    let runtime = Builder::new_multi_thread()
        .thread_name("tokw")
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    if let Err(e) = runtime.block_on(async { start().await }) {
        error!("🛑🛑🛑 failed main {}", pretty_format_error(&e));
        ExitCode::FAILURE
    } else {
        info!("exiting cleanly");
        ExitCode::SUCCESS
    }
}

async fn start() -> ScrapeResult<()> {
    let shutdown = Arc::new(Notify::new());
    let config = ScrapeConfig {
        // jobs: vec![ScrapeJob {
        //     url: "https://xana.sh/Aelitasupercomputer.jpg".into(),
        //     referer: "https://xana.sh/".into(),
        //     output: PathBuf::from("./tmpout/xanatest.jpg"),
        // }],
        request_throttle: Duration::from_secs(5),
    };

    let job_receiver = start_comms_server(42_000, shutdown.clone()).await?;
    start_browser_scraper_server(8080, config, job_receiver, shutdown.clone()).await?;

    shutdown.notified().await;
    Ok(())
}
