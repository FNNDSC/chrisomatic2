use std::rc::Rc;

use chrisomatic_step::{Dependency, Entries, PendingStep, Step};
use futures_concurrency::stream::StreamGroup;
use futures_lite::{Stream, StreamExt};

use crate::{
    dependency_spy::target_of,
    dependency_tree::{DependencyTree, NodeIndex},
    exec_step::{Outcome, StepEffect, exec_step},
    state::DependencyHashMap,
};
use async_stream::stream;

/// Execute steps from `tree` in topological ordering.
///
/// When the returned [Stream] produces an item, it means that a step was
/// finished (or that a [PendingStep] was determined to not run). The stream
/// will produce the same number of items as the count of nodes in the given
/// `tree`. When the item is [Some], it is conveyed that an API resource was
/// checked/created/modified (and when the item is [None], the step was an
/// intermediary dependency to get information for a later step that will
/// check/create/modify a resource).
///
/// How it works: first, all steps without dependencies (as produced by
/// [DependencyTree::start]) are executed. Whenever a step is finished,
/// a [Outcome] is yielded by this stream, and the [Entries] produced
/// by the step will be stored in a local [DependencyMap]. Any satisfied
/// dependents of the finished step (as produced by [DependencyTree::after])
/// are then provided with their dependencies via [PendingStep::build]
/// before being executed by [exec_step].
///
/// Notes:
///
/// - The implementation does not use "tasks", i.e. this [Stream] must be
///   polled/`.await`-ed for it to do work.
/// - If this [Stream] is dropped without being exhausted, all running steps
///   will be cancelled. Doing so might be desirable e.g. to implement a
///   "fail-fast" feature.
/// - If a [Outcome::Error] are produced, it is likely that many
///   [Outcome::Unfulfilled] will follow.
/// - If a [Outcome::Unfulfilled] appears without a preceeding [Outcome::Error],
///   it means there is a bug in [crate::plan].
pub fn exec_tree(
    client: reqwest::Client,
    mut tree: DependencyTree<Rc<dyn PendingStep>>,
) -> impl Stream<Item = Option<Outcome>> {
    stream! {
        let mut cache = DependencyHashMap::with_capacity(tree.count() * 4);
        let mut group = StreamGroup::new();

        // NOTE: using macro instead of closure or function to reduce verbosity
        //       of type and lifetime annotations, also to work around the
        //       restrictions of where `yield` can appear inside the `stream!`
        macro_rules! run_steps {
            ($pending_steps:expr) => {
                for (id, pending_step) in $pending_steps {
                    let pre_check: PreCheck<_> = pending_step.build(&cache).into();
                    match pre_check {
                        PreCheck::Fulfilled => (),
                        PreCheck::Unfulfilled(dependency) => {
                            yield Some(Outcome {
                                target: target_of(pending_step),
                                effect: StepEffect::Unfulfilled(dependency)
                            })
                        }
                        PreCheck::Step(step) => {
                            let fut = exec_step_wrapper(&client, step, id);
                            let stream = futures_lite::stream::once_future(Box::pin(fut));
                            group.insert(stream);
                        }
                    }
                }
            };
        }

        run_steps!(tree.start());

        while let Some((id, outcome, outputs)) = group.next().await {
            cache.insert_all(outputs);
            yield outcome;
            run_steps!(tree.after(id));
        }
    }
}

/// Wraps [exec_step] to wrangle its return types.
async fn exec_step_wrapper(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
    id: NodeIndex,
) -> (NodeIndex, Option<Outcome>, Entries) {
    let (outcome, outputs) = exec_step(client, step).await;
    (id, outcome, outputs)
}

/// Flattened enum for return value of [PendingStep::build].
enum PreCheck<T> {
    /// The step is redundant.
    Fulfilled,
    /// The step cannot run because of an unfulfilled dependency.
    Unfulfilled(Dependency),
    /// The step can run.
    Step(T),
}

impl<T> From<Result<Option<T>, Dependency>> for PreCheck<T> {
    fn from(value: Result<Option<T>, Dependency>) -> Self {
        match value {
            Ok(option) => match option {
                Some(x) => PreCheck::Step(x),
                None => PreCheck::Fulfilled,
            },
            Err(e) => PreCheck::Unfulfilled(e),
        }
    }
}
