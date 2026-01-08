//! # Runique Framework
//!
//! Un framework web inspiré de Django pour Rust, construit sur Axum.
//!
//! ## Exemple d'utilisation
//!
//! ```rust,no_run
//! use runique::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = Settings::default_values();
//!     RuniqueApp::new(settings).await?.run().await?;
//!     Ok(())
//! }
//! ```

// Modules
pub mod app;
#[cfg(feature = "orm")]
pub mod database;
pub mod error;
pub mod formulaire;
pub mod macro_perso;
pub mod middleware;
pub mod orm;
pub mod processor;
pub mod response;
pub mod settings;
pub mod tera_function;
pub mod utils;

// Réexports des crates externes (pour que les utilisateurs n'aient pas à les ajouter)
pub use axum;
pub use serde;
pub use serde_json;
pub use tera;
pub use tokio;
pub use tower_sessions;

// Middleware
pub use middleware::csp::security_headers_middleware;
pub use middleware::csp::CspConfig;
pub use middleware::csrf::csrf_middleware;
pub use middleware::error_handler::{render_404, render_500};
pub use middleware::flash_message::flash_middleware;
pub use middleware::login_requiert::{login_required, redirect_if_authenticated};
pub use middleware::middleware_sanetiser::sanitize_middleware;

pub use derive_form;
pub use processor::{Message, Template};

pub use macro_perso::router::{
    register_name_url::register_name_url, reverse, reverse_with_parameters,
};

// Modules et ré-exports liés à la base de données
#[cfg(feature = "orm")]
pub use sea_orm;

#[cfg(feature = "orm")]
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};

#[cfg(feature = "orm")]
pub use database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

// Token csrf
pub use hmac::{Hmac, Mac};
pub use sha2::Sha256;

// Ré-exports publics pour faciliter l'utilisation
pub use app::RuniqueApp;
pub use error::{ErrorContext, ErrorType};
pub use settings::Settings;

// Ré-exports de types Axum couramment utilisés
pub use axum::{
    debug_handler,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post, put},
    Extension, Form as AxumForm, Router,
};

pub use once_cell::sync::Lazy;

// Ré-export de tera
pub use tera::{Context, Tera};

// Ré-export de serde
pub use serde::ser::{SerializeStruct, Serializer};
pub use serde::{Deserialize, Serialize};
pub use serde_json::json;

pub use async_trait::async_trait;

// Macros de formulaire
pub use derive_form::runique_form;
pub use derive_form::DeriveModelForm;

pub use formulaire::extracteur::ExtractForm;
pub use formulaire::formsrunique::{Forms, RuniqueForm};
pub use macro_perso::context_macro::ContextHelper;

/// Version du framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module prelude pour importer facilement tous les types couramment utilisés
///
/// # Exemple
///
/// ```rust,no_run
/// # use axum::{Router, routing::get};
/// # use runique::prelude::*;
/// # async fn index() -> &'static str { "Hello, Runique!" }
/// # async fn doc_test() -> Result<(), Box<dyn std::error::Error>> {
/// use runique::prelude::*;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let settings = Settings::default_values();
///
///     RuniqueApp::new(settings).await?
///         .routes(Router::new().route("/", get(index)))
///         .run()
///         .await?;
///
///     Ok(())
/// }
/// # Ok(())
/// # }
/// ```
pub mod prelude {
    // === Framework Runique Core ===
    pub use crate::app::RuniqueApp;
    pub use crate::processor::{Message, Template};
    pub use crate::settings::{Settings, SettingsBuilder};

    // === Macros Runique ===
    pub use crate::context;
    pub use crate::urlpatterns;
    pub use crate::{error, flash_now, info, success, warning};

    // === Formulaires ===
    pub use crate::formulaire::extracteur::ExtractForm;
    pub use crate::formulaire::field::*;
    pub use crate::formulaire::formsrunique::{Forms, RuniqueForm};
    pub use crate::runique_form;
    pub use crate::tera;
    pub use crate::DeriveModelForm;

    // === Routing et URL reversing ===
    pub use crate::reverse;
    pub use crate::reverse_with_parameters;

    // === Axum - Router et Routing ===
    pub use axum::{
        routing::{delete, get, patch, post, put},
        Router,
    };

    // === Axum - Response ===
    pub use axum::response::{Html, IntoResponse, Redirect, Response};

    // === Axum - Extractors ===
    pub use axum::extract::{Extension, Form, Path, Query, State};
    pub use axum::Json;

    // === Axum - HTTP ===
    pub use axum::http::StatusCode;

    // === Tokio ===
    pub use crate::tokio;

    // === Async ===
    pub use crate::async_trait;

    // === Serde ===
    pub use crate::serde::ser::{SerializeStruct, Serializer};
    pub use crate::serde::{Deserialize, Serialize};

    // === Collections ===
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Arc;

    // === Tera ===
    pub use crate::tera::Context;

    // === Sessions ===
    pub use crate::tower_sessions::Session;

    // === Middleware courants ===
    pub use crate::middleware::csp::CspConfig;
    pub use crate::middleware::login_requiert::{login_required, redirect_if_authenticated};

    // === ORM (si feature orm activée) ===
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
        QueryFilter, QueryOrder, QuerySelect, Set,
    };

    #[cfg(feature = "orm")]
    pub use crate::database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

    #[cfg(feature = "orm")]
    pub use crate::orm::impl_objects;
}
