use std::{collections::HashMap, process::Command};

use color_eyre::eyre::{Context, bail};

/// Wrapper around `docker` or `podman` CLI.
#[derive(Clone, Debug)]
pub struct ContainerEngine(pub String);

impl ContainerEngine {
    fn running_images(&self) -> color_eyre::Result<HashMap<String, String>> {
        let args = ["ps", "--format", r#""{{ printf "%s %s" .ID .Image }}""#];
        self.cmd(&args)?
            .split("\n")
            .map(|line| {
                if let Some((id, image)) = line.split_once(' ') {
                    Ok((id.to_string(), image.to_string()))
                } else {
                    bail!("Unexpected output from command")
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
