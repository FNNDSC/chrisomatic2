use std::collections::HashMap;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::{plugin_spec::PluginSpec, share_target::ShareTarget, types::Username};

/// Chrisomatic manifest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub global: Option<Global>,
    pub user: HashMap<Username, UserDetails>,
    pub userfiles: Vec<UserFileSpec>,
    pub feeds: Vec<FeedSpec>,
}

/// Chrisomatic global configuration options.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Global {}

/// Chrisomatic user details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDetails {
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

/// Specification to create a user file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFileSpec {
    pub path: String,
    pub owner: Option<Username>,
    pub text: Option<String>,
    pub share: Vec<ShareTarget>,
}

/// Specification to create a feed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeedSpec {
    name: String,
    owner: Username,
    share: Vec<ShareTarget>,
    #[serde(flatten)]
    plugin: FsPluginSpec,
}

/// Specification to run a _ts_-type plugin.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FsPluginSpec {
    Dircopy {
        #[serde(rename = "dircopy")]
        path: String,
    },
    Other {
        plugin: PluginSpec,
        args: HashMap<CompactString, toml::Value>,
    },
}
