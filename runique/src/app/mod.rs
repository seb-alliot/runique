//! Module app — constructeur `RuniqueAppBuilder`, application finale `RuniqueApp`, erreurs de build et staging.
pub mod builder;
pub mod error_build;
pub mod runique_app;
pub mod staging;
pub mod templates;

pub use builder::RuniqueAppBuilder;
pub use error_build::{BuildError, BuildErrorKind, CheckError, CheckReport};
pub use runique_app::RuniqueApp;
pub use staging::{AdminStaging, CoreStaging, CspConfig, MiddlewareStaging, StaticStaging};
pub use templates::*;
