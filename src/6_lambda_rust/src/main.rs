use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use aws_sdk_s3 as s3;
use cobalt_aws::lambda::{run_message_handler, Error, LambdaContext};
use futures::prelude::*;
use serde::Serialize;
use std::io::prelude::*;
use std::sync::Arc;

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
async fn main() -> Result<(), Error> {
    run_message_handler(message_handler).await
}

#[derive(Debug, clap::Parser)]
struct Config {}

#[derive(Debug)]
struct Context {
    client: s3::Client,
    input_bucket: String,
    output_bucket: String,
    output_prefix: String,
}

#[async_trait]
impl LambdaContext<Config> for Context {
    /// Initialise a shared context object from which will be
    /// passed to all instances of the message handler.
    async fn from_env(_config: &Config) -> Result<Context> {
        let config = aws_config::load_from_env().await;
        let client = s3::Client::new(&config);
        Ok(Context {
            client,
            input_bucket: "rfkelly-rust-python-lambda-demo".into(),
            output_bucket: "rfkelly-rust-python-lambda-demo".into(),
            output_prefix: "output".into(),
        })
    }
}

async fn message_handler(message: String, context: Arc<Context>) -> Result<()> {
    index_tarball(
        &context.client,
        &context.input_bucket,
        &message,
        &context.output_bucket,
        &context.output_prefix,
    )
    .await?;

    Ok(())
}
