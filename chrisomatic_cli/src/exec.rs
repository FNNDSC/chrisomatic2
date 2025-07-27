use std::{collections::HashMap, fmt::Display};

use chrisomatic_core::{Outcome, StepEffect};
use chrisomatic_spec::Manifest;
use chrisomatic_step::Dependency;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

const USER_AGENT: &'static str = concat!(env!("CARGO_CRATE_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub(crate) async fn exec_with_progress(manifest: Manifest) -> color_eyre::Result<Count> {
    let tree = chrisomatic_core::plan(manifest);
    let total = tree.count();
    let mut counts = Count::with_capacity(total);
    let pb = ProgressBar::new(total as u64);
    pb.set_style(progress_style());
    pb.set_message(counts.short_msg());

    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let stream = chrisomatic_core::exec_tree(client, tree);
    futures::pin_mut!(stream);
    while let Some(outcome) = stream.next().await {
        pb.inc(1);
        counts.add(outcome);
        pb.set_message(counts.short_msg());
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    // pb.finish_with_message(counts.final_report());
    pb.finish_and_clear();
    counts.print_final_report();
    Ok(counts)
}

fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{msg}]{bar}").unwrap()
}

pub struct Count(HashMap<Dependency, StepEffect>);

impl Count {
    fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    /// Add the effect for a target. If called again for the same target, keep
    /// the more important value. See [importance_of].
    fn add(&mut self, Outcome { target, effect }: Outcome) {
        let prev = self.0.remove(&target);
        self.0.insert(target, more_important_between(prev, effect));
    }

    fn short_msg(&self) -> String {
        let num_error = self.count_unfulfilled() + self.count_error();
        format!(
            "{}/{}/{}/{}",
            self.unmodified(),
            self.created(),
            self.modified(),
            if num_error > 0 {
                num_error.bright_red().to_string()
            } else {
                num_error.dimmed().to_string()
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

    fn unmodified(&self) -> impl Display {
        self.0
            .values()
            .filter(|e| matches!(e, StepEffect::Modified))
            .count()
            .green()
            .to_string()
    }

    fn created(&self) -> impl Display {
        self.0
            .values()
            .filter(|e| matches!(e, StepEffect::Created))
            .count()
            .cyan()
            .to_string()
    }

    fn modified(&self) -> impl Display {
        self.0
            .values()
            .filter(|e| matches!(e, StepEffect::Modified))
            .count()
            .yellow()
            .to_string()
    }

    fn count_unfulfilled(&self) -> usize {
        self.0
            .values()
            .filter(|e| matches!(e, StepEffect::Unfulfilled(..)))
            .count()
    }

    fn count_error(&self) -> usize {
        self.0
            .values()
            .filter(|e| matches!(e, StepEffect::Error(..)))
            .count()
    }

    fn unfulfilled(&self) -> impl Display {
        colorize_bad(self.count_unfulfilled())
    }

    fn error(&self) -> impl Display {
        colorize_bad(self.count_error())
    }

    pub(crate) fn all_ok(&self) -> bool {
        self.0.values().all(|e| {
            matches!(
                e,
                StepEffect::Created | StepEffect::Unmodified | StepEffect::Modified
            )
        })
    }
}

fn more_important_between(prev: Option<StepEffect>, current: StepEffect) -> StepEffect {
    if let Some(prev) = prev {
        if importance_of(&current) > importance_of(&prev) {
            current
        } else {
            prev
        }
    } else {
        current
    }
}

/// Rank the importance of a [StepEffect].
///
/// E.g. suppose an earlier step produced [StepEffect::Created], then a later
/// step produced [StepEffect::Error] for the same thing. [StepEffect::Error]
/// is more important than [StepEffect::Created], so the [StepEffect::Error]
/// should overwrite the value of [StepEffect::Created] in [Count].
fn importance_of(effect: &StepEffect) -> u8 {
    match effect {
        StepEffect::Created => 3,
        StepEffect::Unmodified => 1,
        StepEffect::Modified => 2,
        StepEffect::Unfulfilled(dependency) => 4,
        StepEffect::Error(step_error) => 5,
    }
}

fn colorize_bad(count: usize) -> impl Display {
    if count == 0 {
        count.dimmed().to_string()
    } else {
        count.bright_red().bold().to_string()
    }
}
