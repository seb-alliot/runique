//! Main request context: `AppError`, `RuniqueContext`, and Tera context construction.
use crate::app::templates::TemplateLoader;
use crate::auth::current_user::CurrentUser;
use crate::errors::error::ErrorContext;
use crate::flash::Message;
use crate::forms::{
    extractor::{Prisme, csrf_required, prisme_pipeline},
    field::RuniqueForm,
};
use crate::impl_from_error;
use crate::middleware::security::anti_bot::HoneypotFieldName;
use crate::utils::aliases::{ADb, AEngine, AppResult, StrMap};
use crate::utils::trad::{t, tf};
use crate::utils::url_params::UrlParams;
use crate::utils::{csp_nonce::CspNonce, csrf::CsrfToken};
use axum::{
    body::Body,
    extract::{FromRequest, FromRequestParts, Path},
    http::{Request as HttpRequest, StatusCode, method::Method},
    response::{Html, IntoResponse, Response},
};
use sea_orm::DbErr;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use std::sync::Arc;
use tera::Context;
use tower_sessions::Session;
use tracing::error;

// --- ERROR HANDLING ---

/// Application error returned by handlers: encapsulates an [`ErrorContext`] and implements [`IntoResponse`].
pub struct AppError {
    /// Error context: status code, message, type.
    pub context: ErrorContext,
}

impl AppError {
    /// Wraps an [`ErrorContext`] into an `AppError`.
    pub fn new(context: ErrorContext) -> Self {
        Self { context }
    }

    /// Converts a Tera rendering error into an `AppError`, logging detail in debug mode.
    pub fn map_tera(e: tera::Error, route: &str, tera: &tera::Tera) -> Box<Self> {
        // Log the detailed error in the console
        error!(
            template = route,
            error = ?e,
            "Template rendering error"
        );

        Box::new(Self {
            context: ErrorContext::from_tera_error(&e, route, tera),
        })
    }
}

impl_from_error!(anyhow::Error => from_anyhow, DbErr => database);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.context.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        //  Log the error in detail
        error!(
            status = status.as_u16(),
            error_type = ?self.context.error_type,
            message = %self.context.message,
            "AppError occurred"
        );

        let mut res = status.into_response();
        //  Insert ErrorContext so that the middleware can retrieve it
        res.extensions_mut().insert(Arc::new(self.context));
        res
    }
}

impl IntoResponse for Box<AppError> {
    fn into_response(self) -> Response {
        (*self).into_response()
    }
}

// --- TEMPLATE CONTEXT ---

/// Request context automatically extracted in handlers via `FromRequest`.
/// Contains the engine, session, flash messages, CSRF token, Tera context,
/// and the Prisme-extracted form data (query params on GET, body on POST).
#[derive(Clone)]
pub struct Request {
    /// Shared application engine.
    pub engine: AEngine,
    /// Session of the current request.
    pub session: Session,
    /// Session flash messages.
    pub notices: Message,
    /// Request CSRF token (masked in the Tera context).
    pub csrf_token: CsrfToken,
    /// Pre-filled Tera context (csrf_token, debug, messages, user…).
    pub context: Context,
    /// HTTP method of the request.
    pub method: Method,
    /// HTTP headers of the request.
    pub headers: axum::http::HeaderMap,
    /// Path parameters (`/{id}`).
    pub path_params: StrMap,
    /// Raw query string (`?a=1&b=2`), preserved for typed deserialization.
    pub raw_query: String,
    /// Query string parameters.
    pub query_params: StrMap,
    /// Current user (None if not authenticated).
    pub user: Option<CurrentUser>,
    /// Parsed form data from the request (query params on GET, body on POST).
    /// CSRF is validated — csrf_valid = false signals an invalid token.
    pub prisme: Prisme,
    /// Honeypot field name injected by anti_bot middleware (None if middleware not active).
    pub honeypot_field_name: Option<String>,
    /// Inside the admin's constraint, the account its middleware checked:
    /// forms built here save through the gate, as this account.
    pub(crate) admin_constraint: Option<CurrentUser>,
}

