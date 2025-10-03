#![feature(path_add_extension)]

mod err;
mod jobs;
mod utils;
mod ws_server;

pub use jobs::{ScrapeConfig, ScrapeJob};
pub use ws_server::start_browser_scraper_server;
