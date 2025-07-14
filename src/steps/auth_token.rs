use crate::step::{AuthTokenHeader, Check, Dependency, RequestBuilder, RequestResult, Step};
use crate::{spec::*, types::*};
use chris_oag::models;
use either::Either;

/// A [Step] to make sure that a user exists.
#[derive(Debug, Copy, Clone)]
pub(crate) struct UserExists<'a> {
    username: &'a Username,
    details: &'a UserDetails,
    url: &'a CubeUrl,
}

impl Dependency for UserExists<'_> {
    type Value = Either<models::User, AuthTokenHeader>;
}

impl<'a> Step for UserExists<'a> {
    type Dep = ();
    type Res = models::AuthToken;
    type Out = Self;

    fn dependencies(&self) -> Self::Dep {}

    fn query(&self, _: &<Self::Dep as Dependency>::Value) -> RequestResult {
        request_auth_token(self.url, self.username, self.details)
    }

    fn check(&self, body: Self::Res) -> Check<Self::Out> {
        Check::Exists(
            self.clone(),
            Either::Right(AuthTokenHeader::new(body.token)),
        )
    }

    fn create(&self, _: &<Self::Dep as Dependency>::Value) -> RequestResult {
        let body = models::UserRequest {
            username: Some(self.username.to_string()),
            email: self
                .details
                .email
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}@example.org", self.username)),
            password: self.details.password.clone(),
            is_staff: None,
        };
        RequestBuilder::new(self.url.join("users/")).post(&body)
    }
}

/// Request for an authorization token.
pub(crate) struct UserGetAuthToken<'a> {
    username: &'a Username,
    details: &'a UserDetails,
    url: &'a CubeUrl,
}

/// [Dependency] on a user's authorization token.
pub(crate) struct AuthTokenFor<'a>(pub &'a Username);

impl Dependency for AuthTokenFor<'_> {
    type Value = AuthTokenHeader;
}

impl<'a> Step for UserGetAuthToken<'a> {
    type Dep = UserExists<'a>;
    type Res = models::AuthToken;
    type Out = AuthTokenFor<'a>;

    fn dependencies(&self) -> Self::Dep {
        UserExists {
            username: self.username,
            details: self.details,
            url: self.url,
        }
    }

    fn extract(&self, user_exists: &<Self::Dep as Dependency>::Value) -> Check<Self::Out> {
        if let Either::Right(token) = user_exists {
            Check::Exists(AuthTokenFor(self.username), token.clone())
        } else {
            Check::DoesNotExist
        }
    }

    fn query(&self, _: &<Self::Dep as Dependency>::Value) -> RequestResult {
        request_auth_token(self.url, self.username, self.details)
    }

    fn check(&self, body: Self::Res) -> Check<Self::Out> {
        Check::Exists(
            AuthTokenFor(self.username),
            AuthTokenHeader::new(body.token),
        )
    }

    fn create(&self, _: &<Self::Dep as Dependency>::Value) -> RequestResult {
        request_auth_token(self.url, self.username, self.details)
    }
}

fn request_auth_token(url: &CubeUrl, username: &Username, details: &UserDetails) -> RequestResult {
    let credentials = models::AuthTokenRequest {
        username: username.to_string(),
        password: details.password.to_string(),
    };
    RequestBuilder::new(url.join("auth-token/")).post(&credentials)
}
