use crate::config::CoverConfig;
use crate::error::AppError;
use image::ImageFormat;
use reqwest::blocking::Client;
use std::io::{Cursor, Read};
use std::thread::sleep;
use std::time::Duration;

const MAX_COVER_BYTES: usize = 20 * 1024 * 1024;
const MAX_ATTEMPTS: usize = 3;

pub struct CoverDownloader {
    client: Client,
}

impl CoverDownloader {
    pub fn new(config: &CoverConfig) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds.max(1)))
            .user_agent("shrncnt/0.1")
            .build()?;
        Ok(Self { client })
    }

    pub fn download_webp(&self, url: &str) -> Result<Vec<u8>, AppError> {
        for attempt in 0..MAX_ATTEMPTS {
            let response = match self.client.get(url).send() {
                Ok(response) => response,
                Err(error) if is_retryable_error(&error) && attempt + 1 < MAX_ATTEMPTS => {
                    sleep(retry_delay(attempt));
                    continue;
                }
                Err(error) => return Err(error.into()),
            };

            if !response.status().is_success() {
                let retry = is_retryable_status(response.status());
                if retry && attempt + 1 < MAX_ATTEMPTS {
                    sleep(retry_delay(attempt));
                    continue;
                }
                return Err(response.error_for_status().unwrap_err().into());
            }

            return decode_response(response);
        }

        unreachable!("cover attempts always return or continue to a bounded retry")
    }
}

fn decode_response(mut response: reqwest::blocking::Response) -> Result<Vec<u8>, AppError> {
    if let Some(length) = response.content_length()
        && length > MAX_COVER_BYTES as u64
    {
        return Err(AppError::Config("cover response exceeds 20 MiB".to_owned()));
    }
    let mut source = Vec::new();
    response
        .by_ref()
        .take((MAX_COVER_BYTES + 1) as u64)
        .read_to_end(&mut source)?;
    if source.len() > MAX_COVER_BYTES {
        return Err(AppError::Config("cover response exceeds 20 MiB".to_owned()));
    }
    let image = image::load_from_memory(&source)?;
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::WebP)?;
    Ok(output.into_inner())
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

fn retry_delay(attempt: usize) -> Duration {
    match attempt {
        0 => Duration::from_millis(200),
        _ => Duration::from_millis(500),
    }
}
