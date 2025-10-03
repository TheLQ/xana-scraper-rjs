use std::process::ExitCode;
use xana_commons_rs::tracing_re::{error, info};
use xana_commons_rs::{XanaCommonsLogConfig, log_init_trace, pretty_format_error};
use xana_scraper_rjs::ws_server::start_web_socket_server;

#[tokio::main]
async fn main() -> ExitCode {
    log_init_trace(XanaCommonsLogConfig::default());

    if let Err(e) = start_web_socket_server(8080).await {
        error!("🛑🛑🛑 failed main {}", pretty_format_error(&e));
        ExitCode::FAILURE
    } else {
        info!("exiting cleanly");
        ExitCode::SUCCESS
    }
}
