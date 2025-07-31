//! Concrete implementations of [`Step`].
//!
//! ### Notes
//!
//! - Not many docstrings. In theory, implementations of [`Step`]
//!   are "self-documenting" by the method bodies of
//!   [`Step::affects`], [`Step::effects`], and [`Step::provides`].
//! - Naming convention: Noun before verb (like French)
//!
//! [`Step`]: crate::types::Step
//! [`Step::affects`]: crate::types::Step::affects
//! [`Step::effects`]: crate::types::Step::effects
//! [`Step::provides`]: crate::types::Step::provides

// mod plugin;
mod user;
// mod feed;

// pub(crate) use plugin::*;
pub(crate) use user::*;
// pub(crate) use feed::*;
