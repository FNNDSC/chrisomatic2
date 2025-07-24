use std::rc::Rc;

use crate::state::{Dependency, DependencyMap, Entry};

/// A `PendingStep` represents a [Step] with data dependencies.
pub(crate) trait PendingStep {
    /// Provide the dependencies and create a [Step].
    ///
    /// - [Err] indicates the step has an unfulfilled dependency.
    /// - [None] indicates the step is redundant.
    /// - [Some] indicates the step can run.
    fn build(&self, map: &DependencyMap) -> PendingStepResult;
    /// Describe the target this [PendingStep] wants to create/modify.
    ///
    /// [None] indicates the step is for utility only and does not
    /// correspond to the creation or modification of an API resource.
    fn description(&self) -> Option<Dependency> {
        None
    }
}

/// Return type of [PendingStep::build].
pub(crate) type PendingStepResult = Result<Option<Rc<dyn Step>>, Dependency>;

#[inline(always)]
/// Convenience function to return [Step] from [PendingStep::build].
pub(crate) fn ok_step(step: impl Step + 'static) -> PendingStepResult {
    Ok(Some(Rc::new(step)))
}

/// A `Step` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **search** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
///
/// See [crate::exec_step::exec_step].
pub(crate) trait Step {
    /// Create an HTTP request which searches the API for this resource.
    fn search(&self) -> reqwest::Request;

    /// Check the HTTP response status code to the request of [Step::search].
    fn check_status(&self, status: reqwest::StatusCode) -> StatusCheck {
        if status.is_success() {
            StatusCheck::Exists
        } else {
            StatusCheck::Error
        }
    }

    /// Deserialize the HTTP response body to the request of [Step::search] and decide what to do next.
    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Check>;

    /// Create an HTTP request which creates the API resource.
    fn create(&self) -> Option<Box<dyn StepRequest>> {
        None
    }

    /// Create an HTTP request which modifies the API resource.
    fn modify(&self) -> Option<Box<dyn StepRequest>> {
        None
    }
}

/// Multiple [Entry].
pub(crate) type Entries = Vec<Entry>;

/// An HTTP request and response body deserializer.
pub(crate) trait StepRequest {
    /// Create the HTTP request.
    fn request(&self) -> reqwest::Request;

    /// Deserialize the HTTP response body.
    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries>;
}

/// Possible outcomes when checking for the existence of an API resource.
pub(crate) enum Check {
    /// The resource exists and is correct.
    Exists(Entries),
    /// The resource exists and was modified.
    Modified(Entries),
    /// The resource does not exist and needs to be created.
    DoesNotExist,
    /// The resource exists but needs modification.
    NeedsModification,
}

/// Status conveyed by HTTP response status code.
pub(crate) enum StatusCheck {
    /// The resource exists and its body should be checked.
    Exists,
    /// The resource does not exist and needs to be created.
    DoesNotExist,
    /// There was an error.
    Error,
}
