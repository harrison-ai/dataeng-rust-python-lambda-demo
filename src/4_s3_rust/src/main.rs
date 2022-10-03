use anyhow::{Context, Result};
use aws_sdk_s3 as s3;
use futures::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct Output {
    filename: String,
    size: u64,
}

async fn get_object(client: &s3::Client, bucket: &str, key: &str) -> Result<impl AsyncBufRead> {
    Ok(client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?
        .body
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        .into_async_read())
}

async fn index_tarball(
    client: &s3::Client,
    input_bucket: &str,
    input_key: &str,
    output_bucket: &str,
    output_prefix: &str,
) -> Result<()> {
    let tarball = async_tar::Archive::new(get_object(client, input_bucket, input_key).await?);

    let mut output_path = PathBuf::from("output");
    output_path.push(
        Path::new(input_key)
            .file_name()
            .context("missing filename")?,
    );
    output_path.set_extension("jsonl");
    let output = &mut BufWriter::new(File::create(&output_path)?);

    tarball
        .entries()?
        .map_err(anyhow::Error::from)
        .try_fold(output, |mut output, entry| async move {
            serde_json::to_writer(
                &mut output,
                &Output {
                    filename: entry.path()?.to_str().context("non-utf8 path")?.into(),
                    size: entry.header().size()?,
                },
            )?;
            writeln!(output, "")?;
            Ok(output)
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = aws_config::load_from_env().await;
    let client = s3::Client::new(&config);
    for ln in std::io::stdin().lines() {
        index_tarball(
            &client,
            "rfkelly-rust-python-lambda-demo",
            ln?.trim(),
            "rfkelly-rust-python-lambda-demo",
            "output",
        )
        .await?;
    }
    Ok(())
}
