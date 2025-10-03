use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{XanaCommonsLogConfig, log_init_trace, pretty_format_error};
use xana_scraper_rjs::{ScrapeConfig, ScrapeJob, start_browser_scraper_server};

#[tokio::main]
async fn main() -> ExitCode {
    log_init_trace(XanaCommonsLogConfig::default());

    let config = ScrapeConfig {
        jobs: vec![ScrapeJob {
            url: "https://xana.sh/Aelitasupercomputer.jpg".into(),
            output: PathBuf::from("./tmpout/xanatest.jpg"),
        }],
        request_throttle: Duration::from_secs(1),
    };

    if let Err(e) = start_browser_scraper_server(8080, config).await {
        error!("🛑🛑🛑 failed main {}", pretty_format_error(&e));
        ExitCode::FAILURE
    } else {
        info!("exiting cleanly");
        ExitCode::SUCCESS
    }
}
