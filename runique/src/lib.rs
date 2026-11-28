#![doc = include_str!("../README.md")]

// In the crate's own tests, `::runique::…` (what `model!{}` and `extend!{}`
// generate) names the crate itself.
#[cfg(test)]
extern crate self as runique;

// ---------------------------------------------------------------------------
// Mutually exclusive database engines
// ---------------------------------------------------------------------------
// Exemptions: cfg(doc) (docs.rs/`cargo doc`, which builds with `all-databases`
// to document every backend) and the `all-databases` feature itself,
// requested explicitly (multi-engine tooling — see scripts/smoke_migrations.sh).
#[cfg(all(
    not(doc),
    not(feature = "all-databases"),
    feature = "postgres",
    feature = "mysql"
))]
compile_error!(
    "features `postgres` and `mysql` are mutually exclusive — pick a single engine (or `all-databases` explicitly for multi-engine tooling)"
);
#[cfg(all(
    not(doc),
    not(feature = "all-databases"),
    feature = "postgres",
    feature = "sqlite"
))]
compile_error!(
    "features `postgres` and `sqlite` are mutually exclusive — pick a single engine (or `all-databases` explicitly for multi-engine tooling)"
);
#[cfg(all(
    not(doc),
    not(feature = "all-databases"),
    feature = "mysql",
    feature = "sqlite"
))]
compile_error!(
    "features `mysql` and `sqlite` are mutually exclusive — pick a single engine (or `all-databases` explicitly for multi-engine tooling)"
);

// ---------------------------------------------------------------------------
// Main Modules
// ---------------------------------------------------------------------------*
pub mod acme;
pub mod app;
pub mod atomic;
pub mod auth;
pub mod config;
#[cfg(feature = "orm")]
pub mod db;
pub mod engine;
pub mod flash;
pub mod forms;
pub mod http;
pub mod log;
pub mod macros;
pub mod mailer;
pub mod router;
pub mod schema;
pub mod security;
pub mod templates;

/// Internationalization — the language in use, the `t()` / `tf()` helpers
/// and the project's own translations.
/// The engine and the messages live in the `runique_i18n` crate.
#[doc = include_str!("../doc-tests/i18n/switch_lang.md")]
pub mod translation {
    pub use runique_i18n::{
        Lang, LangId, current_lang, keys,
        project::{LanguageReport, LocaleError, ProjectReport, load_project, project_dir},
        t, tf, with_lang,
    };
}

#[cfg(feature = "admin")]
pub mod admin;
pub mod errors;
pub mod middleware;

pub mod utils;

/// Every framework migration, under the path the projects' generated
/// `migration/src/lib.rs` imports (`runique::prelude::migrations_table`).
pub use auth::migrations_table;

#[cfg(feature = "test-utils")]
pub mod runique_test;

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
pub use runique_dsl;
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
pub mod versions;

// ---------------------------------------------------------------------------
// Simplified Prelude
// ---------------------------------------------------------------------------
pub mod prelude {
    // ========================================================================
    // ERRORS
    // ========================================================================
    pub use crate::{
        errors::{ErrorContext, RuniqueError},
        log::{
            AdminTracing, AuthTracing, BuilderTracing, DbTracing, ErrorsTracing, FormTracing,
            LogOutput, LogRecord, LogRotation, LogSink, MailerTracing, MiddlewareTracing,
            MigrationTracing, RuniqueLog, SessionTracing, TemplatesTracing, init_logging,
        },
    };
    pub use tracing::{self, Level as LogLevel};

