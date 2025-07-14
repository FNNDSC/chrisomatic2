/// The `Dependency` trait associates an "input" type with a "canonical" type
/// to be resolved at runtime.
///
/// This is best explained by example: suppose a [crate::spec::FeedSpec]
/// specifies for the feed to be shared with a user. The API call to share a
/// feed with a user requires the client to know (1) the owner's auth_token,
/// (2) the feed's ID number. The dependency on the owner's auth_token is
/// represented by [crate::deps::AuthTokenFor], whose [Dependency::Value] is
/// [String], and the dependency on knowing the feed's ID number is represented
/// by [crate::deps::FeedIdFor], whose [Dependency::Value] is [u32].
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
