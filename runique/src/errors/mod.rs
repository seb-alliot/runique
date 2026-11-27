//! Framework errors, one file per subject: `BuildError` (the start), `RuniqueError` (the cause),
//! `AppError` (the response), `ErrorContext` (what the page shows), and what the debug page reads.
pub mod app_error;
pub mod build;
pub mod context;
pub mod debug_page;
pub mod error;
pub use app_error::*;
pub use build::*;
pub use context::*;
pub use debug_page::*;
pub use error::*;
