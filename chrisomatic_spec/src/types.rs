use compact_str::CompactString;
use nutype::nutype;

/// ChRIS backend API URL.
#[nutype(
    validate(predicate = |s: &str| (s.starts_with("http://") || s.starts_with("https://") && s.ends_with("/api/v1/"))),
    derive(Display, Debug, Clone, Eq, PartialEq, Hash, FromStr, AsRef, Deref, Serialize, Deserialize, Into)
)]
pub struct CubeUrl(String);

impl CubeUrl {
    pub fn to_url(&self) -> reqwest::Url {
        reqwest::Url::parse(self.as_str()).unwrap()
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

impl From<&'static str> for Username {
    fn from(value: &'static str) -> Self {
        Username::new(CompactString::const_new(value))
    }
}

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
pub struct EmailDomain(CompactString);
