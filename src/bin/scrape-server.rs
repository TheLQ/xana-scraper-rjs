use std::process::ExitCode;
use xana_commons_rs::{XanaCommonsLogConfig, log_init_trace};
use xana_scraper_rjs::ws_server::start_web_socket_server;

#[tokio::main]
async fn main() -> ExitCode {
    log_init_trace(XanaCommonsLogConfig::default());

    let server = start_web_socket_server(8080).await;
    ExitCode::SUCCESS
}
