use crate::err::{ScrapeError, ScrapeResult};
use std::backtrace::Backtrace;
use std::time::Duration;

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
