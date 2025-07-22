use std::rc::Rc;

use either::Either;
use futures_concurrency::stream::StreamGroup;
use futures_lite::StreamExt;

use crate::{
    dependency_tree::{DependencyTree, NodeIndex},
    exec_step::StepError,
    state::DependencyMap,
    step::{Entries, PendingStep, Step},
};

pub(crate) async fn exec_tree(
    client: reqwest::Client,
    mut tree: DependencyTree<Rc<dyn PendingStep>>,
) -> Result<(), StepError> {
    let mut cache: DependencyMap = DependencyMap::with_capacity(tree.count() * 4);
    let mut group = StreamGroup::new();

    // NOTE: using macro instead of closure or function to reduce verbosity
    //       of type and lifetime annotations
    macro_rules! run_steps {
        ($pending_steps:expr) => {
            for (id, pending_step) in $pending_steps {
                match pending_step.build(&cache).unwrap().clone() {
                    Either::Left(step) => {
                        let fut = exec_step(&client, step, id);
                        let stream = futures_lite::stream::once_future(Box::pin(fut));
                        group.insert(stream);
                    }
                    Either::Right(m) => cache.insert_all(m),
                }
            }
        };
    }

    run_steps!(tree.start());

    // NOTE: `?` returns without awaiting the remaining futures in steam,
    //       which will effectively be cancelled (and the HTTP requests
    //       will be closed abruptly).
    while let Some((id, outputs)) = group.try_next().await? {
        cache.insert_all(outputs);
        run_steps!(tree.after(id));
    }
    Ok(())
}

async fn exec_step(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
    id: NodeIndex,
) -> Result<(NodeIndex, Entries), StepError> {
    crate::exec_step::exec_step(client, step)
        .await
        .map(|m| (id, m))
}
