//! Definition of all type aliases: `ATera`, `ADb`, `StrMap`, `FieldsMap`, `AppResult`…
use crate::auth::session::CurrentUser;
use crate::config::app::RuniqueConfig;
use crate::context::template::AppError;
use crate::engine::RuniqueEngine;
use crate::prelude::{HostPolicy, SecurityPolicy};
use crate::utils::{csp_nonce::CspNonce, csrf::CsrfToken};
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, result::Result, sync::Arc, sync::RwLock};
use tera::{Result as TeraResult, Tera, Value};
use tower_sessions::{SessionManagerLayer, SessionStore};

// Import for new aliases
use crate::flash::FlashMessage;
use crate::forms::base::FormField;
use indexmap::IndexMap;

// ============================================================================
// ARC<T> ALIASES - THREAD-SAFE SHARED TYPES
// ============================================================================

/// Tera
pub type ATera = Arc<Tera>;
pub type OATera = Option<ATera>;

/// Database Connection
pub type ADb = Arc<DatabaseConnection>;
pub type Bdd = Option<DatabaseConnection>;
pub type OADb = Option<ADb>;

/// Security Policy CSP
pub type ASecurityCsp = Arc<SecurityPolicy>;
pub type OSecurityCsp = Option<ASecurityCsp>;

/// Security Policy Hosts
pub type ASecurityHosts = Arc<HostPolicy>;
pub type OSecurityHosts = Option<ASecurityHosts>;

/// Runique Engine
pub type AEngine = Arc<RuniqueEngine>;
pub type OAEngine = Option<AEngine>;

/// Runique Config
pub type ARuniqueConfig = Arc<RuniqueConfig>;
pub type OARuniqueConfig = Option<ARuniqueConfig>;

/// Session Store (for `SessionBackend::Custom`)
pub type ASessionStore = Arc<dyn SessionStore + Send + Sync>;

// ============================================================================
// OPTION<T> ALIASES - OPTIONAL TYPES
// ============================================================================

/// Current User
pub type OCurrentUser = Option<CurrentUser>;

/// CSRF Token
pub type OCsrfToken = Option<CsrfToken>;

/// CSP Nonce
pub type OCspNonce = Option<CspNonce>;

// ============================================================================
// COLLECTIONS ALIASES - STANDARD COLLECTIONS
// ============================================================================
// Convention: ONE alias per concrete type (avoids repetition)
// Names describe the structure, not the specific usage

// --- Core Collections ---
/// String-to-String map (headers, form data, attributes, errors, etc.)
pub type StrMap = HashMap<String, String>;

/// String-to-Vec<String> map (raw multipart/urlencoded form data)
pub type StrVecMap = HashMap<String, Vec<String>>;

/// String-to-JSON map (Tera args, serialized form data)
pub type JsonMap = HashMap<String, Value>;

/// Ordered form fields collection
pub type FieldsMap = IndexMap<String, Box<dyn FormField>>;

/// Flash messages list
pub type Messages = Vec<FlashMessage>;

// --- URL Registry ---
/// URL Registry (existing - kept for compatibility)
pub type ARlockmap = Arc<RwLock<HashMap<String, String>>>;

/// Pending URL registrations (name, path)
pub type PendingUrls = Vec<(String, String)>;

// ============================================================================
// SESSION ALIASES
// ============================================================================

/// Alias session
pub type Session<S> = SessionManagerLayer<S>;

// ============================================================================
// RESULT ALIASES - RETURN TYPES
// ============================================================================

/// Application Result Type
pub type AppResult<T> = Result<T, Box<AppError>>;

/// Tera Result Type
pub type TResult = TeraResult<Value>;

/// Database Result Type (optional, for `SeaORM`)
#[cfg(feature = "orm")]
pub type DbResult<T> = Result<T, sea_orm::DbErr>;
