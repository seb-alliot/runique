//! Built-in password reset flow: forgot + reset via email token.
use crate::utils::config::TraceResult;
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use futures_util::future::BoxFuture;
use serde::Serialize;
use std::{marker::PhantomData, sync::Arc, time::Duration};

use async_trait::async_trait;

use crate::auth::session::{UserEntity, logout};
use crate::auth::user_trait::RuniqueUser;
use crate::context::template::Request;
use crate::forms::{
    Forms,
    field::RuniqueForm,
    fields::{hidden::HiddenField, text::TextField},
};
use crate::utils::{
    aliases::{AppResult, StrMap},
    trad::{current_lang, t, tf},
};
use crate::{context_update, impl_form_access};

// ─── ForgotPasswordForm ───────────────────────────────────────────────────────

/// "Forgot password" form: a single required email field, used to trigger a
/// reset token email without revealing whether the address exists.
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct ForgotPasswordForm {
    pub form: Forms,
}

impl RuniqueForm for ForgotPasswordForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("email")
                .label(&t("reset.email_label"))
                .required(),
        );
    }

    impl_form_access!();
    // `validator_get` defaults to `false` — the reset-token email send (a
    // state-changing action) never runs on a GET, no override needed.
}

// ─── PasswordResetForm ────────────────────────────────────────────────────────

/// Password reset form submitted from the emailed link: carries the token and
/// encrypted email as hidden fields, plus email/password/confirm for the user
/// to fill in. `clean()` checks the token decrypts to the submitted email and
/// enforces the password policy (10+ chars, upper/lower/digit/special).
#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct PasswordResetForm {
    pub form: Forms,
}

#[async_trait]
impl RuniqueForm for PasswordResetForm {
    fn register_fields(form: &mut Forms) {
        form.field(&HiddenField::new("token"));
        form.field(&HiddenField::new("encrypted_email"));
        form.field(
            &TextField::text("email")
                .label(&t("reset.email_label"))
                .required(),
        );
        form.field(
            &TextField::password("password")
                .label(&t("reset.new_password_label"))
                .required(),
        );
        form.field(
            &TextField::password("confirm")
                .label(&t("reset.confirm_label"))
                .required(),
        );
    }

    async fn clean(&mut self) -> Result<(), StrMap> {
        let token = self.cleaned_string("token").unwrap_or_default();
        let encrypted = self.cleaned_string("encrypted_email").unwrap_or_default();
        let email = self.cleaned_string("email").unwrap_or_default();
        let password = self.cleaned_string("password").unwrap_or_default();
        let confirm = self.cleaned_string("confirm").unwrap_or_default();
        let mut errors = StrMap::new();

        match crate::utils::reset_token::decrypt_email(&token, &encrypted) {
            Some(ref expected) if expected.to_lowercase() == email.trim().to_lowercase() => {}
            Some(_) => {
                errors.insert("email".to_string(), t("reset.email_mismatch").to_string());
            }
            None => {
                errors.insert("token".to_string(), t("reset.invalid_link").to_string());
            }
        }

        const SPECIAL: &str = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                tf("reset.password_min_length", &["10"]).clone(),
            );
        } else if !password.chars().any(|c| c.is_uppercase())
            || !password.chars().any(|c| c.is_lowercase())
            || !password.chars().any(|c| c.is_ascii_digit())
            || !password.chars().any(|c| SPECIAL.contains(c))
        {
            errors.insert("password".to_string(), t("reset.password_weak").to_string());
        }

        if password != confirm {
            errors.insert(
                "confirm".to_string(),
                t("reset.password_mismatch").to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    impl_form_access!();
    // `validator_get` defaults to `false` — the password update never runs on
    // a GET, no override needed.
}

// ─── Config ───────────────────────────────────────────────────────────────────

/// Hook run just before rendering, letting the host app inject template
/// context this generic flow has no way to know about (e.g. site branding
/// pulled from the app's own DB tables). Runs for both the forgot and reset
/// templates.
pub type ExtraContextFn = Arc<dyn for<'a> Fn(&'a mut Request) -> BoxFuture<'a, ()> + Send + Sync>;

