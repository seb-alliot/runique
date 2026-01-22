//! # Runique Framework
//!
//! Un framework web inspiré de Django pour Rust, avec ORM, templates, et sécurité intégrée.
//!
//! ## Utilisation de base
//!
//! ```rust,no_run
//! use runique::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = RuniqueConfig::from_env().unwrap();
//!     let app = RuniqueApp::new(config)
//!         .with_routes(Router::new().route("/", get(index)))
//!         .build()
//!         .await
//!         .unwrap();
//!
//!     app.run("0.0.0.0:3000").await.unwrap();
//! }
//!
//! async fn index() -> impl IntoResponse {
//!     "Hello, Runique!"
//! }
//! ```
//!
//! ## Modules
//!
//! - [`config_runique`] - Configuration de l'application
//! - [`data_base_runique`] - Gestion de la base de données et ORM
//! - [`formulaire`] - Système de formulaires avec validation
//! - [`gardefou`] - Middlewares de sécurité (CSRF, CSP, sanitization, etc.)
//! - [`macro_runique`] - Macros utilitaires pour le routing et contexte
//! - [`moteur_engine`] - Moteur principal de Runique
//! - [`request_context`] - Gestion du contexte de requête et templates
//! - [`runique_body`] - Builder d'application Runique
//! - [`utils`] - Utilitaires divers

// ============================================================================
// DÉCLARATION DES MODULES
// ============================================================================

pub mod config_runique;
#[cfg(feature = "orm")]
pub mod data_base_runique;
pub mod formulaire;
pub mod gardefou;
#[macro_use]
pub mod macro_runique;
pub mod moteur_engine;
pub mod request_context;
pub mod runique_start;
pub mod utils;

// ============================================================================
// RÉ-EXPORTS GÉNÉRAUX
// ============================================================================

// Frameworks et bibliothèques externes utilisés par Runique
pub use axum;
pub use serde;
pub use serde_json;
pub use tera;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tower_sessions;

#[cfg(feature = "orm")]
pub use sea_orm;

// Ré-export de derive_form
pub use derive_form::DeriveModelForm;

// Ré-export des macros pour qu'elles soient accessibles
// Les macros #[macro_export] sont automatiquement disponibles à la racine

// ============================================================================
// MODULE PRELUDE - Imports simplifiés pour les utilisateurs
// ============================================================================

/// Module prelude contenant tous les imports courants pour démarrer rapidement.
///
/// # Exemple
///
/// ```rust
/// use runique::prelude::*;
///
/// // Tous les types essentiels sont maintenant disponibles
/// ```
pub mod prelude {
    // ========================================================================
    // CORE - Composants principaux de Runique
    // ========================================================================

    /// Moteur principal de Runique
    pub use crate::moteur_engine::RuniqueEngine;

    /// Builder pour créer une application Runique
    pub use crate::runique_start::RuniqueApp;

    /// Configuration de l'application
    pub use crate::config_runique::RuniqueConfig;

    // ========================================================================
    // CONTEXT & TEMPLATES - Gestion du contexte et moteur de templates
    // ========================================================================

    /// Contexte de template pour Tera
    pub use crate::request_context::{AppError, TemplateContext};

    /// Contexte de requête Runique
    pub use crate::request_context::RuniqueContext;

    /// Gestionnaire de messages flash
    pub use crate::request_context::FlashManager;

    /// Utilitaires Tera
    pub use crate::request_context::tera_tool::*;

    // ========================================================================
    // MIDDLEWARES - Sécurité et gestion des requêtes
    // ========================================================================

    // --- CSP (Content Security Policy) ---
    pub use crate::gardefou::composant_middleware::csp_middleware::{
        csp_middleware, csp_report_only_middleware, security_headers_middleware, CspConfig,
        NONCE_KEY,
    };

    // --- CSRF (Cross-Site Request Forgery) ---
    pub use crate::gardefou::composant_middleware::csrf_middleware::csrf_middleware;
    pub use crate::utils::csrf::{mask_csrf_token, unmask_csrf_token, CsrfToken};

    // --- Flash Messages ---
    pub use crate::gardefou::composant_middleware::flash_message::flash_middleware;

    // --- Sanitization ---
    pub use crate::gardefou::composant_middleware::middleware_sanitiser::sanitize_middleware;

