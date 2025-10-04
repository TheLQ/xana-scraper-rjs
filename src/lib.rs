#![feature(path_add_extension)]

mod comms_server;
mod err;
mod jobs;
mod utils;
mod ws_server;

pub use comms_server::start_comms_server;
pub use err::{MapNetIoError, ScrapeError, ScrapeResult};
pub use jobs::{ScrapeConfig, ScrapeJob};
pub use ws_server::start_browser_scraper_server;