/// Password reset flow configuration registered via the builder.
#[derive(Clone)]
pub struct PasswordResetConfig {
    /// Path of the "forgot password" GET/POST route. Default: `/forgot-password`.
    pub forgot_route: String,
    /// Path prefix of the reset route; the token and encrypted email are
    /// appended as path segments (`{reset_route}/{token}/{encrypted_email}`).
    /// Default: `/reset-password`.
    pub reset_route: String,
    /// Template rendered for the "forgot password" page.
    pub forgot_template: String,
    /// Template rendered for the reset-password page (both the form and the
    /// post-reset success state).
    pub reset_template: String,
    /// Template used to render the reset email body. When `None`, a plain
    /// HTML body is built from the `reset.email_body` translation string instead.
    pub email_template: Option<String>,
    /// Configured redirect target for a successful reset. Not currently read
    /// by the built-in reset flow, which re-renders `reset_template` in place
    /// with `reset_done = true` instead of redirecting.
    pub success_redirect: String,
    /// Base URL used to build the reset link sent by email. When `None`, it
    /// is derived from the request's `Host` header (falling back to
    /// `http://localhost:3000`).
    pub base_url: Option<String>,
    /// Maximum number of requests allowed per rate-limit window on the
    /// forgot/reset routes.
    pub max_requests: u64,
    /// Rate-limit window length, in seconds, paired with `max_requests`.
    pub retry_after: u64,
    /// Lifetime of a reset token before it expires. Default: 1 hour.
    pub token_ttl: Duration,
    /// Optional hook to inject extra template context before rendering.
    pub extra_context: Option<ExtraContextFn>,
}

impl Default for PasswordResetConfig {
    fn default() -> Self {
        Self {
            forgot_route: "/forgot-password".to_string(),
            reset_route: "/reset-password".to_string(),
            forgot_template: "auth/forgot_password.html".to_string(),
            reset_template: "auth/reset_password.html".to_string(),
            email_template: None,
            success_redirect: "/".to_string(),
            base_url: None,
            max_requests: 5,
            retry_after: 300,
            token_ttl: Duration::from_secs(3600),
            extra_context: None,
        }
    }
}

impl PasswordResetConfig {
    /// Creates a config with the default routes, templates and rate limits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Overrides the "forgot password" route path (default: `/forgot-password`).
    #[must_use]
    pub fn forgot_route(mut self, route: &str) -> Self {
        self.forgot_route = route.to_string();
        self
    }
    /// Overrides the reset route path prefix (default: `/reset-password`).
    #[must_use]
    pub fn reset_route(mut self, route: &str) -> Self {
        self.reset_route = route.to_string();
        self
    }
    /// Overrides the template rendered for the "forgot password" page.
    #[must_use]
    pub fn forgot_template(mut self, template: &str) -> Self {
        self.forgot_template = template.to_string();
        self
    }
    /// Overrides the template rendered for the reset-password page.
    #[must_use]
    pub fn reset_template(mut self, template: &str) -> Self {
        self.reset_template = template.to_string();
        self
    }
    /// Sets the redirect target recorded for a successful reset. See the
    /// `success_redirect` field doc: the built-in flow does not currently
    /// read this value.
    #[must_use]
    pub fn success_redirect(mut self, redirect: &str) -> Self {
        self.success_redirect = redirect.to_string();
        self
    }
    /// Sets the base URL used to build the reset link sent by email, instead
    /// of deriving it from the request's `Host` header.
    #[must_use]
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }
    /// Sets the template used to render the reset email body.
    #[must_use]
    pub fn email_template(mut self, template: &str) -> Self {
        self.email_template = Some(template.to_string());
        self
    }
    /// Sets how long a reset token stays valid (default: 1 hour).
    #[must_use]
    pub fn token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = ttl;
        self
    }
    /// Registers a hook run before each render, to inject app-specific
    /// template context (e.g. site branding) this generic flow can't
    /// provide on its own.
    #[must_use]
    pub fn extra_context(mut self, hook: ExtraContextFn) -> Self {
        self.extra_context = Some(hook);
        self
    }
}

