use std::{collections::HashMap, rc::Rc};

use chrisomatic_step::{Dependency, DependencyMap, Entry};

/// Map of values which steps may depend on.
///
/// NOTE: the values are "stringly typed", they might be strings, URLs,
/// integer IDs, ... This is a design trade-off to keep things simple.
pub(crate) struct DependencyHashMap(HashMap<Dependency, Rc<String>>);

impl DependencyHashMap {
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
}

impl DependencyMap for DependencyHashMap {
    fn get(&self, k: Dependency) -> Result<Rc<String>, Dependency> {
        self.0.get(&k).map(Rc::clone).ok_or(k)
    }

    fn contains_key(&self, k: &Dependency) -> bool {
        self.0.contains_key(k)
    }
}
