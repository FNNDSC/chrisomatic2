// use serde::de::DeserializeOwned;

// use crate::{
//     Manifest,
//     dependency_tree::DependencyTree,
//     step::{Dependency, Step},
// };

// pub(crate) trait DeserializeOwnedSize: DeserializeOwned + Sized {}

// pub(crate) type DynDeserialize = Box<dyn DeserializeOwned>;

// pub(crate) type DynStep = Box<dyn Step<Dep = Box<dyn Dependency>, Out = Box<dyn Dependency>>>;

// pub(crate) fn plan(manifest: &Manifest) -> DependencyTree<DynStep> {
//     todo!()
// }
