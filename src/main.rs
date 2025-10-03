use anyhow::Result;
use clap::Parser;
use logpile::{cli::Args, processor::LogProcessor};

fn main() -> Result<()> {
    let args = Args::parse();

    let mut processor = LogProcessor::new(args)?;
    processor.run()?;

    Ok(())
}
