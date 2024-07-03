use anyhow::{bail, Result};
use clap::Parser;

use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    source_file: PathBuf,
    #[arg(short = 'I')]
    include_dirs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.source_file.exists() {
        bail!("{} does not exist", cli.source_file.display());
    }
    let source_file = cli.source_file.canonicalize()?;
    Ok(())
}
