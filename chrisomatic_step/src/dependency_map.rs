use std::rc::Rc;

use chrisomatic_spec::{PluginSpec, Username};

/// [Dependency] and value pair.
pub type Entry = (Dependency, String);

/// Dependency keys.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum Dependency {
    /// A placeholder key which, if present, guarantees that the user exists.
    UserExists(Username),
    UserUrl(Username),
    UserGroupsUrl(Username),
    UserEmail(Username),
    AuthToken(Username),
    PluginUrl(PluginSpec),
}

pub trait DependencyMap {
    /// Returns a [Rc] to the value corresponding to the key.
    ///
    /// Calling [DependencyMap::get] from [crate::step::PendingStep::build]
    /// implies the step has a strict dependency on `k`.
    fn get(&self, k: Dependency) -> Result<Rc<String>, Dependency>;

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// Calling [DependencyMap::contains_key] from [crate::step::PendingStep::build]
    /// implies that the step may be redundant if `true` is returned.
    /// More specifically, the code should look something like:
    ///
    /// ```
    /// use chrisomatic_spec::Username;
    /// use chrisomatic_step::{Dependency, DependencyMap, PendingStep, PendingStepResult};
    ///
    /// struct Example {
    ///     username: Username
    /// };
    ///
    /// impl PendingStep for Example {
    ///     fn build(&self, map: &dyn DependencyMap) -> PendingStepResult {
    ///         if map.contains_key(&Dependency::UserUrl(self.username.clone())) {
    ///             // if true...
    ///             return Ok(None); // step is redundant, skip it
    ///         }
    ///         todo!()
    ///     }
    /// }
    /// ```
    fn contains_key(&self, k: &Dependency) -> bool;
}
