/// A `Step` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **check** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
///
/// ## Behvaior
///
/// 1. [Step::dependencies] is called to get the dependencies of this step.
/// 2. [Step::extract] is called. If [Step::extract] is overridden and it
///    returns [Check::Exists], then the return value is accepted. It could
///    also return [Check::NeedsModiciation], at which point it will jump
///    to call [Step::modify].
/// 3. If [Step::extract] returns [Check::DoesNotExist] (which is what the
///    default implementation does), [Step::query] is called to produce an
///    HTTP request.
/// 4. If the response to the request produced by [Step::query] produces a
///    200-299 response, the response body is deserialized and passed to
///    [Step::check].
pub(crate) trait Step {
    /// Data dependencies for [Step::query].
    type Dep: Dependency;
    /// Expected response type for [Step::query].
    type Res: serde::de::DeserializeOwned;
    /// Representation of the resource created by this action.
    type Out: Dependency;

    /// Specify the dependencies for checking whether this resource exists.
    fn dependencies(&self) -> Self::Dep;

    /// Check whether the resource already exists within its dependencies.
    fn extract(&self, _dependencies: &<Self::Dep as Dependency>::Value) -> Check<Self::Out> {
        Check::DoesNotExist
    }

    /// Create an HTTP request which queries the API for the resource's existence.
    fn query(&self, dependencies: &<Self::Dep as Dependency>::Value) -> RequestResult;

    /// Check the response to the request created by [Step::query] and decide
    /// how to proceed with the resource creation.
    fn check(&self, body: Self::Res) -> Check<Self::Out>;

    /// Create an HTTP request to create the resource.
    fn create(&self, dependencies: &<Self::Dep as Dependency>::Value) -> RequestResult;

    /// Create an HTTP request to modify the pre-existing resource.
    /// Return [None] if the resource is unmodifiable.
    fn modify(&self, _dependencies: &<Self::Dep as Dependency>::Value) -> Option<RequestResult> {
        None
    }
}

/// Possible outcomes when checking for the existence of an API resource.
pub(crate) enum Check<T: Dependency> {
    /// The resource exists and is correct.
    Exists(T, T::Value),
    /// The resource does not exist and needs to be created.
    DoesNotExist,
    /// The resource exists but needs modification.
    NeedsModification,
}

/// Request build result.
pub(crate) type RequestResult = Result<http::Request<Vec<u8>>, CreateRequestError>;

/// Convenience wrapper for [http::request::Builder].
pub(crate) struct RequestBuilder(http::request::Builder);

const USER_AGENT: &'static str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

impl RequestBuilder {
    /// Create a request builder for the specified URI.
    pub(crate) fn new(uri: impl TryInto<http::Uri, Error = impl Into<http::Error>>) -> Self {
        let builder = http::Request::builder()
            .header(http::header::USER_AGENT, USER_AGENT)
            .uri(uri);
        Self(builder)
    }

    /// Add token authorization to this request.
    pub(crate) fn auth(self, token: &AuthTokenHeader) -> Self {
        let builder = self.0.header(http::header::AUTHORIZATION, token);
        Self(builder)
    }

    /// Construct a GET request which accepts JSON.
    pub(crate) fn get(self) -> RequestResult {
        self.0
            .method("GET")
            .header(http::header::ACCEPT, "application/json")
            .body(Vec::new())
            .map_err(CreateRequestError::Request)
    }

    /// Construct a POST request with a JSON body which accepts JSON.
    pub(crate) fn post<T: serde::Serialize>(self, body: &T) -> RequestResult {
        let req = self
            .0
            .method("POST")
            .header(http::header::ACCEPT, "application/json")
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(body)?)?;
        Ok(req)
    }
}

/// `Authorization` header value with a token.
#[nutype::nutype(
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

/// Error constructing request.
#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateRequestError {
    /// Error serializing request body.
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    /// Error with request.
    #[error(transparent)]
    Request(#[from] http::Error),
}

/// The `Dependency` trait associates an "input" type with a "canonical" type
/// to be resolved at runtime.
///
/// This is best explained by example: suppose a [crate::spec::FeedSpec]
/// specifies for the feed to be shared with a user. The API call to share a
/// feed with a user requires the client to know (1) the owner's auth_token,
/// (2) the feed's URL (which contains its ID number). The dependency on the
/// owner's auth_token is represented by [crate::steps::AuthTokenFor], whose
/// [Dependency::Value] is a type representing auth_tokens, and the dependency
/// on knowing the feed's ID number is represented by [crate::steps::FeedFor],
/// whose [Dependency::Value] is [chris_oag::models::Feed].
pub(crate) trait Dependency {
    /// The type for the canonical value.
    type Value;
}

impl Dependency for () {
    type Value = ();
}

impl<A, B> Dependency for (A, B)
where
    A: Dependency,
    B: Dependency,
{
    type Value = (<A as Dependency>::Value, <B as Dependency>::Value);
}
impl<A, B, C> Dependency for (A, B, C)
where
    A: Dependency,
    B: Dependency,
    C: Dependency,
{
    type Value = (
        <A as Dependency>::Value,
        <B as Dependency>::Value,
        <C as Dependency>::Value,
    );
}
impl<A, B, C, D> Dependency for (A, B, C, D)
where
    A: Dependency,
    B: Dependency,
    C: Dependency,
    D: Dependency,
{
    type Value = (
        <A as Dependency>::Value,
        <B as Dependency>::Value,
        <C as Dependency>::Value,
        <D as Dependency>::Value,
    );
}
impl<A, B, C, D, E> Dependency for (A, B, C, D, E)
where
    A: Dependency,
    B: Dependency,
    C: Dependency,
    D: Dependency,
    E: Dependency,
{
    type Value = (
        <A as Dependency>::Value,
        <B as Dependency>::Value,
        <C as Dependency>::Value,
        <D as Dependency>::Value,
        <E as Dependency>::Value,
    );
}