impl<S> FromRequest<S> for Request
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: HttpRequest<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let err = |msg: &str| (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response();

        let (mut parts, body) = req.into_parts();
        let ex = &parts.extensions;

        let engine = ex
            .get::<AEngine>()
            .cloned()
            .ok_or_else(|| err("engine missing"))?;
        let csrf_token = ex
            .get::<CsrfToken>()
            .cloned()
            .ok_or_else(|| err("csrf missing"))?;
        let session = ex
            .get::<Session>()
            .cloned()
            .ok_or_else(|| err("session missing"))?;
        let nonce = ex.get::<CspNonce>().map(|n| n.as_str()).unwrap_or_default();
        let user = ex.get::<CurrentUser>().cloned();
        let honeypot_field_name = ex.get::<HoneypotFieldName>().map(|h| h.0.clone());
        let admin_constraint = ex
            .get::<crate::auth::gate::AdminConstraint>()
            .map(|c| c.0.clone());

        let notices = Message {
            session: session.clone(),
        };
        let messages = notices.get_all().await;

        let mut context = Context::new();
        context.insert("debug", &engine.config.debug);
        context.insert(
            "csrf_token",
            &csrf_token
                .masked()
                .unwrap_or_else(|_| csrf_token.clone())
                .as_str(),
        );
        context.insert("csp_nonce", nonce);
        context.insert("static_runique", &engine.config.static_files);
        context.insert("messages", &messages);
        if let Some(ref u) = user {
            context.insert("current_user", u);
        }

        let path_params = Path::<StrMap>::from_request_parts(&mut parts, state)
            .await
            .map(|Path(p)| p)
            .unwrap_or_default();

        let ico_image = std::env::var("ICON_IMAGE")
            .unwrap_or("/runique/static/favicon_runique.ico".to_string());
        let public_url = engine.config.server.public_url.as_deref();
        let ico_image = crate::utils::resolve_og_image(public_url, &ico_image);
        context.insert("icon_image", &ico_image);

        let og_image =
            std::env::var("OG_IMAGE").unwrap_or("/runique/static/runique_320.avif".to_string());
        let og_image = crate::utils::resolve_og_image(public_url, &og_image);
        context.insert("og_image", &og_image);

        context.insert("current_path", parts.uri.path());

        let raw_query = parts.uri.query().unwrap_or_default().to_string();
        let query_params = serde_urlencoded::from_str::<StrMap>(&raw_query).unwrap_or_default();

        let method = parts.method.clone();
        let headers = parts.headers.clone();

        // Run Prisme pipeline (aegis → CSRF check)
        let req = HttpRequest::from_parts(parts, body);
        let prisme = prisme_pipeline(req, state).await?;

        Ok(Self {
            engine,
            session,
            notices,
            csrf_token,
            context,
            method,
            headers,
            path_params,
            raw_query,
            query_params,
            user,
            prisme,
            honeypot_field_name,
            admin_constraint,
        })
    }
}

impl Request {
    /// Builds a bare `Request` outside of the normal `FromRequest` extraction path
    /// (e.g. for tests or internally-constructed requests), with empty path/query
    /// params, no user, and `prisme.csrf_valid` fixed to `false` since no body was
    /// ever parsed to validate a CSRF token against.
    pub fn new(engine: AEngine, session: Session, csrf_token: CsrfToken, method: Method) -> Self {
        let mut context = tera::Context::new();
        // mod reload for templates in debug mode
        // The backend cannot be reloaded here because it is shared between requests
        context.insert("debug", &engine.config.debug);
        context.insert("static_runique", &engine.config.static_files);
        context.insert(
            "csrf_token",
            &csrf_token
                .masked()
                .unwrap_or_else(|_| csrf_token.clone())
                .as_str(),
        );

        Self {
            engine,
            session: session.clone(),
            notices: Message { session },
            csrf_token,
            context,
            method,
            headers: axum::http::HeaderMap::new(),
            path_params: StrMap::new(),
            raw_query: String::new(),
            query_params: StrMap::new(),
            user: None,
            prisme: Prisme {
                data: Default::default(),
                // RuniqueContext (FromRequestParts) never runs check_csrf — no body access.
                // Fail-closed: only `Request` via `request.form()` can actually validate CSRF.
                csrf_valid: false,
            },
            honeypot_field_name: None,
            admin_constraint: None,
        }
    }

