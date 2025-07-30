use std::collections::HashMap;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::types::*;

/// User-supplied input "manifest" for the _chrisomatic_ tool. A "manifest"
/// specifies desired state of the _ChRIS_ backend, e.g. what users, plugins,
/// feeds, etc. should exist.
///
/// ### See Also
///
/// - [crate::reduce] combines multiple `GivenManifest` into one.
/// - [Manifest] is the validated data of `GivenManifest`, obtained by the
///   [TryFrom] trait. The difference between `GivenManifest` and `Manifest`
///   is `GivenManifest` has many [Option] fields for user convenience.
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GivenManifest {
    /// Global configuration options.
    #[serde(default, skip_serializing_if = "GivenGlobal::is_none")]
    pub global: GivenGlobal,

    /// User accounts.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub user: HashMap<Username, GivenUserDetails>,

    /// Configuration of _ChRIS_ backend compute resources.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub compute_resource: HashMap<ComputeResourceName, ComputeResource>,

    /// List of plugins to register.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginConfig>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub userfiles: Vec<UserFileSpec>,
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // pub feeds: Vec<FeedSpec>,
}

/// Configuration of a [pfcon](https://github.com/FNNDSC/pfcon) to be
/// registered as a compute resource of _ChRIS_ backend.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ComputeResource {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub innetwork: Option<bool>,
    pub user: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_job_exec_seconds: Option<String>,
}

/// Configuration of a plugin to register.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PluginConfig {
    /// Plugin name.
    pub name: CompactString,
    /// Plugin version ([None] if use any plugin version).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<CompactString>,
    /// Name of compute resources to register this plugin to.
    /// If unspecified or empty, register the plugin to all compute resources.
    #[serde(alias = "computes", default, skip_serializing_if = "Vec::is_empty")]
    pub compute_resources: Vec<ComputeResourceName>,
}

/// User-supplied input for global configuration.
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    #[serde(skip_serializing_if = "Option::is_none", default = "public_cube")]
    pub peer: Option<CubeUrl>,
}

impl GivenGlobal {
    pub fn is_none(&self) -> bool {
        self.cube.is_none()
            && self.admin.is_none()
            && self.email_domain.is_none()
            && self.peer.is_none()
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<CubeUrl>,
}

fn public_cube() -> Option<CubeUrl> {
    CubeUrl::try_new("https://cube.chrisproject.org/api/v1/").ok()
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

/// Password or token for user authentication.
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
    /// Configuration of _ChRIS_ backend compute resources.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub compute_resource: HashMap<ComputeResourceName, ComputeResource>,

    /// List of plugins to register.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginConfig>,
}

/// Given user details.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GivenUserDetails {
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

/// Chrisomatic user details.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserDetails {
    pub password: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
