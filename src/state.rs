use std::{collections::HashMap, rc::Rc};

use crate::{plugin_spec::PluginSpec, types::Username};

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

/// Map of values which steps may depend on.
///
/// NOTE: the values are "stringly typed", they might be strings, URLs,
/// integer IDs, ... This is a design trade-off to keep things simple.
pub(crate) struct DependencyMap(HashMap<Dependency, Rc<String>>);

/// [Dependency] and value pair.
pub(crate) type Entry = (Dependency, String);

impl DependencyMap {
    /// Creates an empty [DependencyMap] with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    /// Insert multiple values.
    pub fn insert_all(&mut self, entries: impl IntoIterator<Item = Entry>) {
        for (k, v) in entries {
            self.insert(k, v)
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, k: Dependency, v: String) {
        self.0.insert(k, Rc::new(v));
    }

    /// Returns a [Rc] to the value corresponding to the key.
    pub fn get(&self, k: Dependency) -> Result<Rc<String>, Dependency> {
        self.0.get(&k).map(Rc::clone).ok_or(k)
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, k: &Dependency) -> bool {
        self.0.contains_key(k)
    }
}
