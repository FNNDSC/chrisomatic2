use base64::Engine;
use chrisomatic_spec::{PasswordOrToken, UserCredentials, Username};
use reqwest::{Request, header, header::HeaderValue};
use serde::Serialize;

/// Extension trait for [reqwest::Request].
pub(crate) trait RequestBuilder {
    /// Add a JSON body.
    fn json<T: Serialize + ?Sized>(self, body: &T) -> serde_json::Result<Request>;

    /// Expect JSON response.
    fn accept_json(self) -> Request;

    /// Set the Authorization header to use token authorization.
    fn auth_token(self, value: impl AsRef<str>) -> Request;

    /// Set the Authorization header to use HTTP basic auth.
    fn basic_auth(self, username: &Username, password: impl AsRef<str>) -> Request;

    /// Set the Authorization header.
    fn auth(self, credentials: &UserCredentials) -> Request;
}

const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

impl RequestBuilder for reqwest::Request {
    fn json<T: Serialize + ?Sized>(mut self, body: &T) -> serde_json::Result<Self> {
        let _ = self
            .headers_mut()
            .insert(header::CONTENT_TYPE, APPLICATION_JSON);
        let bytes = serde_json::to_vec(body)?;
        let _ = self.body_mut().insert(bytes.into());
        Ok(self)
    }

    fn accept_json(mut self) -> Self {
        let _ = self.headers_mut().insert(header::ACCEPT, APPLICATION_JSON);
        self
    }

    fn auth_token(mut self, value: impl AsRef<str>) -> Self {
        let _ = self.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(value.as_ref()).unwrap(),
        );
        self
    }

    fn basic_auth(mut self, username: &Username, password: impl AsRef<str>) -> Self {
        let _ = self.headers_mut().insert(
            header::AUTHORIZATION,
            format!("Basic {}", encode(username, password.as_ref()))
                .try_into()
                .unwrap(),
        );
        self
    }

    fn auth(self, credentials: &UserCredentials) -> Self {
        match &credentials.secret {
            PasswordOrToken::Password(password) => self.basic_auth(&credentials.username, password),
            PasswordOrToken::Token(token) => self.auth_token(token),
        }
    }
}

fn encode(username: &Username, password: &str) -> String {
    base64::prelude::BASE64_STANDARD.encode(format!("{username}:{password}"))
}
