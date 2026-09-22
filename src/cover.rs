use crate::config::CoverConfig;
use crate::error::AppError;
use image::ImageFormat;
use reqwest::blocking::Client;
use std::io::Cursor;
use std::time::Duration;

const MAX_COVER_BYTES: usize = 20 * 1024 * 1024;

pub fn download_webp(url: &str, config: &CoverConfig) -> Result<Vec<u8>, AppError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1)))
        .user_agent("shirone-content/0.1")
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    if let Some(length) = response.content_length()
        && length as usize > MAX_COVER_BYTES
    {
        return Err(AppError::Config("cover response exceeds 20 MiB".to_owned()));
    }
    let mut source = Vec::new();
    response
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

use std::io::Read;
