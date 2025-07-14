use chris_oag::models;
use nutype::nutype;

use crate::spec::*;
use crate::step::{Check, Dependency, RequestResult, Step};
use crate::types::*;

/// `Authorization` header value with a token.
#[nutype(
    sanitize(with = |mut s: String| { s.insert_str(0, "Token "); s}),
    derive(Display, Debug, Clone, Eq, PartialEq, AsRef, Deref)
)]
pub(crate) struct AuthTokenHeader(String);

impl TryFrom<&AuthTokenHeader> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;

    fn try_from(value: &AuthTokenHeader) -> Result<Self, Self::Error> {
        http::HeaderValue::from_str(value.as_str())
    }
}

/// Request for an authorization token.
pub(crate) struct UserGetAuthToken<'a> {
    username: &'a Username,
    details: &'a UserDetails,
    url: &'a CubeUrl,
}

/// [Dependency] on a user's authorization token.
pub(crate) struct AuthTokenFor<'a>(&'a Username);

impl Dependency for AuthTokenFor<'_> {
    type Value = AuthTokenHeader;
}

impl<'a> Step for UserGetAuthToken<'a> {
    type Dep = ();
    type Res = models::AuthToken;
    type Out = AuthTokenFor<'a>;

    fn dependencies(&self) -> Self::Dep {}

    fn query(&self, _: &<Self::Dep as Dependency>::Value) -> RequestResult {
        let url = format!("{}auth-token/", self.url);
        let credentials = models::AuthTokenRequest {
            username: self.username.to_string(),
            password: self.details.password.to_string(),
        };
        let req = http::Request::builder()
            .uri(url)
            .header("Accept", "application/json")
            .body(serde_json::to_vec(&credentials)?)?;
        Ok(req)
    }

    fn check(&self, body: Self::Res) -> Check<Self::Out> {
        Check::Exists(
            AuthTokenFor(self.username),
            AuthTokenHeader::new(body.token),
        )
    }
}

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

    fn check(&self, body: Self::Res) -> Check<Self::Out> {
        if let Some(permission) = body.results.into_iter().next() {
            Check::Exists(self.clone(), permission)
        } else {
            Check::DoesNotExist
        }
    }
}
