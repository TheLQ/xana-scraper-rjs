use crate::err::{ScrapeError, ScrapeResult};
use std::backtrace::Backtrace;
use std::net::Ipv4Addr;
use std::time::Duration;

pub const ANY_ADDR: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

pub fn split_once(content: &str, separator: char) -> ScrapeResult<(&str, &str)> {
    content
        .split_once(separator)
        .ok_or_else(|| ScrapeError::SplitFailed {
            content: content.to_string(),
            separator,
            backtrace: Backtrace::capture(),
        })
}

pub fn format_duration(duration: Duration) -> String {
    duration.as_secs().to_string()
}
