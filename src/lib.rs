#![feature(path_add_extension)]

mod client_util;
mod comms_server;
mod err;
mod jobs;
mod utils;
mod ws_server;

pub use client_util::client_send_job_to_server;
pub use comms_server::{CommsOp, start_comms_server};
pub use err::{ScrapeError, ScrapeResult};
pub use jobs::{ScrapeConfig, ScrapeJob};
pub use ws_server::start_browser_scraper_server;
