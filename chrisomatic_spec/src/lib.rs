//! _chrisomatic_ manifest schema definition. See [GivenManifest].
mod canonicalize;
mod plugin_spec;
mod share_target;
mod spec;
mod types;

pub use canonicalize::*;
pub use plugin_spec::PluginSpec;
pub use spec::*;
pub use types::*;