async fn apply_extra_context(request: &mut Request, hook: &Option<ExtraContextFn>) {
    if let Some(hook) = hook {
        hook(request).await;
    }
}

// ─── handle_forgot_password ───────────────────────────────────────────────────

/// Handles both the GET (render the form) and POST (issue a reset token)
/// sides of the "forgot password" page.
///
/// On POST, whether or not the email matches a user, the response is the same
/// success notice and redirect: the handler never reveals account existence
/// through its response, and the SMTP send is fired via `tokio::spawn` rather
/// than awaited, so an existing account can't be enumerated via response timing.
pub async fn handle_forgot_password<E: UserEntity + 'static>(
    request: &mut Request,
    form: ForgotPasswordForm,
    config: &PasswordResetConfig,
) -> AppResult<Response> {
    let template = config.forgot_template.as_str();
    let forgot_route = config.forgot_route.as_str();
    let reset_path = config.reset_route.as_str();
    let base_url = config.base_url.as_deref();
    let email_template = config.email_template.as_deref();
    let token_ttl = config.token_ttl;
    request.context.insert("lang", &current_lang().code());

    let form = match crate::forms::ValidationForm::try_new(form, request).await {
        Ok(validated) => validated.into_form(),
        Err(form) => {
            apply_extra_context(request, &config.extra_context).await;
            context_update!(request => {
                "title"       => t("reset.forgot_title").as_ref(),
                "forgot_form" => &form,
            });
            return request.render(template);
        }
    };

    let email = form.cleaned_string("email").unwrap_or_default();
    let email = email.trim().to_lowercase();

    let db = request.engine.db.clone();

    if let Some(user) = E::find_by_email(&db, &email).await
        && let Ok(token) = crate::utils::reset_token::generate(&db, user.user_id(), token_ttl).await
    {
        let encrypted_email = crate::utils::reset_token::encrypt_email(&token, &email);

        if let Some(level) = crate::utils::runique_log::get_log()
            .auth
            .as_ref()
            .and_then(|a| a.reset)
        {
            crate::runique_log!(level, %email, "reset token generated");
        }

        let host = base_url
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| {
                request
                    .headers
                    .get("host")
                    .and_then(|v| v.to_str().ok())
                    .map(|h| format!("http://{h}"))
                    .unwrap_or_else(|| "http://localhost:3000".to_string())
            });

        let reset_url = format!(
            "{}/{}/{}/{}",
            host,
            reset_path.trim_matches('/'),
            token,
            encrypted_email
        );

        if crate::utils::mailer_configured() {
            let username = user.username().to_string();
            let subject = t("reset.email_subject").to_string();
            let mail = crate::utils::Email::new()
                .to(email.clone())
                .subject(&subject);
            // Fire-and-forget: do not await the SMTP send — a blocking await would leak
            // whether the email exists via response time (timing enumeration attack).
            if let Some(tpl) = email_template {
                use tera::Context as TeraCtx;
                let mut ctx = TeraCtx::new();
                ctx.insert("username", &username);
                ctx.insert("reset_url", &reset_url);
                if let Ok(msg) = mail.template(&request.engine.tera, tpl, ctx) {
                    let log_level = crate::utils::runique_log::get_log()
                        .auth
                        .as_ref()
                        .and_then(|a| a.reset);
                    tokio::spawn(async move {
                        if msg
                            .send()
                            .await
                            .trace_or(log_level, tracing::Level::WARN, "reset email send")
                            .is_some()
                            && let Some(level) = log_level
                        {
                            crate::runique_log!(level, "reset email sent");
                        }
                    });
                } else {
                    tracing::warn!(
                        template = %tpl,
                        "reset email template render failed — email not sent"
                    );
                }
            } else {
                let body = tf("reset.email_body", &[&username, &reset_url, &reset_url]).clone();
                let log_level = crate::utils::runique_log::get_log()
                    .auth
                    .as_ref()
                    .and_then(|a| a.reset);
                tokio::spawn(async move {
                    if mail
                        .html(body)
                        .send()
                        .await
                        .trace_or(log_level, tracing::Level::WARN, "reset email send")
                        .is_some()
                        && let Some(level) = log_level
                    {
                        crate::runique_log!(level, "reset email sent");
                    }
                });
            }
        }
    }

    // Security: do not reveal if the email exists or not
    request
        .notices
        .success(t("reset.check_inbox").to_string())
        .await;
    Ok(Redirect::to(forgot_route).into_response())
}

