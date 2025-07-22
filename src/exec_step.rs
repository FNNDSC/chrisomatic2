use std::rc::Rc;

use crate::step::{Check, Entries, StatusCheck, Step};

pub(crate) async fn exec_step(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
) -> Result<Entries, StepError> {
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
        Check::Exists(data) => Ok(data),
        Check::Modified(data) => Ok(data),
        Check::DoesNotExist => {
            if let Some(req) = step.create() {
                let res = client.execute(req.request()).await?.error_for_status()?;
                let data = req.deserialize(res.bytes().await?)?;
                Ok(data)
            } else {
                Err(StepError::Uncreatable(url))
            }
        }
        Check::NeedsModification => {
            if let Some(req) = step.modify() {
                let res = client.execute(req.request()).await?.error_for_status()?;
                let data = req.deserialize(res.bytes().await?)?;
                Ok(data)
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