    /// Unique generic rendering to avoid duplication
    pub fn render(&mut self, template: &str) -> AppResult<Response> {
        let html_result = if self.engine.config.debug {
            // In debug mode, Tera is fully reinitialized with the Loader
            // This applies Regex on {% messages %}, {% form.xxx %}, etc.
            match TemplateLoader::init(&self.engine.config, self.engine.url_registry.clone()) {
                Ok(dev_tera) => {
                    let res = dev_tera.render(template, &self.context);
                    if let Err(ref e) = res {
                        // Detailed log of the Tera error with all sources
                        error!(
                            template = template,
                            error_kind = ?e.kind(),
                            error_message = %e,
                            "Tera rendering failed in debug mode"
                        );

                        // Log the full error chain (source)
                        // Uses the source() method from the std::error::Error trait
                        use std::error::Error as StdError;
                        if let Some(source) = e.source() {
                            error!(
                                source_error = %source,
                                "Tera error source"
                            );
                        }
                    }
                    res
                }
                Err(e) => {
                    error!(
                        template = template,
                        error = %e,
                        "Failed to initialize TemplateLoader in debug mode"
                    );
                    return Err(AppError::map_tera(
                        tera::Error::message(e.to_string()),
                        template,
                        &self.engine.tera,
                    ));
                }
            }
        } else {
            // Production mode: uses the already transformed instance
            self.engine.tera.render(template, &self.context)
        };

        html_result
            .map(|html| Html(html).into_response())
            .map_err(|e| AppError::map_tera(e, template, &self.engine.tera))
    }

    /// Fluent insertion with builder pattern
    pub fn insert(mut self, key: &str, value: impl serde::Serialize) -> Self {
        // Tera 2 stores keys as `Cow<'static, str>`; the borrowed `&str` of this
        // public signature is owned here so callers keep passing plain `&str`.
        self.context.insert(key.to_string(), &value);
        self
    }

    /// Loads the signed-in user's groups and rights from the database into
    /// `self.user` and the template's `current_user`, for a view that checks a
    /// right (`rights_on`, `can_access_resource`). Not loaded by default:
    /// most pages never look at them.
    pub async fn load_user_rights(&mut self) {
        if let Some(user) = self.user.as_mut() {
            crate::auth::permissions::load_rights(&self.engine.db, user).await;
            self.context.insert("current_user", &*user);
        }
    }

    /// Returns a reference to the database connection.
    pub fn db(&self) -> &ADb {
        &self.engine.db
    }

    /// Base of the absolute links an app sends out (activation, reset, emails):
    /// the public URL set with `.with_public_url()`, else — in debug only —
    /// `http://` + the request's `Host`. `None` in production without a public
    /// URL: `Host` is the client's to choose, and a link built on it would carry
    /// its token to someone else's site.
    #[must_use]
    pub fn public_url(&self) -> Option<String> {
        crate::auth::password::public_link_base(
            self.engine.config.server.public_url.as_deref(),
            &self.headers,
            self.engine.config.debug,
        )
    }

    /// The id `login` stored in the session; `None` for an anonymous visitor.
    pub async fn user_id(&self) -> Option<crate::utils::pk::Pk> {
        crate::auth::session::get_user_id(&self.session).await
    }

    /// Returns a path segment as a string slice (`/users/{id}` → `get_path("id")`).
    pub fn get_path(&self, key: &str) -> Option<&str> {
        self.path_params.get(key).map(|s| s.as_str())
    }

    /// Parses a path segment into any type implementing `FromStr`.
    pub fn get_path_as<T: FromStr>(&self, key: &str) -> Option<T> {
        self.path_params.get(key)?.parse().ok()
    }

    /// Returns a query parameter as a string slice (`?page=2` → `get_query("page")`).
    pub fn get_query(&self, key: &str) -> Option<&str> {
        self.query_params.get(key).map(|s| s.as_str())
    }

