mod utils;
mod wkhtmltopdf;

use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger};
use std::error::Error;

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

#[derive(Deserialize, strum_macros::Display, Clone)]
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
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = std::sync::Mutex::new(drain).fuse();
    let logger = Logger::root(drain, slog::o!());
    LOGGER
        .set(logger)
        .map_err(|_| HandlerError::from("Failed to initialise logger"))?;

    info!("Initialisation completed");
    lambda!(wkhtmltopdf::convert);

    Ok(())
}
