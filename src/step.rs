use either::Either;
use std::rc::Rc;

use crate::state::{Dependency, DependencyMap, Entry};

/// A `PendingStep` represents a [Step] with data dependencies.
pub(crate) trait PendingStep {
    /// Provide the dependencies and create a [Step].
    ///
    /// ## Return
    ///
    /// - [None] if dependencies are not met.
    /// - [Right] if this step's value can be produced trivially from its dependencies.
    /// - [Left] if this step needs to make HTTP requests to produce its value.
    fn build(&self, map: &DependencyMap) -> Option<Either<Rc<dyn Step>, Entries>>;
}

/// A `Step` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **check** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
///
/// ## Behvaior
///
/// 1. [Step::search] is called to search for the resource's prior existence in the API.
///    (It can also do the resource creation/modification right away if that is possible.)
/// 2. [Step::deserialize_search_response] decides what to do next.
/// 3. If the resource needs to be created, call [Step::create]. Or, if the resource
///    needs to be modified, calll [Step::modify].
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
