use std::rc::Rc;

use reqwest::Request;

use crate::types::*;

/// Execute the request of a [Step].
pub(crate) async fn exec_step(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
    request: Request,
) -> (Outcome, Entries) {
    let target = step.affects();
    let effect = step.effect();
    match exec_step_impl(client, step, request).await {
        Ok(outputs) => {
            let outcome = Outcome { target, effect };
            (outcome, outputs)
        }
        Err(e) => {
            let effect = StepEffect::Error(e);
            let outcome = Outcome { target, effect };
            (outcome, vec![])
        }
    }
}

async fn exec_step_impl(
    client: &reqwest::Client,
    step: Rc<dyn Step>,
    request: Request,
) -> Result<Entries, StepError> {
    let method = request.method().clone();
    let res = client.execute(request).await?;
    let status = res.status();
    if !step.check_status(status) {
        let url = res.url().clone();
        return Err(StepError::Status {
            status,
            method,
            url,
        });
    }
    let body = res.bytes().await?;
    let outputs = step.deserialize(body)?;
    Ok(outputs)
}
