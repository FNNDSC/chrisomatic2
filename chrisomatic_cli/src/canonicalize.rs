use chrisomatic_spec::{CubeUrl, GivenGlobal, GivenManifest, Manifest, UserCredentials};
use color_eyre::eyre::Context;

use crate::container_engine::{ContainerDetails, ContainerEngine};

const CUBE_CONTAINER_PORT: &'static str = "8000/tcp";
const CUBE_IMAGE_CONTAINS: &'static str = "/fnndsc/cube:";
const CUBE_SUPERUSER_ENV: &'static str = "CHRIS_SUPERUSER_PASSWORD";

/// Convert [GivenManifest] to [Manifest], auto-filling values for `global`
/// from a locally running CUBE if needed.
///
/// WARNING: does blocking I/O.
pub(crate) fn canonicalize(mut given: GivenManifest) -> color_eyre::Result<Manifest> {
    given.global = guess_global_config(given.global)?;
    Ok(given.try_into()?)
}

fn guess_global_config(given: GivenGlobal) -> color_eyre::Result<GivenGlobal> {
    if given.is_none() {
        infer_global_from_running_container().with_context(
            ||
            "Trying to infer `global` configuration from ChRIS backend running in local container",
        )
    } else {
        Ok(given)
    }
}

/// Try to get the [GivenGlobal] from the CUBE container running in Docker or Podman.
fn infer_global_from_running_container() -> color_eyre::Result<GivenGlobal> {
    let engines = ["podman", "docker"]
        .iter()
        .map(which::which)
        .filter_map(|engine| engine.map(ContainerEngine::from).ok());
    for engine in engines {
        if let Some(cube) = get_chris_container(&engine)? {
            return Ok(cube);
        }
    }
    Ok(Default::default())
}

/// A container is identified as _CUBE_ if all:
///
/// - image contains [CUBE_IMAGE_CONTAINS]
/// - container port [CUBE_CONTAINER_PORT] is bound to host
/// - container has environment variable [CUBE_SUPERUSER_ENV]
fn get_chris_container(engine: &ContainerEngine) -> color_eyre::Result<Option<GivenGlobal>> {
    for (container_id, image) in engine.running_images()? {
        if !image.contains(CUBE_IMAGE_CONTAINS) {
            continue;
        }
        let details = engine.inspect(container_id)?;
        let cube = get_cube_details(details);
        if cube.is_some() {
            return Ok(cube);
        }
    }
    Ok(None)
}

fn get_cube_details(mut details: ContainerDetails) -> Option<GivenGlobal> {
    let socket = details
        .ports
        .into_iter()
        .filter_map(|(socket, port)| {
            if port == CUBE_CONTAINER_PORT {
                Some(socket)
            } else {
                None
            }
        })
        .next()?;
    let password = details.env.remove(CUBE_SUPERUSER_ENV)?;
    let url = CubeUrl::try_new(format!("http://localhost:{}/api/v1/", socket.port())).unwrap();
    Some(GivenGlobal {
        cube: Some(url),
        admin: Some(UserCredentials::basic_auth("chris", password)),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {

    use std::{collections::HashSet, process::Command};

    use super::*;
    use rstest::*;

    const TEST_IMAGE: &'static str = "ghcr.io/knative/helloworld-go:latest";
    const TEST_IMAGE_RENAMED: &'static str = "localhost/fnndsc/cube:fake";
    const TEST_CONTAINER_LABEL: &'static str = "org.chrisproject.test=chrisomatic";
    const TEST_PORT: u32 = 12345;
    const TEST_PASSWORD: &'static str = "i@mV3r^$ecur";

    fn test_infer_global_from_podman(example_container: &TestContainer) {
        let _ = example_container;
        let expected = GivenGlobal {
            cube: Some(CubeUrl::try_new(format!("http://localhost:{TEST_PORT}/api/v1/")).unwrap()),
            admin: Some(UserCredentials::basic_auth("chris", TEST_PASSWORD)),
            ..Default::default()
        };
        let actual = infer_global_from_running_container().unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    fn test_container_engine(example_container: &TestContainer) {
        let _ = example_container;
        let running_images: HashSet<_> = ContainerEngine("podman".into())
            .running_images()
            .unwrap()
            .into_values()
            .collect();
        assert!(running_images.contains(TEST_IMAGE_RENAMED));
    }

    #[fixture]
    #[once]
    fn example_container() -> TestContainer {
        let ps = Command::new("podman").arg("ps").arg("-q").output().unwrap();
        assert!(
            ps.stdout.is_empty(),
            "Test cannot proceed, please stop all Podman containers.
    HINT: run

        podman kill $(podman ps --filter label={TEST_CONTAINER_LABEL} -q)

    to clean up previous test failures."
        );

        let existing = Command::new("podman")
            .args(["images", "-q", TEST_IMAGE_RENAMED])
            .output()
            .unwrap();
        assert!(existing.status.success());
        if existing.stdout.is_empty() {
            let status = Command::new("podman")
                .args(["pull", TEST_IMAGE])
                .output()
                .unwrap()
                .status;
            assert!(status.success());
            let status = Command::new("podman")
                .args(["tag", TEST_IMAGE, TEST_IMAGE_RENAMED])
                .output()
                .unwrap()
                .status;
            assert!(status.success());
        }
        let output = Command::new("podman")
            .arg("run")
            .arg("--rm")
            .arg("-d")
            .arg("--label")
            .arg(TEST_CONTAINER_LABEL)
            .arg("--cpus=0.1")
            .arg("--memory=1m")
            .arg("-p")
            .arg(format!("127.0.0.1:{TEST_PORT}:8080"))
            .arg("-e")
            .arg(format!("{CUBE_SUPERUSER_ENV}={TEST_PASSWORD}"))
            .arg(TEST_IMAGE_RENAMED)
            .output()
            .unwrap();
        assert!(output.status.success());
        TestContainer(String::from_utf8(output.stdout).unwrap())
        // Command::new("podman").args(["pull"])
    }

    /// Podman container ID, `podman kill` is called when dropped.
    struct TestContainer(String);

    impl Drop for TestContainer {
        fn drop(&mut self) {
            let output = Command::new("podman")
                .arg("kill")
                .arg(&self.0)
                .output()
                .unwrap();
            assert!(output.status.success())
        }
    }
}
