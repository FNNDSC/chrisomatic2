use std::{cell::RefCell, collections::HashSet, rc::Rc};

use chrisomatic_step::{Dependency, DependencyMap, PendingStep};

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

/// Get the value of [crate::step::Step::description] or the first item of [crate::step::Step::provides] corresponding to the given [PendingStep].
pub(crate) fn target_of(pending_step: impl AsRef<dyn PendingStep>) -> Dependency {
    provides_of(pending_step).head
}

/// Get the dependencies of a [PendingStep].
#[cfg(debug_assertions)]
pub(crate) fn dependencies_of(pending_step: impl AsRef<dyn PendingStep>) -> HashSet<Dependency> {
    let spy = DependencySpy::new();
    let _step = pending_step.as_ref().build(&spy).unwrap().unwrap();
    spy.into_inner()
}

/// Get the value of [Step::provides] of the step corresponding to the [PendingStep].
pub(crate) fn provides_of(
    pending_step: impl AsRef<dyn PendingStep>,
) -> nonempty::NonEmpty<Dependency> {
    let spy = DependencySpy::new();
    let step = pending_step.as_ref().build(&spy).unwrap().unwrap();
    step.provides()
}
