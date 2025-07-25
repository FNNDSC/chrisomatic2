use std::rc::Rc;

use crate::{plugin_spec::PluginSpec, types::Username};

/// [Dependency] and value pair.
pub(crate) type Entry = (Dependency, String);

/// Dependency keys.
#[derive(Hash, Eq, PartialEq, Clone)]
pub(crate) enum Dependency {
    // UserId(Username),
    UserUrl(Username),
    UserGroupsUrl(Username),
    UserEmail(Username),
    AuthToken(Username),
    PluginUrl(PluginSpec),
    // PluginId(PluginSpec),
}

pub(crate) trait DependencyMap {
    /// Returns a [Rc] to the value corresponding to the key.
    fn get(&self, k: Dependency) -> Result<Rc<String>, Dependency>;

    /// Returns `true` if the map contains a value for the specified key.
    fn contains_key(&self, k: &Dependency) -> bool;
}
