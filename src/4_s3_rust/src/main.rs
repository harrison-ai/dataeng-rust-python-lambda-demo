use anyhow::{Context, Result};
use aws_sdk_s3 as s3;
use futures::prelude::*;
use s3::types::ByteStream;
use serde::Serialize;
use std::io::prelude::*;

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

async fn put_object(
    client: &s3::Client,
    bucket: &str,
    key: &str,
    body: impl Into<ByteStream>,
) -> Result<()> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.into())
        .send()
        .await?;
    Ok(())
}

async fn index_tarball(
    client: &s3::Client,
    input_bucket: &str,
    input_key: &str,
    output_bucket: &str,
    output_prefix: &str,
) -> Result<()> {
    let tarball = async_tar::Archive::new(get_object(client, input_bucket, input_key).await?);

    let output = tarball
        .entries()?
        .map_err(anyhow::Error::from)
        .try_fold(Vec::new(), |mut output, entry| async move {
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

    put_object(
        client,
        output_bucket,
        &format!(
            // FIXME: need basename
            "{output_prefix}/partition={}/{input_key}.jsonl",
            &input_key[..2]
        ),
        output,
    )
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