// ─── handle_password_reset ────────────────────────────────────────────────────

/// Handles the reset-password page reached from the emailed link: verifies
/// the token, logs out any existing session, then renders the form (GET) or
/// consumes the token and updates the password (POST).
///
/// The token is single-use (`consume`) and resolves to a server-side
/// `user_id`; the password update is applied by that id, not by the email
/// taken from the URL, and the URL email is only cross-checked against the
/// account's real email as a UX sanity check — the URL alone cannot be used
/// to reset an arbitrary account's password.
pub async fn handle_password_reset<E: UserEntity + 'static>(
    request: &mut Request,
    form: PasswordResetForm,
    token: String,
    encrypted_email: String,
    config: &PasswordResetConfig,
) -> AppResult<Response> {
    let template = config.reset_template.as_str();
    request.context.insert("lang", &current_lang().code());
    logout(&request.session, None).await.trace(
        crate::utils::runique_log::get_log()
            .session
            .as_ref()
            .and_then(|s| s.store),
        "logout before password reset",
    );

    let Some(email) = crate::utils::reset_token::decrypt_email(&token, &encrypted_email) else {
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    };

    let db = request.engine.db.clone();

    if !crate::utils::reset_token::peek(&db, &token).await {
        if let Some(level) = crate::utils::runique_log::get_log()
            .auth
            .as_ref()
            .and_then(|a| a.reset)
        {
            crate::runique_log!(level, %email, "reset token invalid or expired");
        }
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    }

    let mut form = match crate::forms::ValidationForm::try_new(form, request).await {
        Ok(validated) => validated.into_form(),
        Err(mut form) => {
            if request.method.is_safe() {
                form.get_form_mut().add_value("token", &token);
                form.get_form_mut()
                    .add_value("encrypted_email", &encrypted_email);
            }
            apply_extra_context(request, &config.extra_context).await;
            context_update!(request => {
                "title"           => t("reset.reset_title").as_ref(),
                "reset_form"      => &form,
                "token"           => &token,
                "encrypted_email" => &encrypted_email,
            });
            return request.render(template);
        }
    };

    let Some(user_id) = crate::utils::reset_token::consume(&db, &token).await else {
        if let Some(level) = crate::utils::runique_log::get_log()
            .auth
            .as_ref()
            .and_then(|a| a.reset)
        {
            crate::runique_log!(level, %email, "reset token consume failed");
        }
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    };

    // The token binds the reset to one user_id (server-derived). Resolve and
    // mutate by that id (IDOR-safe); the URL email is only a UX cross-check.
    let Some(user) = E::find_by_id(&db, user_id).await else {
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    };
    if user.email().to_lowercase() != email.to_lowercase() {
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    }

    let email_clean = form.cleaned_string("email").unwrap_or_default();
    let new_hash = form.cleaned_string("password").unwrap_or_default();

    match E::update_password_by_id(&db, user_id, &new_hash).await {
        Ok(()) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .auth
                .as_ref()
                .and_then(|a| a.reset)
            {
                crate::runique_log!(level, email = %email_clean, "password reset ok");
            }
            form.clear();
            apply_extra_context(request, &config.extra_context).await;
            context_update!(request => {
                "title"           => t("reset.success_title").as_ref(),
                "reset_form"      => &form,
                "reset_done"      => &true,
                "token"           => &token,
                "encrypted_email" => &encrypted_email,
            });
            return request.render(template);
        }
        Err(e) => {
            if let Some(level) = crate::utils::runique_log::get_log()
                .auth
                .as_ref()
                .and_then(|a| a.reset)
            {
                crate::runique_log!(level, email = %email_clean, error = %e, "password reset db error");
            }
            form.get_form_mut().database_error(&e);
        }
    }

    apply_extra_context(request, &config.extra_context).await;
    context_update!(request => {
        "title"           => t("reset.reset_title").as_ref(),
        "reset_form"      => &form,
        "token"           => &token,
        "encrypted_email" => &encrypted_email,
    });
    request.render(template)
}

