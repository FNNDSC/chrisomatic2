use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::types::{Dependency, DependencyMap, Step};

/// A [DependencyMap] used to inspect [PendingStep] implementations.
pub(crate) struct DependencySpy(RefCell<HashSet<Dependency>>);

impl DependencyMap for DependencySpy {
    fn get(&self, k: Dependency) -> Result<std::rc::Rc<String>, Dependency> {
        let mut set = self.0.borrow_mut();
        set.insert(k);
        Ok(Rc::new("".to_string()))
    }

    fn contains_key(&self, _: &Dependency) -> bool {
        false
    }
}

impl DependencySpy {
    pub(crate) fn new() -> Self {
        Self(RefCell::new(HashSet::new()))
    }

    pub(crate) fn into_inner(self) -> HashSet<Dependency> {
        self.0.into_inner()
    }
}

/// Get the dependencies of a [PendingStep].
pub(crate) fn dependencies_of(step: &impl Step) -> HashSet<Dependency> {
    let spy = DependencySpy::new();
    let _step = step.request(&spy).unwrap().unwrap();
    spy.into_inner()
}
