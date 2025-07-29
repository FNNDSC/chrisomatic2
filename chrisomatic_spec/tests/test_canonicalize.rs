use std::collections::{HashMap, HashSet};

use chrisomatic_spec::*;
use compact_str::CompactString;
use pretty_assertions::assert_eq;

#[test]
fn test_convert_empty() {
    let empty: GivenManifest = Default::default();
    let actual: Result<Manifest, _> = empty.try_into();
    let expected = ManifestError::Missing(&["global.cube"]);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_convert_no_cube() {
    let manifest = GivenManifest {
        global: GivenGlobal {
            cube: None,
            admin: Some(UserCredentials::basic_auth("chris", "chris1234")),
            email_domain: None,
            peer: None,
        },
        ..Default::default()
    };
    let actual: Result<Manifest, _> = manifest.try_into();
    let expected = ManifestError::Missing(&["global.cube"]);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_reduce_duplicate_user() {
    let manifest1 = GivenManifest {
        global: GivenGlobal {
            cube: Some(CubeUrl::try_new("https://cube.example.org/api/v1/").unwrap()),
            ..Default::default()
        },
        user: create_users(["alice", "bobby"]),
        ..Default::default()
    };
    let manifest2 = GivenManifest {
        global: Default::default(),
        user: create_users(["bobby", "samuel"]),
        ..Default::default()
    };
    let actual = reduce([manifest1, manifest2]);
    let duplicate = Username::new(CompactString::const_new("bobby"));
    let expected = ManifestError::DuplicateUser(duplicate);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_reduce_multiple_users() {
    let manifest1 = GivenManifest {
        global: GivenGlobal {
            cube: Some(CubeUrl::try_new("https://cube.example.org/api/v1/").unwrap()),
            admin: Some(UserCredentials::basic_auth("chris", "chris1234")),
            ..Default::default()
        },
        user: create_users(["alice", "bobby"]),
        ..Default::default()
    };
    let manifest2 = GivenManifest {
        global: Default::default(),
        user: create_users(["samuel", "washington"]),
        ..Default::default()
    };
    let actual: HashSet<_> = reduce([manifest1, manifest2])
        .unwrap()
        .user
        .into_keys()
        .collect();
    let expected: HashSet<_> = create_users(["alice", "bobby", "samuel", "washington"])
        .into_keys()
        .collect();
    assert_eq!(actual, expected)
}

#[test]
fn test_validate_no_plugin_duplicates() {
    let given = GivenManifest {
        global: GivenGlobal {
            cube: Some(CubeUrl::try_new("https://cube.example.org/api/v1/").unwrap()),
            admin: Some(UserCredentials::basic_auth("chris", "chris1234")),
            ..Default::default()
        },
        plugins: vec![
            PluginConfig {
                name: CompactString::const_new("pl-dup"),
                version: Some(CompactString::const_new("3.2.1")),
                compute_resources: vec![],
            },
            PluginConfig {
                name: CompactString::const_new("pl-uni"),
                version: Some(CompactString::const_new("3.2.1")),
                compute_resources: vec![],
            },
            PluginConfig {
                name: CompactString::const_new("pl-dup"),
                version: Some(CompactString::const_new("3.2.1")),
                compute_resources: vec![],
            },
            PluginConfig {
                name: CompactString::const_new("pl-uni"),
                version: Some(CompactString::const_new("1.2.3")),
                compute_resources: vec![],
            },
        ],
        ..Default::default()
    };
    let actual: Result<Manifest, _> = given.try_into();
    let expected = ManifestError::DuplicatePlugin(PluginSpec::new("pl-dup", "3.2.1"));
    assert_eq!(actual, Err(expected))
}

fn create_users(
    usernames: impl IntoIterator<Item = &'static str>,
) -> HashMap<Username, GivenUserDetails> {
    usernames
        .into_iter()
        .map(|username| {
            (
                Username::new(CompactString::const_new(username)),
                GivenUserDetails {
                    password: format!("{username}1234"),
                    email: None,
                    groups: vec![],
                },
            )
        })
        .collect()
}
