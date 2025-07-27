use std::rc::Rc;

use nonempty::NonEmpty;

use crate::dependency_map::{Dependency, DependencyMap, Entry};

/// A `PendingStep` represents a [Step] with data dependencies.
pub trait PendingStep {
    /// Provide the dependencies and create a [Step].
    ///
    /// - [Err] indicates the step has an unfulfilled dependency.
    /// - [None] indicates the step is redundant.
    /// - [Some] indicates the step can run.
    fn build(&self, map: &dyn DependencyMap) -> PendingStepResult;
}

/// Return type of [PendingStep::build].
pub type PendingStepResult = Result<Option<Rc<dyn Step>>, Dependency>;

/// Convenience function to return [Step] from [PendingStep::build].
#[inline(always)]
pub fn ok_step(step: impl Step + 'static) -> PendingStepResult {
    Ok(Some(Rc::new(step)))
}

/// A `Step` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **search** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
///
/// The implementation of step execution _should_:
///
/// 1. Call [Step::search] to produce an HTTP request
/// 2. Send the HTTP request and call [Step::check_status] to decide what to do next.
/// 3. If [StatusCheck::DoesNotExist] is returned by [Step::check_status], call
///    [Step::create] and send the HTTP request to create the API resource.
/// 4. Else if [StatusCheck::Exists] is returned by [Step::check_status], call
///    [Step::deserialize] to decide what to do next.
/// 5. If [Check::DoesNotExist] is returned by [Step::deserialize], call
///    [Step::create] and send the HTTP request to create the API resource.
/// 6. Else if [Check::NeedsModification] is returned by [Step::deserialize],
///    call [Step::modify] and send the HTTP request to modify the API resource.
pub trait Step {
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

    /// Returns keys of what this step provides unconditionally when successful.
    ///
    /// The first item returned is considered to be the "primary" API resource
    /// affected by this step. It will be used for tracking what was created/
    /// modified/unchanged. For example, [crate::UserDetailsFinalizeStep]
    /// sets a user's email. The primary logical API model affected on the
    /// backend here is a user, so [Dependency::UserExists] is the first item
    /// returned. The step [crate::UserDetailsFinalizeStep] also returns
    /// [Dependency::UserEmail] because it is a second piece of output data
    /// in addition to confirming the user's existence.
    fn provides(&self) -> NonEmpty<Dependency>;
}

/// Multiple [Entry].
pub type Entries = Vec<Entry>;

/// An HTTP request and response body deserializer.
pub trait StepRequest {
    /// Create the HTTP request.
    fn request(&self) -> reqwest::Request;

    /// Deserialize the HTTP response body.
    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries>;
}

/// Possible outcomes when checking for the existence of an API resource.
pub enum Check {
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
pub enum StatusCheck {
    /// The resource exists and its body should be checked.
    Exists,
    /// The resource does not exist and needs to be created.
    DoesNotExist,
    /// There was an error.
    Error,
}
