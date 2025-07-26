use std::collections::HashMap;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::types::*;

/// User-supplied input chrisomatic manifest. See [crate::canonicalize].
/// Compared to [Manifest], some fields are [Option] (for user convenience).
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GivenManifest {
    #[serde(skip_serializing_if = "GivenGlobal::is_none")]
    pub global: GivenGlobal,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub user: HashMap<Username, GivenUserDetails>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub userfiles: Vec<UserFileSpec>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub feeds: Vec<FeedSpec>,
}

/// User-supplied input for global configuration.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GivenGlobal {
    /// URL of ChRIS backend (required, but sometimes can be inferred).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cube: Option<CubeUrl>,
    /// Admin user credentials (optional, but required for adding new plugins).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<UserCredentials>,
    /// Domain name to use for emails of users with unspecified email.
    /// (Default: "example.org")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_domain: Option<CompactString>,
    /// Public CUBE from where to get plugins from.
    /// (Default: "https://cube.chrisproject.org/api/v1/")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_cube: Option<CubeUrl>,
}

impl GivenGlobal {
    pub fn is_none(&self) -> bool {
        self.cube.is_none()
            && self.admin.is_none()
            && self.email_domain.is_none()
            && self.public_cube.is_none()
    }
}

/// Chrisomatic global configuration options.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Global {
    /// URL of ChRIS backend.
    pub cube: CubeUrl,
    /// Admin user credentials.
    pub admin: UserCredentials,
    /// Domain name to use for emails of users with unspecified email.
    pub email_domain: CompactString,
    /// Public CUBE from where to get plugins from.
    pub public_cube: CubeUrl,
}

/// Username and password/token.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserCredentials {
    pub username: Username,
    #[serde(flatten)]
    pub secret: PasswordOrToken,
}

impl UserCredentials {
    pub fn basic_auth(username: impl Into<Username>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            secret: PasswordOrToken::Password(password.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PasswordOrToken {
    Password(String),
    Token(String),
}

/// Chrisomatic manifest.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    pub global: Global,
    pub user: HashMap<Username, UserDetails>,
    // pub userfiles: Vec<UserFileSpec>,
    // pub feeds: Vec<FeedSpec>,
}

/// Given user details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GivenUserDetails {
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

/// Chrisomatic user details.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserDetails {
    pub password: String,
    pub email: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

// /// Specification to create a user file.
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct UserFileSpec {
//     pub path: String,
//     pub owner: Option<Username>,
//     pub text: Option<String>,
//     pub share: Vec<ShareTarget>,
// }

// /// Specification to create a feed.
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct FeedSpec {
//     name: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     owner: Option<Username>,
//     #[serde(skip_serializing_if = "Vec::is_empty")]
//     share: Vec<ShareTarget>,
//     #[serde(flatten)]
//     plugin: FsPluginSpec,
// }

// /// Specification to run a _ts_-type plugin.
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum FsPluginSpec {
//     Dircopy {
//         #[serde(rename = "dircopy")]
//         path: String,
//     },
//     Other {
//         plugin: PluginSpec,
//         args: HashMap<CompactString, ArgValue>,
//     },
// }

// /// Plugin argument values.
// ///
// /// Ref: <https://github.com/FNNDSC/ChRIS_ultron_backEnd/blob/v6.4.0/chris_backend/plugins/enums.py#L3-L5>
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// pub enum ArgValue {
//     /// string, path, or unextpath type
//     Stringish(String),
//     /// float type
//     Float(f64),
//     /// boolean type
//     Boolean(bool),
//     /// integer type
//     Integer(i64),
// }
