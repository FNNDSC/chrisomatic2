use chris_oag::models;

use crate::dependency::Dependency;
use crate::deps::*;
use crate::spec::*;
use crate::types::*;

type CreateRequest = Result<http::Request<Vec<u8>>, CreateRequestError>;

/// An `Action` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **check** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
pub(crate) trait Action {
    type Dep: Dependency;
    type Out: Dependency;

    /// Specify the dependencies for checking whether this resource exists.
    fn dependencies(&self) -> Self::Dep;

    /// Create an HTTP request which queries the API for the resource's existence.
    fn query(&self, dependencies: &<Self::Dep as Dependency>::Value) -> CreateRequest;

    /// Check whether the response to the request created by [Action::query]
    /// indicates the resource already exists.
    fn check(&self, body: Vec<u8>) -> serde_json::Result<Check<Self::Out>>;
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateRequestError {
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Request(#[from] http::Error),
}

pub(crate) enum Check<D: Dependency> {
    Exists(D, D::Value),
    DoesNotExist,
    NeedsModification,
}

pub(crate) struct UserCreate<'a> {
    username: &'a Username,
    details: &'a UserDetails,
}

pub(crate) struct UserfileShareWithUser<'a> {
    path: &'a str,
    owner: &'a Username,
    user: &'a Username,
}

pub(crate) struct FeedShareWithUser<'a> {
    feed: &'a FeedSpec,
    user: &'a Username,
}

impl<'a> Action for FeedShareWithUser<'a> {
    type Dep = (AuthTokenFor<'a>, FeedFor<'a>);
    type Out = FeedUserPermissionFor<'a>;

    fn query(&self, (token, feed): &<Self::Dep as Dependency>::Value) -> CreateRequest {
        let uri = format!(
            "{}/userpermissions/search/?username={}&limit=1",
            feed.url, self.user
        );
        let req = http::Request::builder()
            .method("GET")
            .uri(uri)
            .header("Accept", "application/json")
            .header("Authorization", token)
            .body(vec![])?;
        Ok(req)
    }

    fn dependencies(&self) -> Self::Dep {
        (AuthTokenFor(self.user), FeedFor(self.feed))
    }

    fn check(&self, body: Vec<u8>) -> serde_json::Result<Check<Self::Out>> {
        let data: models::PaginatedFeedUserPermissionList = serde_json::from_slice(&body)?;
        let ret = if let Some(feed) = data.results.into_iter().next() {
            Check::Exists(FeedUserPermissionFor(self.feed, self.user), feed)
        } else {
            Check::DoesNotExist
        };
        Ok(ret)
    }
}
