use std::rc::Rc;

use futures_concurrency::future::FutureGroup;
use futures_lite::{Stream, StreamExt};

use crate::{
    dependency_tree::{DependencyTree, NodeIndex},
    exec_step::exec_step,
    state::DependencyHashMap,
    types::*,
};
use async_stream::stream;

/// Execute steps from `tree` in topological ordering.
///
/// Notes:
///
/// - The implementation does not use "tasks", i.e. this [Stream] must be
///   polled/`.await`-ed for it to do work.
/// - If this [Stream] is dropped without being exhausted, all running steps
///   will be cancelled. Doing so might be desirable e.g. to implement a
///   "fail-fast" feature.
/// - If a [Outcome] containing [StepEffect::Error] are produced, it is likely
///   that many [StepEffect::Unfulfilled] will follow.
/// - If a [Outcome] containing [StepEffect::Unfulfilled] appears without a
///   preceeding [Outcome::Error], it means there is a bug in [crate::plan].
pub fn exec_tree(
    client: reqwest::Client,
    mut tree: DependencyTree<Rc<dyn Step>>,
) -> impl Stream<Item = Outcome> {
    stream! {
        let mut cache = DependencyHashMap::with_capacity(tree.count() * 4);
        let mut group = FutureGroup::new();

        // NOTE: using macro instead of closure or function to reduce verbosity
        //       of type and lifetime annotations, also to work around the
        //       restrictions of where `yield` can appear inside the `stream!`
        macro_rules! run_steps {
            ($steps:expr) => {
                for (id, step) in $steps {
                    let request = step.request(&cache);
                    let fut = exec_step_wrapper(&client, step, request, id);
                    group.insert(Box::pin(fut));
                }
            };
        }

        run_steps!(tree.start());
        while let Some((id, outcome, outputs)) = group.next().await {
            cache.insert_all(outputs);
            yield outcome;
            run_steps!(tree.after(id));
        }
        debug_assert_eq!(tree.count(), 0);
    }
}

/// Wraps [exec_step] to wrangle its parameter and return types.
async fn exec_step_wrapper(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
    request: Result<Option<reqwest::Request>, Dependency>,
    id: NodeIndex,
) -> (NodeIndex, Outcome, Entries) {
    match request {
        Ok(option) => match option {
            Some(request) => {
                let (outcome, outputs) = exec_step(client, step, request).await;
                (id, outcome, outputs)
            }
            None => {
                let outcome = Outcome {
                    target: step.affects(),
                    effect: StepEffect::Unmodified,
                };
                (id, outcome, vec![])
            }
        },
        Err(dependency) => {
            let outcome = Outcome {
                target: step.affects(),
                effect: StepEffect::Unfulfilled(dependency),
            };
            (id, outcome, vec![])
        }
    }
}
