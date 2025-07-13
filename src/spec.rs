use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::share_target::ShareTarget;

/// Chrisomatic manifest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub global: Option<Global>,
    pub user: HashMap<String, UserDetails>,
    pub userfiles: Vec<UserFileSpec>,
}

/// Chrisomatic global configuration options.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Global {}

/// Chrisomatic user details.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDetails {
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    groups: Vec<String>,
}

/// Specification to create a user file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFileSpec {
    path: String,
    owner: Option<String>,
    text: Option<String>,
    share: Vec<ShareTarget>,
}
