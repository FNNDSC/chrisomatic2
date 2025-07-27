mod canonicalize;
mod container_engine;
mod default_files;
mod exec;
mod read_inputs;
mod sample;

use std::path::PathBuf;

use canonicalize::canonicalize;
use clap::Parser;
use default_files::default_files;
use exec::exec_with_progress;
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
    let counts = exec_with_progress(manifest).await?;

    if counts.error + counts.unfulfilled > 0 {
        color_eyre::eyre::bail!("There were errors and/or unfulfilled steps.");
    }
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
