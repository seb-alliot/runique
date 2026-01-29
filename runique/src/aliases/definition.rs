use crate::config::app::RuniqueConfig;
use crate::context::template::AppError;
use crate::engine::RuniqueEngine;
use crate::middleware::auth::CurrentUser;
use crate::prelude::HostPolicy;
use crate::prelude::SecurityPolicy;
use crate::utils::csp_nonce::CspNonce;
use crate::utils::csrf::CsrfToken;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::result::Result;
use std::sync::Arc;
use std::sync::RwLock;
use tera::{Result as TeraResult, Tera, Value};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, SessionStore};

// --- ALIASES GÉNÉRIQUES ---

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

/// Current User
pub type OCurrentUser = Option<CurrentUser>;

/// CSRF Token
pub type OCsrfToken = Option<CsrfToken>;

/// CSP Nonce
pub type OCspNonce = Option<CspNonce>;

/// Url Registry
pub type ARlockmap = Arc<RwLock<HashMap<String, String>>>;

/// alias session
pub type Session<S> = SessionManagerLayer<S>;

/// --- ALIAS APP RESULT ---
pub type AppResult<T> = Result<T, Box<AppError>>;
pub type TResult = TeraResult<Value>;