    // --- Error Handlers ---
    pub use crate::gardefou::composant_middleware::error_handler::{render_404, render_500};

    // --- Authentication ---
    pub use crate::gardefou::composant_middleware::login_requiert::{
        login_required, redirect_if_authenticated,
    };

    // --- Allowed Hosts ---
    pub use crate::gardefou::composant_middleware::allowed_hosts::AllowedHostsValidator;

    // ========================================================================
    // FORMULAIRES - Système de formulaires avec validation
    // ========================================================================

    /// Gestionnaire de formulaires
    pub use crate::formulaire::builder_form::formmanager::Forms;

    /// Structures de base pour les formulaires
    pub use crate::formulaire::builder_form::base_struct::*;

    /// Options pour les champs de formulaire
    pub use crate::formulaire::builder_form::option_field::*;

    /// Types de champs disponibles
    pub use crate::formulaire::builder_form::field_type::*;

    /// Champ générique
    pub use crate::formulaire::builder_form::generique_field::GenericField;

    /// Traits pour les formulaires
    pub use crate::formulaire::builder_form::trait_form::{FormField, RuniqueForm};

    /// Extracteur de formulaire pour Axum
    pub use crate::formulaire::utils::extracteur::ExtractForm;

    // Macro derive pour générer des formulaires depuis des modèles
    pub use derive_form::DeriveModelForm;

    // ========================================================================
    // ROUTING & URL REVERSING - Gestion des routes et URLs
    // ========================================================================

    /// Enregistrer une URL nommée
    pub use crate::macro_runique::router::register_name_url::register_name_url;

    /// Inverser une URL par son nom
    pub use crate::macro_runique::router::reverse;

    /// Inverser une URL avec paramètres
    pub use crate::macro_runique::router::reverse_with_parameters;

    // ========================================================================
    // MACROS - Macros de contexte et helpers
    // ========================================================================

    /// Macros pour le contexte
    pub use crate::macro_runique::context_macro::*;

    /// Macros pour SeaORM - impl_objects!, get_or_return!
    /// Importées automatiquement via #[macro_use]

    // Note : Les macros suivantes sont disponibles directement :
    // - context! : Créer un contexte Tera facilement
    // - success!, error!, info!, warning! : Envoyer des messages flash
    // - flash_now! : Créer des messages flash immédiats
    // - urlpatterns! : Définir des routes avec noms
    // - view! : Créer des routes GET/POST
    // - impl_objects! : Ajouter .objects à un modèle SeaORM
    // - get_or_return! : Macro utilitaire pour gérer les Result

    // ========================================================================
    // UTILITIES - Utilitaires divers
    // ========================================================================

    /// Génération de tokens
    pub use crate::utils::csrf::*;

    /// Parsing HTML
    pub use crate::utils::parse_html::*;

    // ========================================================================
    // AXUM - Types et extracteurs Axum
    // ========================================================================

    pub use axum::{
        extract::{Extension, Form, Path, Query, State},
        http::{HeaderMap, HeaderValue, Method, StatusCode},
        middleware,
        response::{Html, IntoResponse, Redirect, Response},
        routing::{any, delete, get, patch, post, put},
        Json, Router,
    };

    // ========================================================================
    // ORM - Base de données (feature "orm")
    // ========================================================================
    #[cfg(feature = "orm")]
    pub use crate::data_base_runique::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};

    #[cfg(feature = "orm")]
    pub use crate::data_base_runique::composant_data_base::*;

    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self, entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ColumnTrait,
        ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait, NotSet,
        QueryFilter, QueryOrder, QuerySelect, Set,
    };

    // ========================================================================
    // SÉRIALISATION & FORMATS - JSON, TOML, etc.
    // ========================================================================

    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use serde_json::{from_str, json, to_string, Value};

    // ========================================================================
    // TEMPLATES - Tera
    // ========================================================================

    pub use tera::{Context, Tera};

    // ========================================================================
    // ASYNC & CONCURRENCY
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
    // SESSIONS
    // ========================================================================

    pub use anyhow::{Context as AnyhowContext, Error, Result};
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
    pub use regex::Regex;
    pub use tower_sessions::{Session, SessionManagerLayer};
    pub use uuid::Uuid;
}
