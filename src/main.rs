mod utils;
mod wkhtmltopdf;

use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger};
use std::cmp::PartialEq;
use std::error::Error;
use std::sync::Mutex;

#[allow(unused_imports)]
use utils::*;

static LOGGER: OnceCell<Logger> = OnceCell::new();

#[derive(Deserialize, Clone)]
pub struct PdfRequest {
    pages: Vec<PdfPage>,
    output: S3Details,
}

#[derive(Deserialize, Clone)]
pub struct PdfPage {
    #[serde(rename = "type")]
    page_type: PageType,
    #[serde(rename = "htmlBase64")]
    html_base64: Option<String>,
    #[serde(rename = "htmlUrl")]
    html_url: Option<String>,
    #[serde(default = "Vec::new")]
    options: Vec<PdfOption>,
}

#[derive(Deserialize, strum_macros::Display, PartialEq, Clone)]
pub enum PageType {
    #[strum(serialize = "page")]
    PAGE,
    #[strum(serialize = "toc")]
    TOC,
    #[strum(serialize = "cover")]
    COVER,
}

#[derive(Deserialize, Clone)]
pub struct PdfOption {
    option: String,
    value: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct S3Details {
    region: Option<String>,
    bucket: String,
    #[serde(rename = "objectKey")]
    object_key: String,
}

#[derive(Default, Serialize, Clone)]
pub struct PdfResponse {
    success: bool,
    messages: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let logger = Logger::root(
        Mutex::new(slog_bunyan::with_name(env!("CARGO_CRATE_NAME"), std::io::stdout()).build())
            .fuse(),
        slog::o!(),
    );
    LOGGER
        .set(logger)
        .map_err(|_| HandlerError::from("Failed to initialise logger"))?;

    info!("Initialisation completed");
    lambda!(wkhtmltopdf::convert);

    Ok(())
}
