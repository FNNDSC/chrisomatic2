use std::{collections::HashMap, process::Command};

use color_eyre::eyre::{Context, bail};

/// Wrapper around `docker` or `podman` CLI.
#[derive(Clone, Debug)]
pub struct ContainerEngine(pub String);

impl ContainerEngine {
    fn running_images(&self) -> color_eyre::Result<HashMap<String, String>> {
        let args = ["ps", "--format", r#"{{ printf "%s %s" .ID .Image }}"#];
        self.cmd(&args)?
            .split("\n")
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                if let Some((id, image)) = line.split_once(' ') {
                    Ok((id.to_string(), image.to_string()))
                } else {
                    bail!(
                        "Unexpected line of output from command `{} {}`: {line}",
                        &self.0,
                        args.join(" ")
                    )
                }
            })
            .collect()
    }

    fn cmd(&self, args: &[&str]) -> color_eyre::Result<String> {
        let output = Command::new(&self.0)
            .args(args)
            .output()
            .wrap_err_with(|| format!("Could not run command: `{} {}`", &self.0, args.join(" ")))?;
        if !output.status.success() {
            bail!(
                "Command `{} {}` exited with status {}",
                &self.0,
                args.join(" "),
                output.status
            );
        }
        String::from_utf8(output.stdout).wrap_err_with(|| {
            format!(
                "Command `{} {}` wrote invalid UTF-8 to stdout",
                &self.0,
                args.join(" ")
            )
        })
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use super::*;
    use rstest::*;

    const TEST_IMAGE: &'static str = "ghcr.io/knative/helloworld-go:latest";
    const TEST_IMAGE_RENAMED: &'static str = "localhost/fnndsc/cube:fake";
    const TEST_CONTAINER_LABEL: &'static str = "org.chrisproject.test=chrisomatic";
    const TEST_PORT: u32 = 12345;

    #[rstest]
    fn test_container_engine(example_container: &TestContainer) {
        let _ = example_container;
        let running_images: HashSet<_> = ContainerEngine("podman".to_string())
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
