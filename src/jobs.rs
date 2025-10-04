use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use xana_commons_rs::{SimpleIoMap, SimpleIoResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScrapeJob {
    pub url: String,
    pub referer: String,
    pub output: PathBuf,
}

impl ScrapeJob {
    pub fn write_status(&self, status: u16) -> SimpleIoResult<()> {
        let output_path = self.output_with_suffix("status");
        fs::write(&output_path, status.to_string()).map_io_err(output_path)
    }

    pub fn write_response_headers(&self, headers: &str) -> SimpleIoResult<()> {
        let output_path = self.output_with_suffix("headers.response");
        fs::write(&output_path, headers).map_io_err(output_path)
    }

    pub fn write_content(&self, content: &[u8]) -> SimpleIoResult<()> {
        fs::write(&self.output, content).map_io_err(&self.output)
    }

    fn output_with_suffix(&self, extension: &str) -> PathBuf {
        let mut res = self.output.clone();
        res.add_extension(extension);
        res
    }
}

//

#[derive(Serialize, Deserialize)]
pub struct ScrapeConfig {
    pub request_throttle: Duration,
}
