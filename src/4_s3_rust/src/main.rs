use anyhow::{Context, Result};
use aws_sdk_s3 as s3;
use futures::prelude::*;
use serde::Serialize;
use std::io::prelude::*;


#[derive(Serialize)]
struct Output<'a> {
    archive: &'a str,
    filename: &'a str,
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

    let mut output = Vec::new();
    let mut entries = tarball.entries()?;
    while let Some(entry) = entries.try_next().await? {
        serde_json::to_writer(
            &mut output,
            &Output {
                archive: input_key,
                filename: entry.path()?.to_str().context("non-utf8 path")?,
                size: entry.header().size()?,
            },
        )?;
        writeln!(output)?;
    }

    let archive_name = input_key
        .rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(input_key);
    client
        .put_object()
        .bucket(output_bucket)
        .key(format!("{output_prefix}/{archive_name}.jsonl",))
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
            "output/rs",
        )
        .await?;
    }
    Ok(())
}
