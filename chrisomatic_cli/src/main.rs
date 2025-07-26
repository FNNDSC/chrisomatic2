mod cli;
mod container_engine;
mod default_files;
mod sample;

use std::path::PathBuf;

use clap::Parser;
use default_files::default_files;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Print sample chrisomatic.toml file
    #[clap(short, long)]
    sample: bool,
    /// Files to apply. If unspecified, either ./chrisomatic.toml
    /// or ./chrisomatic.d/*.toml will be read.
    files: Vec<PathBuf>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    let args = Cli::parse();
    if args.sample {
        println!("{}", crate::sample::SAMPLE_TOML);
        return Ok(());
    }

    let files = if args.files.is_empty() {
        default_files().await?
    } else {
        args.files
    };

    println!("{files:?}");

    Ok(())
}
