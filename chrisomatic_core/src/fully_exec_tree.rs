use std::{collections::HashMap, rc::Rc};

use crate::dependency_tree::DependencyTree;
use crate::exec_tree::exec_tree;
use crate::types::*;
use futures_lite::StreamExt;

/// Runs [exec_tree] to completion, calling `on_progress` once each time a step finishes.
/// A summary of affected API resources is returned.
pub async fn fully_exec_tree(
    client: reqwest::Client,
    tree: DependencyTree<Rc<dyn Step>>,
    on_progress: impl Fn(Counts),
) -> HashMap<Resource, StepEffect> {
    let mut effects = ChrisomaticEffects::with_capacity(tree.count());
    let stream = exec_tree(client, tree);
    futures_lite::pin!(stream);
    while let Some(outcome) = stream.next().await {
        effects.update(outcome);
        on_progress(effects.count());
    }
    effects.0
}

/// An indication of how many resources have been affected and how so.
#[derive(Copy, Clone, Debug, Default)]
pub struct Counts {
    /// Count of resources created.
    pub created: u32,
    /// Count of resources unmodified.
    pub unmodified: u32,
    /// Count of resources modified.
    pub modified: u32,
    /// Count of resources which could not be affected due to unsatisfied prerequisites.
    pub unfulfilled: u32,
    /// Count of resources which could not be modified due to errors.
    pub error: u32,
}

impl<'a> FromIterator<&'a StepEffect> for Counts {
    fn from_iter<T: IntoIterator<Item = &'a StepEffect>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Default::default(), |mut counts: Counts, effect| {
                let num = match effect {
                    StepEffect::Created => &mut counts.created,
                    StepEffect::Unmodified => &mut counts.unmodified,
                    StepEffect::Modified => &mut counts.modified,
                    StepEffect::Unfulfilled(..) => &mut counts.unfulfilled,
                    StepEffect::Error(..) => &mut counts.error,
                };
                *num += 1;
                counts
            })
    }
}

pub struct ChrisomaticEffects(HashMap<Resource, StepEffect>);

impl ChrisomaticEffects {
    /// Create a new [HashMap] with the specified capacity.
    fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    /// Record an effect happaned to a target.
    ///
    /// If called more than once for the same target, the saved value might be
    /// overwritten depending on "importance:" e.g. if a previous step produced
    /// [StepEffect::Created] for some target, but then a later step produces
    /// [StepEffect::Error] for that same target, [StepEffect::Error] will
    /// overwrite the previous value because [StepEffect::Error] is more
    /// "important" than [StepEffect::Created].
    fn update(&mut self, Outcome { target, effect }: Outcome) {
        let prev = self.0.remove(&target);
        self.0.insert(target, more_important_between(prev, effect));
    }

    /// Count value types.
    fn count(&self) -> Counts {
        Counts::from_iter(self.0.values())
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

/// Rank the importance of a [StepEffect]. See [ChrisomaticEffects::update] for
/// an explanation of "importance."
fn importance_of(effect: &StepEffect) -> u8 {
    match effect {
        StepEffect::Created => 3,
        StepEffect::Unmodified => 1,
        StepEffect::Modified => 2,
        StepEffect::Unfulfilled(..) => 4,
        StepEffect::Error(..) => 5,
    }
}
