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
// ! ## Import obligatoire des modules
pub mod app;
pub mod settings;
pub mod middleware;
pub mod response;
pub mod error;
pub mod orm;
pub mod processor;
pub mod macro_perso;
pub mod tera_function;
pub mod formulaire;

pub use serde_json::json;
#[cfg(feature = "orm")]
pub mod database;

// Middleware
pub use middleware::flash_message::flash_middleware;
pub use middleware::csrf::csrf_middleware;
pub use middleware::middleware_sanetiser::sanitize_middleware;
pub use processor::processor::{Message, Template};
pub use derive_form;

pub use macro_perso::router::{
    reverse,
    reverse_with_parameters,
    register_name_url::register_name_url,
};



// Modules et ré-exports liés à la base de données
pub use sea_orm;
pub use sea_orm::{
    DatabaseConnection,
    Database,
    EntityTrait,
    ModelTrait,
    ActiveModelTrait,
    ColumnTrait,
    QueryFilter,
    Set,
};
pub use database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

// Token csrf
pub use sha2::Sha256;
pub use hmac::{Hmac, Mac};


// Ré-exports publics pour faciliter l'utilisation
pub use app::RustiApp;
pub use settings::Settings;
pub use error::{ErrorContext, ErrorType};

// Ré-exports de types externes couramment utilisés
pub use axum;
pub use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    response::{IntoResponse, Html, Response, Redirect},
    extract::{State, Path, Query, Form},
    http::StatusCode,
    Extension,
    debug_handler,
    Form as AxumForm,
};
pub use once_cell::sync::Lazy;

// Ré-export de tower-sessions pour la gestion des sessions
pub use tower_sessions::Session;

pub use tera::{Tera, Context};
pub use serde::{Serialize, Deserialize};
pub use async_trait::async_trait;

pub use derive_form::rusti_form;
pub use derive_form::DeriveModelForm;

pub use formulaire::formsrusti::{Forms, FormulaireTrait};
pub use formulaire::extracteur::ExtractForm;
pub use macro_perso::context_macro::ContextHelper;


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


/// Version du framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");