use anyhow::{Context, Result};
use aws_sdk_s3 as s3;
use futures::prelude::*;
use serde::Serialize;
use std::io::prelude::*;


#[derive(Serialize)]
struct Output {
    filename: String,
    size: u64,
}

async fn index_tarball(
    client: &s3::Client,
    input_bucket: &str,
    input_key: &str,
    output_bucket: &str,
    output_prefix: &str,
) -> Result<()> {
    let tarball = async_tar::Archive::new(
        client
            .get_object()
            .bucket(input_bucket)
            .key(input_key)
            .send()
            .await?
            .body
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .into_async_read(),
    );

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

    // FIXME: need basename
    let output_key = format!(
        "{output_prefix}/partition={}/{input_key}.jsonl",
        &input_key[..2]
    );
    client
        .put_object()
        .bucket(output_bucket)
        .key(output_key)
        .body(output.into())
        .send()
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
