#![doc = include_str!("../README.md")]

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
pub mod macros;
pub mod migration;

pub mod admin;
pub mod errors;
pub mod middleware;

pub mod utils;

pub use forms::Prisme;
// ---------------------------------------------------------------------------
// Ré-export des dépendances principales
// ---------------------------------------------------------------------------
pub use anyhow;
pub use argon2;
pub use async_trait;
pub use axum;
pub use chrono;
pub use hmac;
pub use regex;
#[cfg(feature = "orm")]
pub use sea_orm;
pub use serde;
pub use serde_json;
pub use sha2;
pub use std::sync::LazyLock;
pub use tera;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tower_sessions;
pub use uuid;

pub use derive_form::model;
pub use dotenvy;

// ---------------------------------------------------------------------------
// Prelude simplifié
// ---------------------------------------------------------------------------
pub mod prelude {
    // ========================================================================
    // ERREURS
    // ========================================================================
    pub use crate::errors::ErrorContext;
    pub use crate::errors::RuniqueError;
    pub use crate::utils::init_logging;
    pub use tracing;

    // ========================================================================
    // MODULES PRINCIPAUX
    // ========================================================================
    pub use crate::app::{RuniqueApp, RuniqueAppBuilder};
    pub use crate::config::app::RuniqueConfig;
    pub use crate::context::{AppError, Request};
    pub use crate::engine::RuniqueEngine;
    pub use crate::flash::{FlashMessage, Message, MessageLevel};
    pub use crate::forms::{
        Forms, Prisme,
        field::{FormField, RuniqueForm},
        fields::{
            CheckboxField, DateField, DurationField, RadioField, TimeField,
            boolean::BooleanField,
            choice::{ChoiceField, ChoiceOption},
            datetime::DateTimeField,
            file::FileField,
            hidden::HiddenField,
            number::NumericField,
            special::{ColorField, IPAddressField, JSONField, SlugField, UUIDField},
            text::TextField,
        },
        generic::{FieldKind, GenericField},
        model_form::ModelForm,
    };
    pub use crate::migration::schema::ModelSchema;
    pub use crate::utils::aliases::*;
    pub use crate::{error, flash_now, info, success, warning};
    pub use derive_form::{form, model};

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    use crate::chrono;
    pub use crate::config::{security::SecurityConfig, server::ServerConfig};
    pub use crate::utils::env::is_debug;
    pub use crate::utils::trad::{Lang, current_lang, set_lang, t, tf};
    pub use dotenvy;

    // ========================================================================
    // UTILS
    // ========================================================================
    pub use crate::macros::{
        register_name_url, register_pending, reverse, reverse_with_parameters,
    };
    pub use crate::utils::csp_nonce::*;
    pub use crate::utils::csrf::*;
    pub use crate::{urlpatterns, view};

    // ========================================================================
    // CONTEXTE & TEMPLATE
    // ========================================================================
    // pub use crate::context::error::*;
    pub use crate::context::request::RuniqueContext;

    // ========================================================================
    // MIDDLEWARE
    // ========================================================================
    pub use crate::middleware::{
        allowed_hosts::*,
        auth::*,
        cache::*,
        config::*,
        csp::*,
        csrf::*,
        errors::*,
        rate_limit::{RateLimiter, rate_limit_middleware},
    };

    // ========================================================================
    // AXUM & HTTP
    // ========================================================================
    pub use axum::{
        Json, Router,
        extract::{Extension, Form, FromRequestParts, Path, Query, State},
        http::{HeaderMap, HeaderValue, Method, StatusCode, method::*},
        middleware,
        response::{Html, IntoResponse, Redirect, Response},
        routing::{any, delete, get, patch, post, put},
    };

    // ========================================================================
    // ORM (optionnel)
    // ========================================================================
    // pub use crate::migration::user_runique;
    #[cfg(feature = "orm")]
    pub use crate::db::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self, ActiveModelBehavior, ActiveModelTrait, ColumnTrait, ConnectOptions, Database,
        DatabaseConnection, DbErr, EntityTrait, ModelTrait, NotSet, QueryFilter, QueryOrder,
        QuerySelect, Set, entity::prelude::*,
    };
    pub use sea_orm_migration::sea_query;

    // ========================================================================
    // SÉRIALISATION & DONNÉES
    // ========================================================================
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use serde_json;

    // ========================================================================
    // TEMPLATE ENGINE
    // ========================================================================
    pub use tera::{Context, Tera};

    // ========================================================================
    // ASYNC & TOKIO
    // ========================================================================
    pub use async_trait::async_trait;
    pub use std::sync::LazyLock;
    pub use tokio;

    // ========================================================================
    // TYPES STANDARDS COURANTS
    // ========================================================================
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Arc;

    // ========================================================================
    // SÉCURITÉ - HMAC, Hashing, etc.
    // ========================================================================
    pub use crate::utils::password::{
        AutoConfig, Manual, PasswordConfig, hash, password_init, verify,
    };
    pub use hmac::{Hmac, Mac};
    pub use sha2::Sha256;

    // ========================================================================
    // SESSIONS & DATES
    // ========================================================================
    pub use crate::tokio::time::Duration;
    pub use anyhow::{Context as AnyhowContext, Error, Result};
    pub use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
    pub use regex::Regex;
    pub use tower_sessions::{Session, SessionManagerLayer};
    pub use uuid::Uuid;

    // ========================================================================
    // Admin
    // ========================================================================
    pub use crate::admin::config::config_admin::AdminConfig;
    pub use crate::admin::daemon::{generate, parse_admin_file, watch};
    pub use crate::admin::resource::{
        AdminIdType, AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ResourcePermissions,
    };
    pub use crate::admin::router::{AdminState, build_admin_router};
    pub use crate::admin::template::{AdminTemplate, PathAdminTemplate};
    pub use crate::admin::{
        AdminRegistry, CountFn, CreateFn, DeleteFn, DynForm, FormBuilder, GetFn, ListFn,
        PrototypeAdminState, ResourceEntry, UpdateFn, admin_get, admin_get_id, admin_post,
        admin_post_id,
    };
    pub use futures_util::future::BoxFuture;
}
