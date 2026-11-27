//! App module — `RuniqueAppBuilder` constructor, final `RuniqueApp` application, build errors, and staging.
pub mod builder;
pub mod runique_app;
pub mod staging;

pub use builder::RuniqueAppBuilder;
pub use crate::errors::build::{BuildError, BuildFailure, CheckError, CheckReport};
pub use runique_app::RuniqueApp;
#[cfg(feature = "admin")]
pub use staging::AdminStaging;
pub use staging::{CoreStaging, MiddlewareStaging, StaticStaging};
