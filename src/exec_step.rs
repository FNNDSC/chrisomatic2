use std::rc::Rc;

use crate::{
    state::Dependency,
    step::{Check, Entries, StatusCheck, Step},
};

/// Execute a [Step].
///
/// 1. [Step::search] is called to search for the resource's prior existence in the API.
///    (It can also do the resource creation/modification right away if that is possible.)
/// 2. [Step::deserialize] decides what to do next.
/// 3. If the resource needs to be created, call [Step::create]. Or, if the resource
///    needs to be modified, calll [Step::modify].
pub(crate) async fn exec_step(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
) -> (StepEffect, Entries) {
    match exec_step_impl(client, step).await {
        Ok(tup) => tup,
        Err(e) => (StepEffect::Error(e), vec![]),
    }
}

async fn exec_step_impl(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
) -> Result<(StepEffect, Entries), StepError> {
    let req = step.search();
    let method = req.method().clone();
    let res = client.execute(step.search()).await?;
    let url = res.url().clone();
    let check = match step.check_status(res.status()) {
        StatusCheck::Exists => step.deserialize(res.bytes().await?)?,
        StatusCheck::DoesNotExist => Check::DoesNotExist,
        StatusCheck::Error => {
            return Err(StepError::Status {
                status: res.status(),
                method,
                url: res.url().clone(),
            });
        }
    };
    match check {
        Check::Exists(data) => Ok((StepEffect::Unmodified, data)),
        Check::Modified(data) => Ok((StepEffect::Modified, data)),
        Check::DoesNotExist => {
            if let Some(req) = step.create() {
                let res = client.execute(req.request()).await?.error_for_status()?;
                let data = req.deserialize(res.bytes().await?)?;
                Ok((StepEffect::Created, data))
            } else {
                Err(StepError::Uncreatable(url))
            }
        }
        Check::NeedsModification => {
            if let Some(req) = step.modify() {
                let res = client.execute(req.request()).await?.error_for_status()?;
                let data = req.deserialize(res.bytes().await?)?;
                Ok((StepEffect::Modified, data))
            } else {
                Err(StepError::Unmodifiable(url))
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum StepError {
    #[error("Will not try to create resource which should have already been created: {0}")]
    Uncreatable(reqwest::Url),
    #[error("Resource cannot be modified: {0}")]
    Unmodifiable(reqwest::Url),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("HTTP status code {status} from {method} {url}")]
    Status {
        status: reqwest::StatusCode,
        method: reqwest::Method,
        url: reqwest::Url,
    },
    #[error(transparent)]
    Deserialize(#[from] serde_json::Error),
}

/// The effect a step has had on the API state.
pub(crate) enum StepEffect {
    /// A resource was created.
    Created,
    /// A resource was found.
    Unmodified,
    /// A resource was modified.
    Modified,
    /// The step was not performed because of an unfulfilled dependency.
    Unfulfilled(Dependency),
    /// The step produced an error.
    Error(StepError),
}
