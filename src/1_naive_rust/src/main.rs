use anyhow::{Context, Result};
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct Output {
    archive: String,
    filename: String,
    size: u64,
}

fn index_tarball(path: &Path) -> Result<()> {
    let mut tarball = tar::Archive::new(File::open(path)?);

    let mut output_path = PathBuf::from("output");
    output_path.push(path.file_name().context("missing filename")?);
    output_path.set_extension("jsonl");
    let mut output = File::create(&output_path)?;

    for entry in tarball.entries()? {
        let entry = entry?;
        let row = serde_json::to_string(&Output {
            archive: path.to_str().context("non-utf8 path")?.into(),
            filename: entry.path()?.to_str().context("non-utf8 path")?.into(),
            size: entry.size(),
        })?;
        writeln!(output, "{}", row)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    for ln in std::io::stdin().lines() {
        index_tarball(Path::new(ln?.trim()))?;
    }
    Ok(())
}
