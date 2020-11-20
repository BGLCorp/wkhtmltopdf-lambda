use anyhow::anyhow;
use lambda_runtime::error::HandlerError;
use rusoto_core::Region;
use rusoto_s3::{PutObjectOutput, PutObjectRequest, S3Client, S3};
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::string::ToString;
use tempfile::NamedTempFile;

#[allow(unused_imports)]
use crate::{debug, error, info, warn};
use crate::{PageType, PdfRequest, PdfResponse, S3Details};

const WKHTMLTOPDF_LAYER_PATH: &'static str = "/opt/bin/wkhtmltopdf";
const WKHTMLTOPDF_BUNDLED_PATH: &'static str = "/bin/wkhtmltopdf";

pub fn convert(ev: PdfRequest, _ctx: lambda_runtime::Context) -> Result<PdfResponse, HandlerError> {
    let response = convert_inner(&ev, &_ctx);
    match response {
        Ok(response) => Ok(response),
        Err(e) => Ok(PdfResponse {
            success: false,
            messages: vec![e.to_string()],
        }),
    }
}

fn convert_inner(ev: &PdfRequest, _ctx: &lambda_runtime::Context) -> anyhow::Result<PdfResponse> {
    let mut args = build_args(&ev)?;
    let mut file = NamedTempFile::new()
        .map_err(|e| anyhow!("Failed to create temp file: {}", e.to_string()))?;
    args.push(file.path().to_string_lossy().to_string());

    let (wkhtmltopdf_path, fontconfig_path) = if Path::new(WKHTMLTOPDF_LAYER_PATH).exists() {
        (WKHTMLTOPDF_LAYER_PATH.to_owned(), "/opt/fonts".to_owned())
    } else if env::var("LAMBDA_TASK_ROOT").is_ok()
        && Path::new(
            (env::var("LAMBDA_TASK_ROOT").unwrap().to_string() + WKHTMLTOPDF_BUNDLED_PATH).as_str(),
        )
        .exists()
    {
        let task_root = env::var("LAMBDA_TASK_ROOT").unwrap().to_string();
        (
            task_root.clone() + WKHTMLTOPDF_BUNDLED_PATH,
            task_root + "/fonts",
        )
    } else {
        (
            "/usr/bin/wkhtmltopdf".to_owned(),
            "/usr/share/fonts".to_owned(),
        )
    };

    let output = Command::new(wkhtmltopdf_path)
        .env("FONTCONFIG_PATH", fontconfig_path)
        .stdin(Stdio::null())
        .args(&args)
        .output()?;

    let mut response = PdfResponse {
        success: output.status.success(),
        ..Default::default()
    };
    if output.status.success() {
        info!("Successfully converted HTML to PDF");
        upload(&mut file, &ev.output)?;
    } else {
        error!("wkhtmltopdf exited with {}", output.status);
        error!(
            "wkhtmltopdf stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        error!(
            "wkhtmltopdf stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if !output.stdout.is_empty() {
            response
                .messages
                .push(String::from_utf8_lossy(&output.stdout).to_string());
        }
        if !output.stderr.is_empty() {
            response
                .messages
                .push(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }

    Ok(response)
}

fn build_args(ev: &PdfRequest) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();
    for page in &ev.pages {
        args.push(page.page_type.to_string());
        if page.page_type == PageType::TOC {
            continue;
        }
        if let Some(ref html_url) = page.html_url {
            args.push(html_url.clone());
        } else if let Some(ref html_base64) = page.html_base64 {
            let html = base64::decode(html_base64)
                .map_err(|e| anyhow!("Failed to decode Base64: {}", e.to_string()))?;
            let mut file = NamedTempFile::new()
                .map_err(|e| anyhow!("Failed to create temp file: {}", e.to_string()))?;
            file.write_all(&html)
                .map_err(|e| anyhow!("Failed to write to temp file: {}", e.to_string()))?;
            args.push(file.path().to_string_lossy().to_string());
        } else {
            return Err(anyhow!("No page source specified"));
        }
        for option in &page.options {
            args.push(option.option.clone());
            if let Some(value) = &option.value {
                args.push(value.clone());
            }
        }
    }

    Ok(args)
}

fn upload(file: &mut NamedTempFile, s3_details: &S3Details) -> anyhow::Result<PutObjectOutput> {
    let region = if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
        let region = Region::Custom {
            name: "us-east-1".to_owned(),
            endpoint: endpoint.to_owned(),
        };
        info!(
            "Picked up non-standard endpoint {:?} from S3_ENDPOINT env var",
            region
        );
        region
    } else if let Some(region) = &s3_details.region {
        Region::from_str(region.as_str())?
    } else {
        Region::ApSoutheast2
    };

    let mut contents = Vec::new();
    let length = file.read_to_end(&mut contents)?;
    if length == 0 {
        return Err(anyhow!("Failed to read PDF output"));
    }
    let put_request = PutObjectRequest {
        bucket: s3_details.bucket.clone(),
        key: s3_details.object_key.clone(),
        content_type: Some("application/pdf".to_owned()),
        body: Some(contents.into()),
        ..Default::default()
    };

    let s3 = S3Client::new(region);
    let mut runtime = tokio::runtime::Runtime::new()?;
    let put_response = runtime.block_on(s3.put_object(put_request))?;
    info!(
        "Uploaded PDF to s3://{}/{}",
        s3_details.bucket, s3_details.object_key
    );

    Ok(put_response)
}
