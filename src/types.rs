use compact_str::CompactString;
use nutype::nutype;

/// ChRIS backend API URL.
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
pub struct CubeUrl(String);

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
