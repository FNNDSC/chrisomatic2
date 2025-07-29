use std::rc::Rc;

use chris_oag::models;
use chrisomatic_spec::{ComputeResourceName, CubeUrl, PluginSpec, UserCredentials};
use chrisomatic_step::{
    Dependency, EffectKind, Entries, PendingStep, PendingStepResult, Step, StepRequest,
};
use nonempty::{NonEmpty, nonempty};
use reqwest::{Method, Request};

use crate::{extra_models::NonEmptyPage, request_builder::RequestBuilder};

/// A [PendingStep] for [PluginFindInLocal].
#[derive(Clone, Debug)]
pub(crate) struct PluginFindInLocal {
    url: CubeUrl,
    plugin: PluginSpec,
}

impl PendingStep for PluginFindInLocal {
    fn build(&self, map: &dyn chrisomatic_step::DependencyMap) -> PendingStepResult {
        debug_assert!(
            !map.contains_key(&Dependency::Plugin(self.plugin.clone())),
            "Duplicate step for {:?}",
            &self.plugin
        );
        Ok(Some(Rc::new(PluginFindInLocalStep(self.clone()))))
    }
}

/// A [Step] to find the plugin in the local _CUBE_.
pub(crate) struct PluginFindInLocalStep(PluginFindInLocal);

impl Step for PluginFindInLocalStep {
    fn request(&self) -> reqwest::Request {
        search_plugin(&self.0.url, &self.0.plugin)
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<EffectKind> {
        let page: models::PaginatedPluginList = serde_json::from_slice(&body)?;
        let placeholder = (Dependency::Plugin(self.0.plugin.clone()), "".to_string());
        let checked = (
            Dependency::PluginCheckedInLocal(self.0.plugin.clone()),
            "".to_string(),
        );
        let outputs = if let Some(plugin) = page.results {
            let spec = PluginSpec::new(&plugin.name, &plugin.version);
            vec![
                placeholder,
                checked,
                (Dependency::PluginUrl(spec.clone()), plugin.url),
                (Dependency::PluginVersion(spec), plugin.version),
            ]
        } else {
            vec![placeholder, checked]
        };
        Ok(EffectKind::Unmodified(outputs))
    }

    fn provides(&self) -> NonEmpty<Dependency> {
        nonempty![
            Dependency::Plugin(self.0.plugin.clone()),
            Dependency::PluginCheckedInLocal(self.0.plugin.clone())
        ]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PluginFindInPeer {
    url: Option<CubeUrl>,
    plugin: PluginSpec,
}

impl PendingStep for PluginFindInPeer {
    fn build(&self, map: &dyn chrisomatic_step::DependencyMap) -> PendingStepResult {
        debug_assert!(
            map.get(Dependency::PluginCheckedInLocal(self.plugin.clone()))
                .is_ok()
        );
        if map.contains_key(&Dependency::PluginUrl(self.plugin.clone())) {
            return Ok(None);
        }
        if let Some(url) = self.url {
            Ok(Some(Rc::new(PluginFindInPeerStep(self.clone()))))
        } else {
            Err(Dependency::PeerUrl)
        }
    }
}

pub(crate) struct PluginFindInPeerStep(PluginFindInPeer);

impl Step for PluginFindInPeerStep {
    fn request(&self) -> reqwest::Request {
        search_plugin(&self.0.url, &self.0.plugin)
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<EffectKind> {
        let page: models::PaginatedPluginList = serde_json::from_slice(body.as_ref())?;
        if let Some(plugin) = page.results.into_iter().next() {
            let spec = PluginSpec::new(&plugin.name, &plugin.version);
            let outputs = vec![
                (Dependency::Plugin(spec.clone()), "".to_string()),
                (Dependency::PluginPeerUrl(spec), plugin.url),
            ];
            Ok(EffectKind::Unmodified(outputs))
        } else {
            Ok(EffectKind::DoesNotExist)
        }
    }

    fn provides(&self) -> NonEmpty<Dependency> {
        nonempty![
            Dependency::Plugin(self.0.plugin.clone()),
            Dependency::PluginPeerUrl(self.0.plugin.clone())
        ]
    }
}

fn search_plugin(url: &CubeUrl, plugin: &PluginSpec) -> Request {
    let mut url = url.to_url().join("plugins/").unwrap();
    url.set_query(Some(&query_of(&plugin)));
    Request::new(Method::GET, url).accept_json()
}

/// A [PendingStep] which adds a plugin to _CUBE_ from a peer.
#[derive(Clone, Debug)]
pub(crate) struct PluginAddFromPeer {
    url: CubeUrl,
    plugin: PluginSpec,
    admin: Option<Rc<UserCredentials>>,
    compute_names: Vec<ComputeResourceName>,
}

impl PendingStep for PluginAddFromPeer {
    fn build(&self, map: &dyn DependencyMap) -> PendingStepResult {
        debug_assert!(
            map.get(Dependency::PluginCheckedInLocal(self.plugin.clone()))
                .is_ok()
        );
        if map.contains_key(&Dependency::PluginUrl(self.plugin.clone())) {
            Ok(None)
        } else if let Some(admin) = &self.admin {
            // If the compute_resources of a plugin are not specified,
            // then register it to all compute resources.
            let compute_names_csv = if self.compute_names.is_empty() {
                map.get(Dependency::ComputeResourceAll)?.to_string()
            } else {
                self.compute_names
                    .iter()
                    .map(|n| n.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            };
            Ok(Some(Rc::new(PluginAddFromPeerStep {
                url: self.url.clone(),
                admin: Rc::clone(admin),
                plugin_peer_url: map.get(Dependency::PluginPeerUrl(self.plugin.clone()))?,
                compute_names_csv,
                plugin: self.plugin.clone(),
            })))
        } else {
            Err(Dependency::ComputeResourceAll)
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PluginAddFromPeerStep {
    url: CubeUrl,
    plugin_peer_url: Rc<String>,
    plugin: PluginSpec,
    admin: Rc<UserCredentials>,
    compute_names_csv: String,
}

impl Step for PluginAddFromPeerStep {
    fn request(&self) -> reqwest::Request {
        let body = models::PluginAdminRequest {
            compute_names: self.compute_names_csv.clone(),
            plugin_store_url: Some(self.plugin_peer_url.to_string()),
            ..Default::default()
        };
        let mut url = self.url.to_url();
        url.set_path(&url.path().replace("/api/v1/", "/chris-admin/api/v1/"));
        Request::new(Method::POST, url)
            .auth(&self.admin)
            .json(&body)
            .unwrap()
            .accept_json()
    }

    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<EffectKind> {
        let plugin: models::Plugin = serde_json::from_slice(&body)?;
        let spec = PluginSpec::new(&plugin.name, &plugin.version);
        debug_assert_eq!(&spec, &self.plugin);
        let outputs = vec![
            (Dependency::Plugin(spec.clone()), plugin.id.to_string()),
            (Dependency::PluginUrl(spec.clone()), plugin.url),
            (Dependency::PluginVersion(spec), plugin.version),
        ];
        Ok(EffectKind::Created(outputs))
    }

    fn provides(&self) -> NonEmpty<chrisomatic_step::Dependency> {
        nonempty![
            Dependency::Plugin(self.plugin.clone()),
            Dependency::PluginUrl(self.plugin.clone()),
            Dependency::PluginVersion(self.plugin.clone()),
        ]
    }
}

fn query_of(plugin: &PluginSpec) -> String {
    if let Some(version) = &plugin.version {
        format!("name_exact={}&version={version}&limit=1", plugin.name)
    } else {
        format!("name_exact={}&limit=1", plugin.name)
    }
}
