use http::HeaderValue;
use reqwest::{Request, header};
use serde::Serialize;

/// Extension trait for [reqwest::Request].
pub(crate) trait RequestBuilder {
    /// Add a JSON body.
    fn json<T: Serialize + ?Sized>(self, body: &T) -> serde_json::Result<Request>;

    /// Expect JSON response.
    fn accept_json(self) -> Request;

    /// Set the Authorization header.
    fn auth_token(self, value: impl AsRef<str>) -> Request;
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
}
