use nutype::nutype;

use crate::{dependency::Dependency, spec::FeedSpec, types::Username};
use chris_oag::models;

pub(crate) struct AuthTokenFor<'a>(pub &'a Username);
pub(crate) struct FeedFor<'a>(pub &'a FeedSpec);

impl<'a> Dependency for AuthTokenFor<'a> {
    type Value = AuthTokenHeader;
}

impl TryFrom<&AuthTokenHeader> for http::HeaderValue {
    type Error = http::header::InvalidHeaderValue;
    fn try_from(value: &AuthTokenHeader) -> Result<Self, Self::Error> {
        http::HeaderValue::from_str(value.as_str())
    }
}
