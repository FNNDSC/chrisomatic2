use std::collections::{HashMap, HashSet};

use compact_str::{CompactString, ToCompactString};

use crate::{PluginSpec, spec::*, types::*};

/// Merge multiple [GivenManifest] into one.
pub fn reduce(
    values: impl IntoIterator<Item = GivenManifest>,
) -> Result<GivenManifest, ManifestError> {
    values.into_iter().try_fold(Default::default(), merge)
}

fn merge(a: GivenManifest, b: GivenManifest) -> Result<GivenManifest, ManifestError> {
    let mut plugins = a.plugins;
    let mut b_plugins = b.plugins;
    plugins.append(&mut b_plugins);
    Ok(GivenManifest {
        global: merge_global(a.global, b.global)?,
        user: merge_users(a.user, b.user)?,
        compute_resource: merge_maps(a.compute_resource, b.compute_resource, "compute_resource")?,
        plugins,
    })
}

fn merge_global(a: GivenGlobal, b: GivenGlobal) -> Result<GivenGlobal, ManifestError> {
    Ok(GivenGlobal {
        cube: none_xor(a.cube, b.cube, "global.cube")?,
        admin: merge_credentials(a.admin, b.admin)?,
        email_domain: none_xor(a.email_domain, b.email_domain, "global.email_domain")?,
        peer: none_xor(a.peer, b.peer, "global.public_cube")?,
    })
}

fn merge_credentials(
    a: Option<UserCredentials>,
    b: Option<UserCredentials>,
) -> Result<Option<UserCredentials>, ManifestError> {
    if let Some(a) = a {
        if let Some(b) = b {
            Err(ManifestError::DuplicateValue {
                key: "global.admin",
                a: a.username.into_inner().into_string(),
                b: b.username.into_inner().into_string(),
            })
        } else {
            Ok(Some(a))
        }
    } else {
        Ok(b)
    }
}

/// Return the value of `a` or `b`, but produce an error if both are [Some].
fn none_xor<T: Into<String>>(
    a: Option<T>,
    b: Option<T>,
    key: &'static str,
) -> Result<Option<T>, ManifestError> {
    if let Some(a) = a {
        if let Some(b) = b {
            Err(ManifestError::DuplicateValue {
                key,
                a: a.into(),
                b: b.into(),
            })
        } else {
            Ok(Some(a))
        }
    } else {
        Ok(b)
    }
}

fn merge_maps<K: std::hash::Hash + Eq + std::string::ToString, V>(
    mut a: HashMap<K, V>,
    b: HashMap<K, V>,
    key: &'static str,
) -> Result<HashMap<K, V>, ManifestError> {
    for (k, v) in b.into_iter() {
        if a.get(&k).is_some() {
            return Err(ManifestError::RepeatedValue {
                key,
                value: k.to_string(),
            });
        }
        a.insert(k, v);
    }
    Ok(a)
}

fn merge_users<T>(
    mut a: HashMap<Username, T>,
    b: HashMap<Username, T>,
) -> Result<HashMap<Username, T>, ManifestError> {
    for (username, details) in b.into_iter() {
        if a.insert(username.clone(), details).is_some() {
            return Err(ManifestError::DuplicateUser(username));
        }
    }
    Ok(a)
}

/// Error merging [GivenManifest] into [Manifest].
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ManifestError {
    #[error("Missing required: `{}`", .0.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", "))]
    Missing(&'static [&'static str]),
    #[error("Username specified more than once: \"{0}\"")]
    DuplicateUser(Username),
    #[error("Duplicate value for `{key}` (first: \"{a}\", second: \"{b}\")")]
    DuplicateValue {
        key: &'static str,
        a: String,
        b: String,
    },
    #[error("Duplicate plugin: {}", .0.to_compact_string())]
    DuplicatePlugin(PluginSpec),
    #[error("Multiple definitions for {key}.{value}")]
    RepeatedValue { key: &'static str, value: String },
}

impl TryFrom<GivenManifest> for Manifest {
    type Error = ManifestError;

    fn try_from(value: GivenManifest) -> Result<Self, Self::Error> {
        let global: Global = value.global.try_into()?;
        let user = value
            .user
            .into_iter()
            .map(
                |(
                    username,
                    GivenUserDetails {
                        password,
                        email,
                        groups,
                    },
                )| {
                    let details = UserDetails {
                        groups,
                        password,
                        email: email
                            .unwrap_or_else(|| format!("{}@{}", &username, &global.email_domain)),
                    };
                    (username, details)
                },
            )
            .collect();
        validate_no_duplicate_plugins(&value.plugins)?;
        Ok(Manifest {
            global,
            user,
            compute_resource: value.compute_resource,
            plugins: value.plugins,
            // userfiles: value.userfiles,
            // feeds: value.feeds,
        })
    }
}

impl TryFrom<GivenGlobal> for Global {
    type Error = ManifestError;

    fn try_from(value: GivenGlobal) -> Result<Self, Self::Error> {
        Ok(Self {
            cube: value.cube.ok_or(ManifestError::Missing(&["global.cube"]))?,
            admin: value.admin.ok_or(ManifestError::Missing(&[
                "global.admin.username",
                "global.admin.password",
            ]))?,
            email_domain: value
                .email_domain
                .unwrap_or_else(|| CompactString::const_new("example.org")),
            peer: value.peer,
        })
    }
}

fn validate_no_duplicate_plugins(plugins: &[PluginConfig]) -> Result<(), ManifestError> {
    plugins
        .iter()
        .try_fold(HashSet::with_capacity(plugins.len()), |mut set, p| {
            let plugin = PluginSpec {
                name: p.name.clone(),
                version: p.version.clone(),
            };
            if set.contains(&plugin) {
                Err(ManifestError::DuplicatePlugin(plugin))
            } else {
                set.insert(plugin);
                Ok(set)
            }
        })
        .map(|_| ())
}
