//! # Rusti Framework
//!
//! Un framework web inspiré de Django pour Rust, construit sur Axum.
//!
//! ## Exemple d'utilisation
//!
//! ```rust,no_run
//! use rusti::{RustiApp, Settings};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = Settings::default_values();
//!     let app = RustiApp::new(settings).await?;
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

pub mod app;
pub mod settings;
pub mod middleware;
pub mod response;
pub mod error;
pub mod orm;
pub mod processor;

pub use middleware::flash_message::FlashMessageSession;
pub use middleware::flash_message::flash_middleware;
pub use middleware::flash_message::{flash_error, flash_info, flash_success};
pub use processor::message_processor::Template;

#[cfg(feature = "orm")]
pub mod db;
#[macro_export]
macro_rules! impl_objects {
    ($entity:ty) => {
        impl $entity {
            /// Manager Django-like pour les queries
            pub const objects: $crate::orm::Objects<Self> = $crate::orm::Objects::new();
        }
    };
}
// Ré-exports publics pour faciliter l'utilisation
pub use app::RustiApp;
pub use settings::Settings;
pub use error::{ErrorContext, ErrorType};

// Ré-exports de types externes couramment utilisés
pub use axum;
pub use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    response::{IntoResponse, Html},
    extract::{State, Path, Query},
    http::StatusCode,
    Extension,
    response::Response,
    debug_handler,
};

// Ré-export de tower-sessions pour la gestion des sessions
pub use tower_sessions::Session;

pub use tera::{Tera, Context};
pub use serde::{Serialize, Deserialize};
pub use async_trait::async_trait;


#[cfg(feature = "orm")]
pub use sea_orm::{self, DatabaseConnection};

/// Macro pour faciliter la création de routes
///
/// # Exemple
/// ```rust,ignore
/// use rusti::routes;
///
/// let router = routes![
///     "/" => get(index),
///     "/about" => get(about),
///     "/user/:id" => get(user_detail),
/// ];
/// ```
#[macro_export]
macro_rules! routes {
    ($($path:expr => $handler:expr),* $(,)?) => {
        {
            let mut router = $crate::axum::Router::new();
            $(
                router = router.route($path, $handler);
            )*
            router
        }
    };
}

/// Version du framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");