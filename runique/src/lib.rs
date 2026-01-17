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

// Modules internes
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

// Ré-exports généraux
pub use axum;
pub use serde;
pub use serde_json;
pub use tera;
pub use tokio;
pub use tower_sessions;

// Middleware
pub use middleware::csp::{security_headers_middleware, CspConfig};
pub use middleware::csrf::csrf_middleware;
pub use middleware::error_handler::{render_404, render_500};
pub use middleware::flash_message::flash_middleware;
pub use middleware::login_requiert::{login_required, redirect_if_authenticated};
pub use middleware::middleware_sanetiser::sanitize_middleware;

// Macros
pub use derive_form::{runique_form, DeriveModelForm};

// Processor
pub use processor::{Message, Template};

// Routing & URL reversing
pub use macro_perso::router::{
    register_name_url::register_name_url, reverse, reverse_with_parameters,
};

// ORM (SeaORM)
#[cfg(feature = "orm")]
pub use database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};
#[cfg(feature = "orm")]
pub use sea_orm;
#[cfg(feature = "orm")]
pub use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter, Set,
};

// CSRF / HMAC
pub use hmac::{Hmac, Mac};
pub use sha2::Sha256;

// Ré-exports principaux pour usage courant
pub use app::RuniqueApp;
pub use error::{ErrorContext, ErrorType};
pub use settings::Settings;

// Axum - types courants
pub use axum::{
    debug_handler,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post, put},
    Extension, Form as AxumForm, Router,
};

// Divers utilitaires
pub use async_trait::async_trait;
pub use once_cell::sync::Lazy;
pub use serde::ser::{SerializeStruct, Serializer};
pub use serde::{Deserialize, Serialize};
pub use serde_json::json;
pub use tera::{Context, Tera};

// Formulaires
pub use formulaire::builder_form::form_manager::Forms;
pub use formulaire::utils::extracteur::ExtractForm;
pub use macro_perso::context_macro::ContextHelper;

// Version du framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// === Prelude simplifié et sûr ===
pub mod prelude {
    // === Core ===
    pub use crate::app::RuniqueApp;
    pub use crate::processor::{Message, Template};
    pub use crate::settings::Settings;

    // === Macros ===
    pub use crate::{runique_form, DeriveModelForm};

    // === Formulaires ===
    pub use crate::formulaire::builder_form::form_manager::Forms;
    pub use crate::formulaire::utils::extracteur::ExtractForm;
    pub use country::Country;
    pub use phonenumber::{country::Id as CountryId, parse, Mode};

    // === Champs standards ===

    // Core logic
    pub use crate::formulaire::builder_form::base_struct::*;
    pub use crate::formulaire::builder_form::option_field::*;
    pub use crate::formulaire::builder_form::trait_form::{FormField, RuniqueForm};

    // Champs mis à jour
    pub use crate::formulaire::builder_form::field_type::number_mode::NumericField;
    pub use crate::formulaire::builder_form::field_type::text_mode::TextField;
    pub use crate::formulaire::builder_form::generique_field::GenericField;

    // === Messages flash ===
    pub use crate::{context, error, flash_now, info, success, warning};

    // === Routing et URL reversing ===
    pub use crate::{reverse, reverse_with_parameters};

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

    // === Tokio / Async ===
    pub use async_trait;
    pub use tokio;

    // === Serde ===
    pub use crate::serde::ser::{SerializeStruct, Serializer};
    pub use crate::serde::{Deserialize, Serialize};

    // === Collections / Sync ===
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Arc;

    // === Tera ===
    pub use crate::tera;
    pub use crate::tera::{Context, Tera};

    // === Sessions ===
    pub use crate::tower_sessions::Session;

    // === Middleware courants ===
    pub use crate::middleware::csp::CspConfig;
    pub use crate::middleware::login_requiert::{login_required, redirect_if_authenticated};

    // === SeaORM ===
    pub use crate::sea_orm;

    // === ORM (si feature orm activée) ===
    #[cfg(feature = "orm")]
    pub use crate::database::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};
    #[cfg(feature = "orm")]
    pub use crate::orm::impl_objects;
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
        QueryOrder, QuerySelect, Set,
    };
}
