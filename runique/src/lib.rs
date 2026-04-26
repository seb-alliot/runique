#![doc = include_str!("../README.md")]

// ---------------------------------------------------------------------------
// Main Modules
// ---------------------------------------------------------------------------*
pub mod app;
pub mod auth;
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
// Main Dependencies Re-exports
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

pub use derive_form::{extend, model};
pub use dotenvy;
pub use pulldown_cmark;

// ---------------------------------------------------------------------------
// Simplified Prelude
// ---------------------------------------------------------------------------
pub mod prelude {
    // ========================================================================
    // ERRORS
    // ========================================================================
    pub use crate::errors::ErrorContext;
    pub use crate::errors::RuniqueError;
    pub use crate::utils::init_logging;
    pub use tracing;

    // ========================================================================
    // MAIN MODULES
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
            file::{FileField, FileSize},
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
    pub use crate::utils::config::Pk;
    pub use derive_form::{extend, form, model};

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    use crate::chrono;
    pub use crate::config::{security::SecurityConfig, server::ServerConfig};
    pub use crate::utils::env::{is_debug, load_env};
    pub use crate::utils::trad::{Lang, current_lang, set_lang};
    pub use dotenvy;

    // ========================================================================
    // UTILS
    // ========================================================================
    pub use crate::macros::{
        RouterExt, register_name_url, register_pending, reverse, reverse_with_parameters,
    };
    pub use crate::utils::csp_nonce::*;
    pub use crate::{
        context_update, error, flash_now, impl_form_access, impl_objects, info, search, success,
        urlpatterns, view, warning,
    };

    // ========================================================================
    // CONTEXT & TEMPLATE
    // ========================================================================
    // pub use crate::context::error::*;
    pub use crate::context::request::RuniqueContext;

    // ========================================================================
    // MIDDLEWARE
    // ========================================================================
    pub use crate::auth::user as runique_users;
    pub use crate::auth::*;
    pub use crate::auth::{
        ForgotPasswordForm, PasswordResetAdapter, PasswordResetConfig, PasswordResetForm,
        handle_forgot_password, handle_password_reset,
    };
    pub use crate::middleware::{
        allowed_hosts::*, cache::*, config::*, csp::*, csrf::*, errors::*, rate_limit::RateLimiter,
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
    // ORM (optional)
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
    // SERIALIZATION & DATA
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
    // STANDARD TYPES
    // ========================================================================
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Arc;

    // ========================================================================
    // SECURITY - HMAC, Hashing, etc.
    // ========================================================================
    pub use crate::utils::mailer::{Email, mailer_configured};
    pub use crate::utils::password::{
        AutoConfig, Manual, PasswordConfig, hash, password_init, verify,
    };
    pub use crate::utils::reset_token;
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

    // Items used by daemon-generated code (external crate) via `use runique::prelude::*`
    pub use crate::admin::{
        admin_main::{PrototypeAdminState, admin_get, admin_get_id, admin_post, admin_post_id},
        builtin::builtin_resources,
        config::config_admin::AdminConfig,
        helper::{
            dyn_form::DynForm,
            resource_entry::{
                CountFn, CreateFn, DeleteFn, FilterFn, FormBuilder, GetFn, GroupAction, ListFn,
                ListParams, ResourceEntry, SortDir, UpdateFn,
            },
        },
        registry::AdminRegistry,
        resource::{
            AdminIdType, AdminResource, ColumnFilter, CrudOperation, DisplayConfig,
            ResourcePermissions,
        },
        table_admin::migrations_table,
    };
    pub use futures_util::future::BoxFuture;
}
