use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::{
    dependency_map::{Dependency, DependencyMap},
    step::PendingStep,
};

/// A [DependencyMap] used to inspect [PendingStep] implementations.
pub(crate) struct DependencySpy(RefCell<HashSet<Dependency>>);

impl DependencyMap for DependencySpy {
    fn get(
        &self,
        k: crate::dependency_map::Dependency,
    ) -> Result<std::rc::Rc<String>, crate::dependency_map::Dependency> {
        let mut set = self.0.borrow_mut();
        set.insert(k);
        Ok(Rc::new("".to_string()))
    }

    fn contains_key(&self, _: &crate::dependency_map::Dependency) -> bool {
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

/// Get the value of [crate::step::Step::description] or the first item of [crate::step::Step::provides] corresponding to the given [PendingStep].
pub(crate) fn target_of(pending_step: impl AsRef<dyn PendingStep>) -> Dependency {
    let spy = DependencySpy::new();
    let step = pending_step.as_ref().build(&spy).unwrap().unwrap();
    if let Some(target) = step.description() {
        target
    } else {
        step.provides().head
    }
}

/// Get the dependencies of a [PendingStep].
pub(crate) fn dependencies_of(pending_step: impl AsRef<dyn PendingStep>) -> HashSet<Dependency> {
    let spy = DependencySpy::new();
    let _step = pending_step.as_ref().build(&spy).unwrap().unwrap();
    spy.into_inner()
}