    // ========================================================================
    // MAIN MODULES
    // ========================================================================
    pub use crate::{
        app::{RuniqueApp, RuniqueAppBuilder},
        config::app::RuniqueConfig,
        engine::RuniqueEngine,
        errors::AppError,
        flash::{FlashMessage, Message, MessageLevel},
        forms::{
            FormEntity, Forms, Prisme,
            fields::{
                CheckboxField, DateField, DurationField, RadioField, TimeField,
                boolean::BooleanField,
                choice::{ChoiceField, ChoiceOption},
                datetime::DateTimeField,
                file::{FileField, FileSize},
                hidden::HiddenField,
                media::MediaField,
                number::NumericField,
                special::{ColorField, IPAddressField, JSONField, SlugField, UUIDField},
                text::TextField,
            },
            generic::{GeneratedField, GenericField},
            traits::{
                model_form::ModelForm,
                runique_form::{FormAccess, FormField, RuniqueForm},
            },
            validation_form::ValidationForm,
        },
        http::Request,
        schema::{ModelSchema, Pk},
        utils::aliases::*,
    };
    pub use derive_form::{extend, form, model};

    // ========================================================================
    // CONFIGURATION
    // ========================================================================
    use crate::chrono;
    pub use crate::{
        config::{
            env::{is_debug, is_dev},
            security::SecurityConfig,
            server::ServerConfig,
        },
        translation::{Lang, current_lang},
    };
    pub use dotenvy;

    // ========================================================================
    // UTILS
    // ========================================================================
    pub use crate::{
        context_update, error, flash_now, impl_form_access, impl_objects, info,
        router::{
            RouterExt, Routes, UrlError, register_name_url, register_pending, reverse,
            reverse_with_parameters,
        },
        search, search_cond,
        security::csp_nonce::*,
        success, urlpatterns, view, warning,
    };

    // ========================================================================
    // CONTEXT & TEMPLATE
    // ========================================================================
    pub use crate::http::context::RuniqueContext;

    // ========================================================================
    // MIDDLEWARE
    // ========================================================================
    pub use crate::{
        auth::{
            ForgotPasswordForm, PasswordResetConfig, PasswordResetForm, handle_forgot_password,
            handle_password_reset, user as runique_users, user::BuiltinUserEntity, *,
        },
        middleware::{
            allowed_hosts::*, cache::*, config::*, csp::*, csrf::*, errors::*,
            permissions_policy::*, rate_limit::RateLimiter, trusted_proxies::*,
        },
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
    pub use crate::macros::database::list::{HasLists, ListField};
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
    pub use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    // ========================================================================
    // SECURITY - HMAC, Hashing, etc.
    // ========================================================================
    pub use crate::{
        mailer::{Email, dispatch_email, mailer_configured},
        security::{
            password::{AutoConfig, Manual, PasswordConfig, hash, password_init, verify},
            reset_token,
        },
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

    #[cfg(feature = "admin")]
    pub use crate::admin::{
        AdminConfig, AdminState, CrudOperation, inject_admin_prefix, insert_admin_messages,
    };
    // Declaring the admin: `src/admin.rs`, its models, their templates.
    #[cfg(feature = "admin")]
    pub use crate::admin::template::{AdminTemplate, Block, GlobalBlock, ModelTemplates, Page};
    #[cfg(feature = "admin")]
    pub use crate::admin::{AdminSite, AdminView, ModelAdmin, ModelMeta, admin_page_context};
    #[cfg(feature = "admin")]
    pub use crate::app::staging::AdminStaging;
    #[cfg(feature = "admin")]
    pub use sea_orm::Order;
    // A page declared `admin` in `urlpatterns!`: the route only checks access to
    // the admin, what the handler reads and writes goes through the gate.
    #[cfg(feature = "admin")]
    pub use crate::auth::gate::check::{authorize_list, authorize_view, readable_ids};
    #[cfg(feature = "admin")]
    pub use crate::auth::gate::{
        Authorized, Create, Crud, CrudAction, Delete, GateCtx, List, Refusal, Update, View,
        authorize_create, authorize_row, write,
    };
    pub use crate::{
        auth::permissions::group,
        db::sql_dialect::{ilike, text_cast_type, text_eq, text_eq_ci},
        migrations_table,
    };
    pub use futures_util::future::BoxFuture;
}
