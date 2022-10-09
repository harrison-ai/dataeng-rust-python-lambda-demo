use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use aws_sdk_s3 as s3;
use cobalt_aws::lambda::{run_message_handler, Error, LambdaContext};
use futures::prelude::*;
use serde::Serialize;
use std::io::prelude::*;
use std::sync::Arc;

#[derive(Serialize)]
struct IndexEntry<'a> {
    archive: &'a str,
    filename: &'a str,
    size: u64,
}

async fn index_tarball(
    client: &s3::Client,
    bucket: &str,
    input_key: &str,
    output_key: &str,
) -> Result<()> {
    let tarball = async_tar::Archive::new(
        client
            .get_object()
            .bucket(bucket)
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
            &IndexEntry {
                archive: input_key,
                filename: entry.path()?.to_str().context("non-utf8 path")?,
                size: entry.header().size()?,
            },
        )?;
        writeln!(output)?;
    }

    client
        .put_object()
        .bucket(bucket)
        .key(output_key)
        .body(output.into())
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run_message_handler(message_handler).await
}

#[derive(Debug, clap::Parser)]
struct Config {}

#[derive(Debug)]
struct Context {
    client: s3::Client,
    bucket: String,
}

#[async_trait]
impl LambdaContext<Config> for Context {
    async fn from_env(_config: &Config) -> Result<Context> {
        let config = aws_config::load_from_env().await;
        let client = s3::Client::new(&config);
        Ok(Context {
            client,
            bucket: "rfkelly-rust-python-lambda-demo".into(),
        })
    }
}

async fn message_handler(message: String, context: Arc<Context>) -> Result<()> {
    let input_key = &message;
    let archive_name = input_key
        .rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(input_key);
    let output_key = format!("output/rs/{archive_name}.jsonl",);
    index_tarball(
        &context.client,
        &context.bucket,
        input_key,
        &output_key,
    )
    .await?;

    Ok(())
}
