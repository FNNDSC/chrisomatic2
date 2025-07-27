use std::fmt::Display;

use chrisomatic_core::{Outcome, StepEffect};
use chrisomatic_spec::Manifest;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

const USER_AGENT: &'static str = concat!(env!("CARGO_CRATE_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) async fn exec_with_progress(manifest: Manifest) -> color_eyre::Result<Count> {
    let tree = chrisomatic_core::plan(manifest);
    let mut counts: Count = Default::default();
    let pb = ProgressBar::new(tree.count() as u64);
    pb.set_style(progress_style());
    pb.set_message(counts.short_msg());

    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let stream = chrisomatic_core::exec_tree(client, tree);
    futures::pin_mut!(stream);
    while let Some(outcome) = stream.next().await {
        pb.inc(1);
        if let Some(outcome) = outcome {
            counts.add(&outcome);
            pb.set_message(counts.short_msg());
            // pb.println(format!("{outcome:?}"));
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
    // pb.finish_with_message(counts.final_report());
    pb.finish_and_clear();
    counts.print_final_report();
    Ok(counts)
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{msg}]{bar}").unwrap()
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Count {
    pub created: u32,
    pub unmodified: u32,
    pub modified: u32,
    pub unfulfilled: u32,
    pub error: u32,
}

impl Count {
    fn add(&mut self, outcome: &Outcome) {
        let field = match outcome.effect {
            StepEffect::Created => &mut self.created,
            StepEffect::Unmodified => &mut self.unmodified,
            StepEffect::Modified => &mut self.modified,
            StepEffect::Unfulfilled(..) => &mut self.unfulfilled,
            StepEffect::Error(..) => &mut self.error,
        };
        *field += 1;
    }

    fn short_msg(&self) -> String {
        let error = self.unfulfilled + self.error;
        format!(
            "{}/{}/{}/{}",
            self.unmodified(),
            self.created(),
            self.modified(),
            if error > 0 {
                error.bright_red().to_string()
            } else {
                error.dimmed().to_string()
            }
        )
    }

    fn print_final_report(&self) {
        println!(
            "{} Unchanged  {} Created  {} Modified  {} Unfulfilled  {} Errors",
            self.unmodified().bold(),
            self.created().bold(),
            self.modified().bold(),
            self.unfulfilled(),
            self.error(),
        )
    }

    fn message(&self) -> String {
        format!("")
    }

    fn unmodified(&self) -> impl Display {
        self.unmodified.green()
    }

    fn created(&self) -> impl Display {
        self.created.cyan()
    }

    fn modified(&self) -> impl Display {
        self.modified.yellow()
    }

    fn unfulfilled(&self) -> impl Display {
        if self.unfulfilled == 0 {
            self.unfulfilled.dimmed().to_string()
        } else {
            self.unfulfilled.bright_red().bold().to_string()
        }
    }

    fn error(&self) -> impl Display {
        if self.error == 0 {
            self.unfulfilled.dimmed().to_string()
        } else {
            self.unfulfilled.bright_red().bold().to_string()
        }
    }
}
