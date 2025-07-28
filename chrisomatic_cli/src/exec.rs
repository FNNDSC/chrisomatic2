use std::{collections::HashMap, fmt::Display};

use chrisomatic_core::{Counts, StepEffect, fully_exec_tree};
use chrisomatic_spec::Manifest;
use chrisomatic_step::Dependency;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

const USER_AGENT: &'static str = concat!(env!("CARGO_CRATE_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) async fn exec_with_progress(
    manifest: Manifest,
) -> color_eyre::Result<HashMap<Dependency, StepEffect>> {
    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let tree = chrisomatic_core::plan(manifest);
    let pb = ProgressBar::new(tree.count() as u64);
    pb.set_style(progress_style());
    let effects = fully_exec_tree(client, tree, |counts| pb.set_message(short_msg(counts))).await;
    pb.finish_and_clear();
    print_final_message(&effects);
    Ok(effects)
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{msg}]{bar}").unwrap()
}

fn short_msg(counts: Counts) -> String {
    let num_bad = counts.unfulfilled + counts.error;
    format!(
        "{}/{}/{}/{}",
        counts.unmodified.green(),
        counts.created.cyan(),
        counts.modified.yellow(),
        colorize_bad(num_bad)
    )
}

fn print_final_message<T>(effects: &HashMap<T, StepEffect>) {
    let counts = Counts::from_iter(effects.values());
    println!(
        "{} Okay  {} Created  {}  Modified {} Unfulfilled  {} Errors",
        counts.unmodified.green().bold(),
        counts.created.cyan().bold(),
        counts.modified.yellow().bold(),
        colorize_bad(counts.unfulfilled),
        colorize_bad(counts.error)
    )
}

fn colorize_bad(count: u32) -> impl Display {
    if count == 0 {
        count.dimmed().to_string()
    } else {
        count.bright_red().bold().to_string()
    }
}
