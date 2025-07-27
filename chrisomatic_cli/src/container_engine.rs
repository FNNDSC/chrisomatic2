use std::{
    collections::HashMap,
    ffi::OsString,
    net::{Ipv4Addr, SocketAddrV4},
    process::Command,
};

use color_eyre::eyre::{Context, bail};

/// Wrapper around `docker` or `podman` CLI.
#[derive(Clone, Debug)]
pub struct ContainerEngine(pub OsString);

impl ContainerEngine {
    /// List running container IDs and their images.
    pub(crate) fn running_images(&self) -> color_eyre::Result<HashMap<String, String>> {
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
                        self.0.to_string_lossy(),
                        args.join(" ")
                    )
                }
            })
            .collect()
    }

    /// Get the bound ports and environment variables of a container.
    pub(crate) fn inspect(
        &self,
        container_id: impl AsRef<str>,
    ) -> color_eyre::Result<ContainerDetails> {
        let args = [
            "inspect",
            "--format",
            r#"{ "PortBindings": {{ json .HostConfig.PortBindings }}, "Env": {{ json .Config.Env }} }"#,
            container_id.as_ref(),
        ];
        let output = self.cmd(&args)?;
        let data: ContainerInspect = serde_json::from_str(&output)?;
        Ok(data.try_into()?)
    }

    fn cmd(&self, args: &[&str]) -> color_eyre::Result<String> {
        let output = Command::new(&self.0)
            .args(args)
            .output()
            .wrap_err_with(|| {
                format!(
                    "Could not run command: `{} {}`",
                    self.0.to_string_lossy(),
                    args.join(" ")
                )
            })?;
        if !output.status.success() {
            // NOTE: if docker daemon is not running, that's OK, just move on.
            if output
                .stderr
                .starts_with(b"Cannot connect to the Docker daemon")
                && output
                    .stderr
                    .trim_ascii_end()
                    .ends_with(b"Is the docker daemon running?")
            {
                return Ok("".to_string());
            }
            bail!(
                "Command `{} {}` exited with status {}",
                self.0.to_string_lossy(),
                args.join(" "),
                output.status
            );
        }
        String::from_utf8(output.stdout).wrap_err_with(|| {
            format!(
                "Command `{} {}` wrote invalid UTF-8 to stdout",
                self.0.to_string_lossy(),
                args.join(" ")
            )
        })
    }
}

pub(crate) struct ContainerDetails {
    pub(crate) ports: HashMap<SocketAddrV4, String>,
    pub(crate) env: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerInspect {
    port_bindings: HashMap<String, Vec<ContainerHostSocketAddr>>,
    env: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerHostSocketAddr {
    host_ip: String,
    host_port: String,
}

impl TryFrom<ContainerHostSocketAddr> for SocketAddrV4 {
    type Error = color_eyre::Report;

    fn try_from(value: ContainerHostSocketAddr) -> Result<Self, Self::Error> {
        let ip = if value.host_ip.is_empty() {
            Ipv4Addr::new(127, 0, 0, 1)
        } else {
            value.host_ip.parse()?
        };
        let port = value.host_port.parse()?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl TryFrom<ContainerInspect> for ContainerDetails {
    type Error = color_eyre::Report;

    fn try_from(value: ContainerInspect) -> Result<Self, Self::Error> {
        let ports = value
            .port_bindings
            .into_iter()
            .flat_map(|(container_port, host_sockets)| {
                host_sockets
                    .into_iter()
                    .map(|h| (h, container_port.clone()))
                    .collect::<Vec<_>>() // meh
            })
            .map(|(h, c)| h.try_into().map(|s: SocketAddrV4| (s, c)))
            .collect::<Result<_, _>>()?;
        let env = value
            .env
            .into_iter()
            .map(|line| {
                if let Some((name, value)) = line.split_once('=') {
                    (name.to_string(), value.to_string())
                } else {
                    (line, "".to_string())
                }
            })
            .collect();
        Ok(ContainerDetails { ports, env })
    }
}

impl From<std::path::PathBuf> for ContainerEngine {
    fn from(value: std::path::PathBuf) -> Self {
        Self(value.into_os_string())
    }
}
