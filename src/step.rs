pub(crate) type RequestResult = Result<http::Request<Vec<u8>>, CreateRequestError>;

/// A `Step` defines a set of operations against the _CUBE_ API regarding a specific resource:
///
/// 1. **check** whether the specified resource already exists
/// 2. **modify** an existing resource to match the spec
/// 3. **create** a resource matching the spec
pub(crate) trait Step {
    /// Data dependencies for [Step::query].
    type Dep: Dependency;
    /// Expected response type for [Step::query].
    type Res: serde::de::DeserializeOwned;
    /// Representation of the resource created by this action.
    type Out: Dependency;

    /// Specify the dependencies for checking whether this resource exists.
    fn dependencies(&self) -> Self::Dep;

    /// Create an HTTP request which queries the API for the resource's existence.
    fn query(&self, dependencies: &<Self::Dep as Dependency>::Value) -> RequestResult;

    /// Check whether the response to the request created by [Step::query]
    /// indicates the resource already exists.
    fn check(&self, body: Self::Res) -> Check<Self::Out>;
}

pub(crate) enum Check<T: Dependency> {
    Exists(T, T::Value),
    DoesNotExist,
    NeedsModification,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CreateRequestError {
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
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