// ─── Builder — auto-registered routes ──────────────────────────────────────

/// Type erasure trait for the staging builder.
pub trait PasswordResetHandler: Send + Sync + 'static {
    /// Builds the merged forgot+reset router, with rate limiting applied to
    /// both routes.
    fn build_router(&self, config: Arc<PasswordResetConfig>) -> Router;
}

/// Generic adapter: implements `PasswordResetHandler` for any E: `UserEntity`.
pub struct PasswordResetAdapter<E: UserEntity>(PhantomData<E>);

impl<E: UserEntity + 'static> PasswordResetAdapter<E> {
    /// Creates an adapter for the given `UserEntity` implementation.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E: UserEntity + 'static> Default for PasswordResetAdapter<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct ForgotState {
    config: Arc<PasswordResetConfig>,
}

#[derive(Clone)]
struct ResetState {
    config: Arc<PasswordResetConfig>,
}

async fn forgot_view<E: UserEntity + 'static>(
    State(state): State<ForgotState>,
    mut request: Request,
) -> AppResult<Response> {
    let form: ForgotPasswordForm = request.form();
    handle_forgot_password::<E>(&mut request, form, &state.config).await
}

async fn reset_view<E: UserEntity + 'static>(
    State(state): State<ResetState>,
    Path((token, encrypted_email)): Path<(String, String)>,
    mut request: Request,
) -> AppResult<Response> {
    let form: PasswordResetForm = request.form();
    handle_password_reset::<E>(&mut request, form, token, encrypted_email, &state.config).await
}

impl<E: UserEntity + 'static> PasswordResetHandler for PasswordResetAdapter<E> {
    fn build_router(&self, config: Arc<PasswordResetConfig>) -> Router {
        use crate::middleware::security::rate_limit::{RateLimiter, rate_limit_middleware};
        use axum::middleware;
        use axum::routing::any;

        let limiter = Arc::new(
            RateLimiter::new()
                .max_requests(u32::try_from(config.max_requests).unwrap_or(u32::MAX))
                .retry_after(config.retry_after),
        );

        let forgot_state = ForgotState {
            config: config.clone(),
        };
        let reset_state = ResetState { config };

        let forgot_route = Router::new()
            .route(&forgot_state.config.forgot_route, any(forgot_view::<E>))
            .with_state(forgot_state)
            .route_layer(middleware::from_fn_with_state(
                limiter.clone(),
                rate_limit_middleware,
            ));

        let reset_path = format!(
            "{}/{{token}}/{{encrypted_email}}",
            reset_state.config.reset_route.trim_end_matches('/')
        );
        let reset_route = Router::new()
            .route(&reset_path, any(reset_view::<E>))
            .with_state(reset_state)
            .route_layer(middleware::from_fn_with_state(
                limiter,
                rate_limit_middleware,
            ));

        forgot_route.merge(reset_route)
    }
}

/// Staging stored in the builder before construction.
pub struct PasswordResetStaging {
    /// Type-erased adapter used to build the forgot+reset router for the
    /// configured `UserEntity`.
    pub handler: Box<dyn PasswordResetHandler>,
    /// Resolved configuration applied when the router is built.
    pub config: PasswordResetConfig,
}
