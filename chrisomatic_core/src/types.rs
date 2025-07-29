use chrisomatic_spec::{ComputeResourceName, PluginSpec, Username};

/// Human-readable API resource identifier.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Resource {
    User(Username),
    Plugin(PluginSpec),
}

/// [Dependency] and value pair.
pub type Entry = (Dependency, String);

/// Multiple [Entry].
pub type Entries = Vec<Entry>;

/// Identifiers of data which a [Step] might depend on.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Dependency {
    UserUrl(Username),
    UserGroupsUrl(Username),
    UserEmail(Username),
    AuthToken(Username),
    /// CSV of all compute resource names
    ComputeResourceAll,
    /// URL of a compute resource
    ComputeResourceUrl(ComputeResourceName),

    /// URL of plugin in targeted _CUBE_.
    PluginUrl(PluginSpec),
    /// URL of plugin in peer _CUBE_.
    PluginPeerUrl(PluginSpec),
    /// Version of plugin in _CUBE_.
    PluginVersion(PluginSpec),
    /// A placeholder key which indicates dependency on an admin user account.
    AdminCredentials,
    /// A placeholder key which indicates dependency on a peer _CUBE_ URL.
    PeerUrl,
}

/// Map data structure where keys are [Dependency] and values are reference-counted [String].
pub trait DependencyMap {
    /// Returns a [Rc] to the value corresponding to the key.
    ///
    /// Calling [DependencyMap::get] from [crate::step::PendingStep::build]
    /// implies the step has a strict dependency on `k`.
    fn get(&self, k: Dependency) -> Result<std::rc::Rc<String>, Dependency>;

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// Calling [DependencyMap::contains_key] from [crate::step::PendingStep::build]
    /// implies that the step may be redundant if `true` is returned.
    /// More specifically, the code should look something like:
    ///
    /// ```
    /// use chrisomatic_spec::Username;
    /// use chrisomatic_core::types::{Dependency, DependencyMap, Step};
    ///
    ///
    /// fn build(map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency> {
    ///     let username: Username = todo!();  // self.username.clone()
    ///     if map.contains_key(&Dependency::UserUrl(username)) {
    ///         // if true...
    ///         return Ok(None); // step is redundant, skip it
    ///     }
    ///     todo!()
    /// }
    ///
    ///     
    ///
    /// ```
    fn contains_key(&self, k: &Dependency) -> bool;
}

/// Error executing a [Step].
#[derive(thiserror::Error, Debug)]
pub enum StepError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("HTTP status code {status} from {method} {url}")]
    Status {
        status: reqwest::StatusCode,
        method: reqwest::Method,
        url: reqwest::Url,
    },
    #[error(transparent)]
    Deserialize(#[from] serde_json::Error),
}

/// Kinds of effects which can happen to the API state after trying to do the
/// request of a [Step].
#[derive(Debug)]
pub enum StepEffect {
    /// A resource was created.
    Created,
    /// A resource was found.
    Unmodified,
    /// A resource was modified.
    Modified,
    /// The step was not performed because of an unfulfilled dependency.
    Unfulfilled(Dependency),
    /// The step produced an error.
    Error(StepError),
}

/// Possible effects a successful request from a [Step].
#[derive(Clone, Copy, Debug)]
pub enum EffectKind {
    /// The resource was created.
    Created,
    /// The resource exists and is correct.
    Unmodified,
    /// The resource exists and was modified.
    Modified,
}

impl From<EffectKind> for StepEffect {
    fn from(value: EffectKind) -> Self {
        match value {
            EffectKind::Created => StepEffect::Created,
            EffectKind::Unmodified => StepEffect::Unmodified,
            EffectKind::Modified => StepEffect::Modified,
        }
    }
}

/// Outcome of running a [Step].
#[derive(Debug)]
pub struct Outcome {
    /// Affected API resource.
    pub target: Resource,
    /// Step effect.
    pub effect: StepEffect,
}

impl Outcome {
    /// Returns `true` if the effect is OK.
    pub fn ok(&self) -> bool {
        matches!(
            &self.effect,
            StepEffect::Created | StepEffect::Unmodified | StepEffect::Modified
        )
    }
}
/// A `Step` produces a [reqwest::Request] to affect an API resource. E.g. it
/// might _search_ for whether the specified resource already exists, _modify_
/// an existing resource to match the specified details, or _create_ a new
/// API resource.
///
/// The implementation of step execution _should_:
///
/// 1. Call [Step::request] to produce an HTTP request
/// 2. Send the HTTP request and call [Step::check_status].
/// 3. If status is OK, then call [Step::deserialize].
pub trait Step {
    /// Create the HTTP request.
    ///
    /// - [Err] indicates the step has an unfulfilled dependency.
    /// - [None] indicates the request is redundant.
    /// - [Some] indicates the request is necessary.
    fn request(&self, map: &dyn DependencyMap) -> Result<Option<reqwest::Request>, Dependency>;

    /// Indicate what API resource is affected by this step.
    fn affects(&self) -> Resource;

    /// Indicate how the API resource is affected by this step if successful.
    fn effect(&self) -> EffectKind;

    /// Check whether the HTTP response is OK.
    fn check_status(&self, status: reqwest::StatusCode) -> bool {
        status.is_success()
    }

    /// Deserialize the HTTP response body to the request of [Step::search] and decide what to do next.
    fn deserialize(&self, body: bytes::Bytes) -> serde_json::Result<Entries>;

    /// Indicate what keys which are guaranteed to be contained in the [Ok]
    /// return of [Step::deserialize].
    fn provides(&self) -> Vec<Dependency>;
}