    /// Deserializes the full query string into a typed struct deriving
    /// `serde::Deserialize`. Unknown keys are ignored; empty values (`key=`)
    /// are dropped, so an `Option` field gets `None` rather than failing.
    ///
    /// A query string that doesn't fit `T` (`?page=abc` for a `u32`) is a
    /// `400 Bad Request`: `request.query::<Filters>()?` in a handler renders
    /// `400.html` (overridable), with the reason on the debug page.
    pub fn query<T: DeserializeOwned>(&self) -> AppResult<T> {
        let cleaned = self
            .raw_query
            .split('&')
            .filter(|pair| pair.split('=').nth(1).is_none_or(|v| !v.is_empty()))
            .collect::<Vec<_>>()
            .join("&");
        serde_urlencoded::from_str(&cleaned).map_err(|e| {
            Box::new(AppError::new(ErrorContext::bad_request(&tf(
                "error.invalid_query",
                &[e.to_string()],
            ))))
        })
    }

    /// Returns an `UrlParams` combining path and query — to be passed to `form.cleaned()`
    pub fn url_params(&self) -> UrlParams<'_> {
        UrlParams::new(&self.path_params, &self.query_params)
    }

    /// Constructs a form `T`, fills it with submitted data (POST body or query string on GET),
    /// and injects the honeypot field if the anti-bot middleware is active.
    /// Call `form.is_valid().await` before reading cleaned values or calling `save()`.
    pub fn form<T: RuniqueForm>(&self) -> T {
        let masked = self
            .csrf_token
            .masked()
            .unwrap_or_else(|_| self.csrf_token.clone());
        let mut form =
            crate::forms::field::build_form::<T>(self.engine.tera.clone(), masked.as_str());
        form.get_form_mut()
            .set_url_params(&self.path_params, &self.query_params);

        if let Some(user) = &self.admin_constraint {
            form.get_form_mut().save_comportement =
                crate::forms::form::SaveComportement::Gated { user: user.clone() };
        }

        if let Some(ref hp_name) = self.honeypot_field_name {
            form.get_form_mut().set_honeypot(hp_name);
            if self.method == Method::POST
                && self.prisme.data.get(hp_name).is_some_and(|v| !v.is_empty())
            {
                form.get_form_mut().force_invalid = true;
            }
        }

        // CSRF enforcement: Prisme computes `csrf_valid` but never rejects on its own.
        // Without this, a mutating request with a missing/invalid token silently passes
        // is_valid(). Fail closed on any non-safe method (same policy as the pipeline).
        //
        // `force_invalid` short-circuits `Forms::is_valid()` before field validation
        // ever runs — `HiddenField::validate()`'s own CSRF branch (`csrf.missing`/
        // `csrf.invalid`) never fires this way, and never did (its `expected_value`
        // is set by nothing outside unit tests). Push a message here directly so a
        // failed submission isn't silently indistinguishable from an untouched form —
        // reuses the same key already shown by the admin CSRF check (`admin_router.rs`).
        if csrf_required(&self.method) && !self.prisme.csrf_valid {
            let csrf = form.get_form_mut();
            csrf.force_invalid = true;
            csrf.errors.push(t("csrf.invalid_or_missing").into_owned());
        }

        form.get_form_mut()
            .fill(&self.prisme.data, self.method.clone());
        form
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod admin_constraint_tests {
    use super::Request;
    use crate::auth::current_user::CurrentUser;
    use crate::auth::gate::{AdminConstraint, test_support::fixtures::pk};
    use crate::auth::permissions::rang::Rang;
    use crate::config::RuniqueConfig;
    use crate::engine::RuniqueEngine;
    use crate::forms::{
        field::{FormAccess, RuniqueForm},
        form::{Forms, SaveComportement},
    };
    use crate::utils::{aliases::AEngine, csrf::CsrfToken};
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{Method, Request as HttpRequest},
    };
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    struct ProbeForm {
        form: Forms,
    }

    impl FormAccess for ProbeForm {
        fn register_fields(_form: &mut Forms) {}
        crate::impl_form_access!();
    }

    impl RuniqueForm for ProbeForm {}

    async fn engine() -> AEngine {
        let conn = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Arc::new(RuniqueEngine::new(
            RuniqueConfig::default(),
            tera::Tera::default(),
            conn,
        ))
    }

    fn session() -> Session {
        Session::new(None, Arc::new(MemoryStore::default()), None)
    }

    fn csrf() -> CsrfToken {
        CsrfToken("ab".repeat(32))
    }

    fn account(n: u8) -> CurrentUser {
        CurrentUser {
            id: pk(n),
            username: format!("account{n}"),
            rang: Rang::Admin,
            groupes: Vec::new(),
            droits: Vec::new(),
        }
    }

    /// The account a form is gated as, `None` when it is open.
    fn gated_as(policy: &SaveComportement) -> Option<crate::utils::pk::Pk> {
        match policy {
            SaveComportement::Gated { user } => Some(user.id),
            SaveComportement::Open => None,
        }
    }

    /// The extractor's input: every extension Runique's own layers put, and the
    /// constraint carrying `constrained` when given.
    async fn extracted(method: Method, constrained: Option<CurrentUser>) -> Request {
        let engine = engine().await;
        let mut http = HttpRequest::builder()
            .method(method)
            .uri("/page")
            .body(Body::empty())
            .unwrap();
        let ext = http.extensions_mut();
        ext.insert(engine.clone());
        ext.insert(Arc::new(engine.config.clone()));
        ext.insert(csrf());
        ext.insert(session());
        if let Some(user) = constrained {
            ext.insert(AdminConstraint(user));
        }
        Request::from_request(http, &())
            .await
            .unwrap_or_else(|_| panic!("extraction failed"))
    }

    #[tokio::test]
    async fn a_request_built_by_hand_is_not_constrained() {
        let req = Request::new(engine().await, session(), csrf(), Method::GET);
        assert!(req.admin_constraint.is_none());
    }

    #[tokio::test]
    async fn the_extension_makes_the_extracted_request_constrained_as_its_account() {
        let req = extracted(Method::GET, Some(account(7))).await;
        assert_eq!(req.admin_constraint.map(|u| u.id), Some(pk(7)));
    }

    #[tokio::test]
    async fn without_the_extension_the_extracted_request_is_not_constrained() {
        assert!(
            extracted(Method::GET, None)
                .await
                .admin_constraint
                .is_none()
        );
    }

    #[tokio::test]
    async fn a_constrained_request_builds_forms_gated_as_its_account() {
        let mut req = Request::new(engine().await, session(), csrf(), Method::GET);
        req.admin_constraint = Some(account(7));
        let form = req.form::<ProbeForm>();
        assert_eq!(gated_as(&form.get_form().save_comportement), Some(pk(7)));
    }

    #[tokio::test]
    async fn an_unconstrained_request_builds_open_forms() {
        let req = Request::new(engine().await, session(), csrf(), Method::GET);
        let form = req.form::<ProbeForm>();
        assert_eq!(gated_as(&form.get_form().save_comportement), None);
    }

    /// The handler can change `req.user`; the form is still gated as the
    /// account the middleware checked.
    #[tokio::test]
    async fn changing_req_user_does_not_change_the_gated_account() {
        let mut req = extracted(Method::GET, Some(account(7))).await;
        req.user = Some(account(9));
        let form = req.form::<ProbeForm>();
        assert_eq!(gated_as(&form.get_form().save_comportement), Some(pk(7)));
    }

    /// A signed-in user outside the constraint gives an open form.
    #[tokio::test]
    async fn a_user_without_the_constraint_builds_open_forms() {
        let mut req = extracted(Method::GET, None).await;
        req.user = Some(account(9));
        let form = req.form::<ProbeForm>();
        assert_eq!(gated_as(&form.get_form().save_comportement), None);
    }

    /// Both policies through the real extraction path, extension to form.
    #[tokio::test]
    async fn the_policy_follows_the_extension_end_to_end() {
        let gated = extracted(Method::GET, Some(account(7)))
            .await
            .form::<ProbeForm>();
        assert_eq!(gated_as(&gated.get_form().save_comportement), Some(pk(7)));
        let open = extracted(Method::GET, None).await.form::<ProbeForm>();
        assert_eq!(gated_as(&open.get_form().save_comportement), None);
    }
}
