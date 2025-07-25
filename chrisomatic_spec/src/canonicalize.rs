use compact_str::CompactString;

use crate::{spec::*, types::*};

///
pub fn canonicalize(
    values: impl IntoIterator<Item = GivenManifest>,
) -> Result<Manifest, ManifestError> {
    values
        .into_iter()
        .try_fold(Default::default(), merge)
        .and_then(|m| m.try_into())
}

fn merge(a: GivenManifest, b: GivenManifest) -> Result<GivenManifest, ManifestError> {
    todo!()
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error("No global configuration found.")]
    NoGlobal,
    #[error("Missing required: `{}`", .0.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>().join(", "))]
    Missing(&'static [&'static str]),
}

impl TryFrom<GivenManifest> for Manifest {
    type Error = ManifestError;

    fn try_from(value: GivenManifest) -> Result<Self, Self::Error> {
        let global: Global = value.global.ok_or(ManifestError::NoGlobal)?.try_into()?;
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
        Ok(Manifest {
            global,
            user,
            userfiles: value.userfiles,
            feeds: value.feeds,
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
                .unwrap_or_else(|| CompactString::const_new("@example.org")),
            public_cube: value.public_cube.unwrap_or_else(|| {
                CubeUrl::try_new("https://cube.chrisproject.org/api/v1/").unwrap()
            }),
        })
    }
}
