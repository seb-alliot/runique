//! # Runique Framework
//!
//! Framework web moderne basé sur Axum pour la création d'applications web robustes et sécurisées.
//!
//! Organisation des modules alignée sur la hiérarchie `src/` actuelle tout en
//! conservant des alias de compatibilité (`config_runique`, `formulaire`,
//! middleware, etc.).
//!
//! ## Modules principaux
//!
//! - **`app`** : Constructeur et gestionnaire d'application
//! - **`config`** : Configuration (serveur, sécurité, settings, fichiers statiques)
//! - **`context`** : Contexte de requête, gestion d'erreurs, moteurs de template
//! - **`engine`** : Moteur principal Runique
//! - **`flash`** : Gestion des messages flash
//! - **`forms`** : Système de formulaires et validation
//! - **`macros`** : Macros utilitaires pour routes, contexte, flash, etc.
//! - **`middleware`** : Middlewares de sécurité (CSRF, CSP, authentification, cache, etc.)
//! - **`utils`** : Utilitaires (CSRF, CSP nonce, parsing HTML, response helpers)
//! - **`db`** : Configuration ORM (optionnel, avec feature `orm`)
//!
//! ## Exemple rapide
//!
//! ```rust,no_run
//! use runique::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = RuniqueConfig::from_env();
//!     let app = RuniqueApp::builder(config).build().await.unwrap();
//!     app.run().await.unwrap();
//! }
//! ```

// ---------------------------------------------------------------------------
// Modules principaux (arborescence actuelle)
// ---------------------------------------------------------------------------
pub mod app;
pub mod config;
pub mod context;
#[cfg(feature = "orm")]
pub mod db;
pub mod engine;
pub mod flash;
pub mod forms;
#[macro_use]
pub mod macros;
pub mod middleware;
pub mod utils;

// ---------------------------------------------------------------------------
// Alias de compatibilité pour l’ancien nommage
// ---------------------------------------------------------------------------
pub mod config_runique {
    pub mod composant_config {
        pub mod security_struct {
            pub use crate::config::security::*;
        }
        pub mod server_struct {
            pub use crate::config::server::*;
        }
        pub mod settings_struct {
            pub use crate::config::settings::*;
        }
        pub mod static_struct {
            pub use crate::config::static_files::*;
        }
    }

    pub mod config_struct {
        pub use crate::config::app::RuniqueConfig;
    }

    pub use composant_config::*;
    pub use config_struct::RuniqueConfig;
}

#[cfg(feature = "orm")]
pub mod data_base_runique {
    pub mod config {
        pub use crate::db::config::*;
    }

    pub mod composant_data_base {
        pub use crate::db::objects::*;
        pub use crate::db::query::*;
    }

    pub use composant_data_base::*;
    pub use config::*;
}

pub mod formulaire {
    pub mod builder_form {
        pub mod base_struct {
            pub use crate::forms::base::*;
        }
        pub mod field_type {
            pub use crate::forms::fields::*;
        }
        pub mod generique_field {
            pub use crate::forms::generic::*;
        }
        pub mod formmanager {
            pub use crate::forms::manager::*;
        }
        pub mod option_field {
            pub use crate::forms::options::*;
        }
        pub mod trait_form {
            pub use crate::forms::field::*;
        }
    }

    pub mod utils {
        pub use crate::forms::utils::*;
    }

    pub use builder_form::*;
    pub use utils::*;
}

pub mod gardefou {
    pub mod composant_middleware {
        pub use crate::middleware::allowed_hosts::*;
        pub use crate::middleware::auth::*;
        pub use crate::middleware::cache::*;
        pub use crate::middleware::config::*;
        pub use crate::middleware::config_middleware::*;
        pub use crate::middleware::csp::*;
        pub use crate::middleware::csrf::*;
        pub use crate::middleware::error::*;
        pub use crate::middleware::sanitizer::*;
    }

    pub use composant_middleware::*;
}

pub mod macro_runique {
    pub use crate::macros::*;
    pub use crate::macros::{context, flash, router};
}

pub mod moteur_engine {
    pub mod engine_struct {
        pub use crate::engine::core::*;
    }

    pub use engine_struct::*;
}

pub mod request_context {
    pub mod context_error {
        pub use crate::context::error::*;
    }
    pub mod template_context {
        pub use crate::context::template::*;
    }
    pub mod tera_tool {
        pub use crate::context::tera::*;
    }
    pub mod composant_request {
        pub use crate::context::request::*;
    }

