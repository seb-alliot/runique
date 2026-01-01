//! # Rusti Framework
//!
//! Un framework web inspiré de Django pour Rust, construit sur Axum.
//!
//! ## Exemple d'utilisation
//!
//! ```rust,no_run
//! use rusti::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = Settings::default_values();
//!     RustiApp::new(settings).await?.run().await?;
//!     Ok(())
//! }
//! ```

// Modules
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
pub mod utils;
#[cfg(feature = "orm")]
pub mod database;

// Réexports des crates externes (pour que les utilisateurs n'aient pas à les ajouter)
pub use tokio;
pub use serde;
pub use serde_json;
pub use tera;
pub use axum;
pub use tower_sessions;

// Middleware
pub use middleware::flash_message::flash_middleware;
pub use middleware::csrf::csrf_middleware;
pub use middleware::middleware_sanetiser::sanitize_middleware;
pub use middleware::error_handler::{
    render_404,
    render_500,
};
pub use middleware::csp::security_headers_middleware;
pub use middleware::csp::CspConfig;
pub use middleware::login_requiert::{
    login_required,
    redirect_if_authenticated,
};



pub use processor::processor::{Message, Template};
pub use derive_form;

pub use macro_perso::router::{
    reverse,
    reverse_with_parameters,
    register_name_url::register_name_url,
};

// Modules et ré-exports liés à la base de données
#[cfg(feature = "orm")]
pub use sea_orm;

#[cfg(feature = "orm")]
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

#[cfg(feature = "orm")]
pub use database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

// Token csrf
pub use sha2::Sha256;
pub use hmac::{Hmac, Mac};

// Ré-exports publics pour faciliter l'utilisation
pub use app::RustiApp;
pub use settings::Settings;
pub use error::{ErrorContext, ErrorType};

// Ré-exports de types Axum couramment utilisés
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

// Ré-export de tera
pub use tera::{Tera, Context};

// Ré-export de serde
pub use serde::{Serialize, Deserialize};
pub use serde_json::json;

pub use async_trait::async_trait;

// Macros de formulaire
pub use derive_form::rusti_form;
pub use derive_form::DeriveModelForm;

pub use formulaire::formsrusti::{Forms, RustiForm };
pub use formulaire::extracteur::ExtractForm;
pub use macro_perso::context_macro::ContextHelper;

/// Version du framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module prelude pour importer facilement tous les types couramment utilisés
///
/// # Exemple
///
/// ```rust,no_run
/// use rusti::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let settings = Settings::default_values();
///
///     RustiApp::new(settings).await?
///         .routes(Router::new().route("/", get(index)))
///         .run()
///         .await?;
///
///     Ok(())
/// }
///
/// async fn index() -> &'static str {
///     "Hello, Rusti!"
/// }
/// ```
pub mod prelude {
    //! Prelude pour importer tous les types couramment utilisés en une seule ligne

    // === Framework Rusti ===
    pub use crate::app::RustiApp;
    pub use crate::settings::{Settings, SettingsBuilder};
    pub use crate::processor::{Template, Message};

    // === Macros Rusti ===
    pub use crate::urlpatterns;
    pub use crate::context;

    // === Formulaires ===
    pub use crate::formulaire::extracteur::ExtractForm;
    pub use crate::formulaire::formsrusti::{Forms, FormulaireTrait};
    pub use crate::rusti_form;
    pub use crate::DeriveModelForm;

    // === Routing et reverse ===
    pub use crate::reverse;
    pub use crate::reverse_with_parameters;

    // === Axum - Router et Routing ===
    pub use axum::{
        Router,
        routing::{get, post, put, delete, patch},
    };

    // === Axum - Response ===
    pub use axum::response::{
        Response,
        IntoResponse,
        Html,
        Redirect,
    };

    // === Axum - Extractors ===
    pub use axum::extract::{
        Path,
        Query,
        Extension,
        Form,
        State,
    };

    // === Axum - HTTP ===
    pub use axum::http::StatusCode;
    pub use axum::Json;

    // === Tokio (IMPORTANT pour #[tokio::main]) ===
    pub use crate::tokio;

    // === Serde ===
    pub use crate::serde::{Serialize, Deserialize};
    pub use crate::serde_json::json;

    // === Tera ===
    pub use crate::tera::Context;

    // === Sessions ===
    pub use crate::tower_sessions::Session;

    // === ORM (si feature orm activée) ===
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self,
        DatabaseConnection,
        EntityTrait,
        ModelTrait,
        ActiveModelTrait,
        ColumnTrait,
        QueryFilter,
        QueryOrder,
        QuerySelect,
        Set,
    };

    #[cfg(feature = "orm")]
    pub use crate::database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

    #[cfg(feature = "orm")]
    pub use crate::orm::impl_objects;

    // === Async ===
    pub use async_trait;
}