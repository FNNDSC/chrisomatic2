use crate::step::{Check, Dependency, RequestBuilder, RequestResult, Step};
use crate::{spec::*, types::*};
use chris_oag::models;

use super::auth_token::AuthTokenFor;

// pub(crate) struct UserfileShareWithUser<'a> {
//     path: &'a str,
//     owner: &'a Username,
//     user: &'a Username,
// }

/// An [Step] to share a [models::Feed] with a [models::User].
#[derive(Debug, Copy, Clone)]
pub(crate) struct FeedShareWithUser<'a> {
    feed: &'a FeedSpec,
    user: &'a Username,
}

impl Dependency for FeedShareWithUser<'_> {
    type Value = models::FeedUserPermission;
}

/// A [Dependency] on a [models::Feed].
#[derive(Debug, Copy, Clone)]
pub(crate) struct FeedFor<'a>(&'a FeedSpec);

impl Dependency for FeedFor<'_> {
    type Value = models::Feed;
}

impl<'a> Step for FeedShareWithUser<'a> {
    type Dep = (AuthTokenFor<'a>, FeedFor<'a>);
    type Res = models::PaginatedFeedUserPermissionList;
    type Out = Self;

    fn query(&self, (token, feed): &<Self::Dep as Dependency>::Value) -> RequestResult {
        let uri = format!("{}?username={}&limit=1", feed.user_permissions, self.user);
        RequestBuilder::new(uri).auth(token).get()
    }

    fn dependencies(&self) -> Self::Dep {
        (AuthTokenFor(self.user), FeedFor(self.feed))
    }

    fn check(&self, body: Self::Res) -> Check<Self::Out> {
        if let Some(permission) = body.results.into_iter().next() {
            Check::Exists(self.clone(), permission)
        } else {
            Check::DoesNotExist
        }
    }

    fn create(&self, (token, feed): &<Self::Dep as Dependency>::Value) -> RequestResult {
        let body = models::FeedUserPermissionRequest {
            username: self.user.to_string(),
        };
        RequestBuilder::new(&feed.user_permissions)
            .auth(token)
            .post(&body)
    }
}
