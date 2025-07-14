use compact_str::CompactString;
use nutype::nutype;

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