    pub use composant_request::*;
    pub use context_error::*;
    pub use template_context::*;
    pub use tera_tool::*;
}

pub mod runique_start {
    pub mod composant_app {
        pub mod builder_util {
            pub use crate::app::builder::*;
        }
        pub mod template_engine {
            pub use crate::app::templates::*;
        }
    }

    pub use crate::app::*;
}

// ---------------------------------------------------------------------------
// Ré-export des dépendances principales
// ---------------------------------------------------------------------------
pub use anyhow;
pub use argon2;
pub use async_trait;
pub use axum;
pub use chrono;
pub use hmac;
pub use once_cell;
pub use regex;
pub use serde;
pub use serde_json;
pub use sha2;
pub use tera;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tower_sessions;
pub use uuid;

#[cfg(feature = "orm")]
pub use sea_orm;

pub use derive_form::DeriveModelForm;

// ---------------------------------------------------------------------------
// Prelude simplifié
// ---------------------------------------------------------------------------
pub mod prelude {
    // ========================================================================
    // MODULES PRINCIPAUX
    // ========================================================================
    pub use crate::app::{RuniqueApp, RuniqueAppBuilder};
    pub use crate::config::app::RuniqueConfig;
    pub use crate::context::{AppError, AppResult, TemplateContext};
    pub use crate::engine::RuniqueEngine;
    pub use crate::flash::{FlashMessage, Message, MessageLevel};
    pub use crate::forms::{
        field::{FormField, RuniqueForm},
        fields::{
            boolean::BooleanField,
            choice::ChoiceField,
            datetime::DateTimeField,
            file::FileField,
            hidden::HiddenField,
            number::NumericField,
            special::{ColorField, JSONField, SlugField},
            text::TextField,
        },
        generic::{FieldKind, GenericField},
        manager::Forms,
    };
    pub use derive_form::DeriveModelForm;

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    pub use crate::config::{security::SecurityConfig, server::ServerConfig};

    // ========================================================================
    // UTILS
    // ========================================================================
    pub use crate::utils::csp_nonce::*;
    pub use crate::utils::csrf::*;
    pub use crate::utils::parse_html::*;
    pub use crate::utils::response_helpers::*;

    // ========================================================================
    // CONTEXTE & TEMPLATE
    // ========================================================================
    pub use crate::context::error::*;
    pub use crate::context::request::{RuniqueContext, TemplateEngine};
    pub use crate::context::tera::*;

    // ========================================================================
    // MIDDLEWARE
    // ========================================================================
    pub use crate::middleware::{
        allowed_hosts::*, auth::*, cache::*, config::*, csp::*, csrf::*, error::*, sanitizer::*,
    };

    // ========================================================================
    // AXUM & HTTP
    // ========================================================================
    pub use axum::{
        extract::{Extension, Form, FromRequestParts, Path, Query, State},
        http::{HeaderMap, HeaderValue, Method, StatusCode},
        middleware,
        response::{Html, IntoResponse, Redirect, Response},
        routing::{any, delete, get, patch, post, put},
        Json, Router,
    };

    // ========================================================================
    // ORM (optionnel)
    // ========================================================================
    #[cfg(feature = "orm")]
    pub use crate::db::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self, entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ColumnTrait,
        ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait, NotSet,
        QueryFilter, QueryOrder, QuerySelect, Set,
    };

    // ========================================================================
    // SÉRIALISATION & DONNÉES
    // ========================================================================
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use serde_json::{from_str, json, to_string, Value};

    // ========================================================================
    // TEMPLATE ENGINE
    // ========================================================================
    pub use tera::{Context, Tera};

    // ========================================================================
    // ASYNC & TOKIO
    // ========================================================================
    pub use async_trait::async_trait;
    pub use once_cell::sync::Lazy;
    pub use tokio;

    // ========================================================================
    // TYPES STANDARDS COURANTS
    // ========================================================================
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Arc;

    // ========================================================================
    // SÉCURITÉ - HMAC, Hashing, etc.
    // ========================================================================
    pub use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    };
    pub use hmac::{Hmac, Mac};
    pub use sha2::Sha256;

    // ========================================================================
    // SESSIONS & DATES
    // ========================================================================
    pub use anyhow::{Context as AnyhowContext, Error, Result};
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
    pub use regex::Regex;
    pub use tower_sessions::{Session, SessionManagerLayer};
    pub use uuid::Uuid;
}
