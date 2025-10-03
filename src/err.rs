use bytes::Bytes;
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use xana_commons_rs::MyBacktrace;

pub type ScrapeResult<T> = Result<T, ScrapeError>;

pub enum ScrapeError {
    SplitFailed {
        content: String,
        separator: char,
        backtrace: Backtrace,
    },
    TokioWebsocket {
        err: tokio_websockets::Error,
        backtrace: Backtrace,
    },
    ContentNotText {
        raw: Bytes,
        backtrace: Backtrace,
    },
    NetIo {
        err: std::io::Error,
        backtrace: Backtrace,
    },
}

impl MyBacktrace for ScrapeError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            ScrapeError::SplitFailed { backtrace, .. } => backtrace,
            ScrapeError::TokioWebsocket { backtrace, .. } => backtrace,
            ScrapeError::ContentNotText { backtrace, .. } => backtrace,
            ScrapeError::NetIo { backtrace, .. } => backtrace,
        }
    }
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::SplitFailed {
                content, separator, ..
            } => {
                write!(f, "failed to split {} with {}", content, separator)
            }
            ScrapeError::TokioWebsocket { err, .. } => {
                write!(f, "tokio websocket error: {}", err)
            }
            ScrapeError::ContentNotText { raw, .. } => {
                write!(
                    f,
                    "content is not text: len {} starts_with {} ...",
                    raw.len(),
                    String::from_utf8_lossy(&raw[0..10])
                )
            }
            ScrapeError::NetIo { err, .. } => {
                write!(f, "net io: {}", err)
            }
        }
    }
}

impl From<tokio_websockets::Error> for ScrapeError {
    fn from(err: tokio_websockets::Error) -> Self {
        ScrapeError::TokioWebsocket {
            err,
            backtrace: Backtrace::capture(),
        }
    }
}
