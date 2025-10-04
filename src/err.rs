use bytes::Bytes;
use std::backtrace::Backtrace;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use xana_commons_rs::{MyBacktrace, SimpleIoError};

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
    InvalidStatus {
        raw_str: String,
        backtrace: Backtrace,
    },
    SerdeJson {
        err: serde_json::Error,
        backtrace: Backtrace,
    },
    NetIo {
        err: std::io::Error,
        addr: SocketAddr,
        backtrace: Backtrace,
    },
    FileIo {
        err: SimpleIoError,
    },
}

impl MyBacktrace for ScrapeError {
    fn my_backtrace(&self) -> &Backtrace {
        match self {
            ScrapeError::SplitFailed { backtrace, .. } => backtrace,
            ScrapeError::TokioWebsocket { backtrace, .. } => backtrace,
            ScrapeError::ContentNotText { backtrace, .. } => backtrace,
            ScrapeError::InvalidStatus { backtrace, .. } => backtrace,
            ScrapeError::SerdeJson { backtrace, .. } => backtrace,
            ScrapeError::NetIo { backtrace, .. } => backtrace,
            ScrapeError::FileIo { err } => err.my_backtrace(),
        }
    }
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::SplitFailed {
                content, separator, ..
            } => {
                write!(f, "failed to split {content} with {separator}")
            }
            ScrapeError::TokioWebsocket { err, .. } => {
                write!(f, "tokio websocket error: {err}")
            }
            ScrapeError::ContentNotText { raw, .. } => {
                let sub = &raw[0..raw.len().min(10)];
                write!(
                    f,
                    "content is not text: len {} starts_with {} ... {}",
                    raw.len(),
                    String::from_utf8_lossy(sub),
                    xana_commons_rs::to_hex_any(sub)
                )
            }
            ScrapeError::InvalidStatus { raw_str, .. } => {
                write!(f, "can't get status from {raw_str}")
            }
            ScrapeError::SerdeJson { err, .. } => {
                write!(f, "serde json error: {err}")
            }
            ScrapeError::NetIo { err, addr, .. } => {
                write!(f, "net io at {addr}: {err}")
            }
            ScrapeError::FileIo { err } => Display::fmt(err, f),
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

impl From<serde_json::Error> for ScrapeError {
    fn from(err: serde_json::Error) -> Self {
        ScrapeError::SerdeJson {
            err,
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<SimpleIoError> for ScrapeError {
    fn from(err: SimpleIoError) -> Self {
        ScrapeError::FileIo { err }
    }
}

pub trait MapNetIoError<T> {
    fn map_net_error(self, addr: SocketAddr) -> Result<T, ScrapeError>;
}

impl<T> MapNetIoError<T> for Result<T, std::io::Error> {
    fn map_net_error(self, addr: SocketAddr) -> Result<T, ScrapeError> {
        self.map_err(|err| ScrapeError::NetIo {
            err,
            addr,
            backtrace: Backtrace::capture(),
        })
    }
}
