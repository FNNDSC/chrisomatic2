use nutype::nutype;

use crate::{dependency::Dependency, spec::FeedSpec, types::Username};
use chris_oag::models;

pub(crate) struct AuthTokenFor<'a>(pub &'a Username);
pub(crate) struct FeedFor<'a>(pub &'a FeedSpec);

impl<'a> Dependency for AuthTokenFor<'a> {
    type Value = AuthTokenHeader;
}

/// `Authorization` header value with a token.
#[nutype(
    sanitize(with = |mut s: String| { s.insert_str(0, "Token "); s}),
    derive(Display, Debug, Clone, Eq, PartialEq, AsRef, Deref)
)]
pub struct AuthTokenHeader(String);

impl TryFrom<&AuthTokenHeader> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;
    fn try_from(value: &AuthTokenHeader) -> Result<Self, Self::Error> {
        http::HeaderValue::from_str(value.as_str())
    }
}

impl<'a> Dependency for FeedFor<'a> {
    type Value = models::Feed;
}

pub(crate) struct FeedUserPermissionFor<'a>(pub &'a FeedSpec, pub &'a Username);

impl Dependency for FeedUserPermissionFor<'_> {
    type Value = models::FeedUserPermission;
}
