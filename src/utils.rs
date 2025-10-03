use crate::err::{ScrapeError, ScrapeResult};
use std::backtrace::Backtrace;

pub fn split_once(content: &str, separator: char) -> ScrapeResult<(&str, &str)> {
    content
        .split_once(separator)
        .ok_or_else(|| ScrapeError::SplitFailed {
            content: content.to_string(),
            separator,
            backtrace: Backtrace::capture(),
        })
}
