use compact_str::CompactString;
use nutype::nutype;

/// ChRIS backend API URL.
#[nutype(
    validate(predicate = |s: &str| (s.starts_with("http://") || s.starts_with("https://") && s.ends_with("/api/v1/"))),
    derive(Display, Debug, Clone, Eq, PartialEq, Hash, FromStr, AsRef, Deref, Serialize, Deserialize)
)]
pub struct CubeUrl(String);

impl CubeUrl {
    pub fn join(&self, path: impl std::fmt::Display) -> String {
        format!("{self}{path}")
    }
}

/// ChRIS user username.
#[nutype(derive(
    Display,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    FromStr,
    AsRef,
    Deref,
    Serialize,
    Deserialize,
    From,
    Into,
))]
pub struct Username(CompactString);

/// ChRIS group name.
#[nutype(derive(
    Display,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    FromStr,
    AsRef,
    Deref,
    Serialize,
    Deserialize,
    From,
    Into,
))]
pub struct Group(CompactString);
