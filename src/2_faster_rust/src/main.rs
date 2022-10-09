use anyhow::{Context, Result};
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

#[derive(Serialize)]
struct IndexEntry<'a> {
    archive: &'a str,
    filename: &'a str,
    size: u64,
}

fn index_tarball(input_path: &str, output_path: &str) -> Result<()> {
    let mut tarball = tar::Archive::new(BufReader::new(File::open(input_path)?));
    let mut output = BufWriter::new(File::create(output_path)?);
    for entry in tarball.entries()? {
        let entry = entry?;
        serde_json::to_writer(&mut output, &IndexEntry {
            archive: input_path,
            filename: entry.path()?.to_str().context("non-utf8 path")?,
            size: entry.size(),
        })?;
        writeln!(output)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    for nm in std::fs::read_dir("input")? {
        // We're going to move away from local filesystem paths in the
        // next iteration of the code, so let it just work with strings.
        let input_path = nm?.path();
        let output_path = format!(
            "output{}{}.jsonl",
            std::path::MAIN_SEPARATOR,
            input_path.file_name().context("missing filename")?.to_str().context("invalid utf-8 path")?
        );
        index_tarball(input_path.to_str().context("invalid utf-8 path")?, &output_path)?;
    }
    Ok(())
}
