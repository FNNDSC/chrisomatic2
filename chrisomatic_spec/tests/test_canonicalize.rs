use std::collections::{HashMap, HashSet};

use chrisomatic_spec::*;
use compact_str::CompactString;
use pretty_assertions::assert_eq;

#[test]
fn test_canonicalize_empty() {
    let actual = canonicalize([]);
    let expected = ManifestError::Missing(&["global.cube"]);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_canonicalize_no_global_cube() {
    let manifest = GivenManifest {
        global: GivenGlobal {
            cube: None,
            admin: Some(UserCredentials::basic_auth("chris", "chris1234")),
            email_domain: None,
            public_cube: None,
        },
        ..Default::default()
    };
    let actual = canonicalize([manifest]);
    let expected = ManifestError::Missing(&["global.cube"]);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_canonicalize_duplicate_user() {
    let manifest1 = GivenManifest {
        global: GivenGlobal {
            cube: Some(CubeUrl::try_new("https://cube.example.org/api/v1/").unwrap()),
            ..Default::default()
        },
        user: create_users(["alice", "bobby"]),
    };
    let manifest2 = GivenManifest {
        global: Default::default(),
        user: create_users(["bobby", "samuel"]),
    };
    let actual = canonicalize([manifest1, manifest2]);
    let duplicate = Username::new(CompactString::const_new("bobby"));
    let expected = ManifestError::DuplicateUser(duplicate);
    assert_eq!(actual, Err(expected));
}

#[test]
fn test_canonicalize_merge_users() {
    let manifest1 = GivenManifest {
        global: GivenGlobal {
            cube: Some(CubeUrl::try_new("https://cube.example.org/api/v1/").unwrap()),
            admin: Some(UserCredentials::basic_auth("chris", "chris1234")),
            ..Default::default()
        },
        user: create_users(["alice", "bobby"]),
    };
    let manifest2 = GivenManifest {
        global: Default::default(),
        user: create_users(["samuel", "washington"]),
    };
    let actual: HashSet<_> = canonicalize([manifest1, manifest2])
        .unwrap()
        .user
        .into_keys()
        .collect();
    let expected: HashSet<_> = create_users(["alice", "bobby", "samuel", "washington"])
        .into_keys()
        .collect();
    assert_eq!(actual, expected)
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
