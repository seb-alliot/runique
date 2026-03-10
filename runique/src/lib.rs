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
pub use once_cell;
pub use regex;
#[cfg(feature = "orm")]
pub use sea_orm;
pub use serde;
pub use serde_json;
pub use sha2;
pub use tera;
pub use tokio;
pub use tower;
pub use tower_http;
pub use tower_sessions;
pub use uuid;

pub use derive_form::model;

// ---------------------------------------------------------------------------
// Prelude simplifié
// ---------------------------------------------------------------------------
pub mod prelude {
    // ========================================================================
    // ERREURS
    // ========================================================================
    pub use crate::errors::ErrorContext;
    pub use crate::errors::RuniqueError;
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
        field::{FormField, RuniqueForm},
        fields::{
            boolean::BooleanField,
            choice::{ChoiceField, ChoiceOption},
            datetime::DateTimeField,
            file::FileField,
            hidden::HiddenField,
            number::NumericField,
            special::{ColorField, IPAddressField, JSONField, SlugField, UUIDField},
            text::TextField,
            CheckboxField, DateField, DurationField, RadioField, TimeField,
        },
        generic::{FieldKind, GenericField},
        model_form::ModelForm,
        Forms, Prisme,
    };
    pub use crate::migration::schema::ModelSchema;
    pub use crate::utils::aliases::*;
    pub use derive_form::{form, model};

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    use crate::chrono;
    pub use crate::config::{security::SecurityConfig, server::ServerConfig};
    pub use crate::utils::trad::{current_lang, set_lang, t, tf, Lang};

    // ========================================================================
    // UTILS
    // ========================================================================
    pub use crate::utils::csp_nonce::*;
    pub use crate::utils::csrf::*;

    // ========================================================================
    // CONTEXTE & TEMPLATE
    // ========================================================================
    // pub use crate::context::error::*;
    pub use crate::context::request::RuniqueContext;

    // ========================================================================
    // MIDDLEWARE
    // ========================================================================
    pub use crate::middleware::{
        allowed_hosts::*, auth::*, cache::*, config::*, csp::*, csrf::*, errors::*,
    };

    // ========================================================================
    // AXUM & HTTP
    // ========================================================================
    pub use axum::{
        extract::{Extension, Form, FromRequestParts, Path, Query, State},
        http::{method::*, HeaderMap, HeaderValue, Method, StatusCode},
        middleware,
        response::{Html, IntoResponse, Redirect, Response},
        routing::{any, delete, get, patch, post, put},
        Json, Router,
    };

    // ========================================================================
    // ORM (optionnel)
    // ========================================================================
    // pub use crate::migration::user_runique;
    #[cfg(feature = "orm")]
    pub use crate::db::{DatabaseConfig, DatabaseConfigBuilder, DatabaseEngine};
    #[cfg(feature = "orm")]
    pub use sea_orm::{
        self, entity::prelude::*, ActiveModelBehavior, ActiveModelTrait, ColumnTrait,
        ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait, NotSet,
        QueryFilter, QueryOrder, QuerySelect, Set,
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
    pub use crate::utils::password::{
        hash, password_init, verify, AutoConfig, Manual, PasswordConfig,
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

    // ========================================================================
    // Admin
    // ========================================================================
    pub use crate::admin::config::config_admin::AdminConfig;
    pub use crate::admin::daemon::{generate, parse_admin_file, watch};
    pub use crate::admin::resource::{
        AdminIdType, AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ResourcePermissions,
    };
    pub use crate::admin::router::{build_admin_router, AdminState};
    pub use crate::admin::template::{AdminTemplate, PathAdminTemplate};
    pub use crate::admin::{
        admin_get, admin_get_id, admin_post, admin_post_id, AdminRegistry, CountFn, CreateFn,
        DeleteFn, DynForm, FormBuilder, GetFn, ListFn, PrototypeAdminState, ResourceEntry,
        UpdateFn,
    };
    pub use futures_util::future::BoxFuture;
}
