mod cli;
mod container_engine;
mod default_files;
mod read_inputs;
mod sample;

use std::path::PathBuf;

use clap::Parser;
use cli::canonicalize;
use default_files::default_files;
use read_inputs::read_inputs;

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
    install_eyre_hook()?;
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

    let given = read_inputs(&files).await?;
    let manifest = canonicalize(given)?;
    dbg!(manifest);

    Ok(())
}

fn install_eyre_hook() -> color_eyre::Result<()> {
    #[cfg(debug_assertions)]
    let display_location = true;
    #[cfg(not(debug_assertions))]
    let display_location = false;
    color_eyre::config::HookBuilder::blank()
        .display_location_section(display_location)
        .install()
}
