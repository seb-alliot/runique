//! Built-in password reset flow: forgot + reset via email token.
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::Serialize;
use std::{marker::PhantomData, sync::Arc};

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
}

// ─── PasswordResetForm ────────────────────────────────────────────────────────

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

        if password.len() < 10 {
            errors.insert(
                "password".to_string(),
                tf("reset.password_min_length", &["10"]).clone(),
            );
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
}

// ─── Config ───────────────────────────────────────────────────────────────────

/// Password reset flow configuration registered via the builder.
#[derive(Clone)]
pub struct PasswordResetConfig {
    pub forgot_route: String,
    pub reset_route: String,
    pub forgot_template: String,
    pub reset_template: String,
    pub email_template: Option<String>,
    pub success_redirect: String,
    pub base_url: Option<String>,
    pub max_requests: u64,
    pub retry_after: u64,
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
        }
    }
}

impl PasswordResetConfig {
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn forgot_route(mut self, route: &str) -> Self {
        self.forgot_route = route.to_string();
        self
    }
    #[must_use]
    pub fn reset_route(mut self, route: &str) -> Self {
        self.reset_route = route.to_string();
        self
    }
    #[must_use]
    pub fn forgot_template(mut self, template: &str) -> Self {
        self.forgot_template = template.to_string();
        self
    }
    #[must_use]
    pub fn reset_template(mut self, template: &str) -> Self {
        self.reset_template = template.to_string();
        self
    }
    #[must_use]
    pub fn success_redirect(mut self, redirect: &str) -> Self {
        self.success_redirect = redirect.to_string();
        self
    }
    #[must_use]
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }
    #[must_use]
    pub fn email_template(mut self, template: &str) -> Self {
        self.email_template = Some(template.to_string());
        self
    }
}

// ─── handle_forgot_password ───────────────────────────────────────────────────

pub async fn handle_forgot_password<E: UserEntity + 'static>(
    request: &mut Request,
    form: &mut ForgotPasswordForm,
    template: &str,
    forgot_route: &str,
    reset_path: &str,
    base_url: Option<&str>,
    email_template: Option<&str>,
) -> AppResult<Response> {
    request.context.insert("lang", &current_lang().code());
    if request.is_get() {
        context_update!(request => {
            "title"       => t("reset.forgot_title").as_ref(),
            "forgot_form" => &*form,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        let email = form.cleaned_string("email").unwrap_or_default();
        let email = email.trim().to_lowercase();

        let db = request.engine.db.clone();

        if let Some(user) = E::find_by_email(&db, &email).await {
            let token = crate::utils::reset_token::generate(&email);
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
                            msg.send().await.ok();
                            if let Some(level) = log_level {
                                crate::runique_log!(level, "reset email sent");
                            }
                        });
                    }
                } else {
                    let body = tf("reset.email_body", &[&username, &reset_url, &reset_url]).clone();
                    let log_level = crate::utils::runique_log::get_log()
                        .auth
                        .as_ref()
                        .and_then(|a| a.reset);
                    tokio::spawn(async move {
                        mail.html(body).send().await.ok();
                        if let Some(level) = log_level {
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
        return Ok(Redirect::to(forgot_route).into_response());
    }

    context_update!(request => {
        "title"       => t("reset.forgot_title").as_ref(),
        "forgot_form" => &*form,
    });
    request.render(template)
}

// ─── handle_password_reset ────────────────────────────────────────────────────

pub async fn handle_password_reset<E: UserEntity + 'static>(
    request: &mut Request,
    form: &mut PasswordResetForm,
    token: String,
    encrypted_email: String,
    template: &str,
) -> AppResult<Response> {
    request.context.insert("lang", &current_lang().code());
    logout(&request.session, None).await.ok();

    let Some(email) = crate::utils::reset_token::decrypt_email(&token, &encrypted_email) else {
        request
            .notices
            .error(t("reset.invalid_or_expired").to_string())
            .await;
        return Ok(Redirect::to("/").into_response());
    };

    if !crate::utils::reset_token::peek(&token) {
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

    if request.is_get() {
        form.get_form_mut().add_value("token", &token);
        form.get_form_mut()
            .add_value("encrypted_email", &encrypted_email);
        context_update!(request => {
            "title"           => t("reset.reset_title").as_ref(),
            "reset_form"      => &*form,
            "token"           => &token,
            "encrypted_email" => &encrypted_email,
        });
        return request.render(template);
    }

    if request.is_post() && form.is_valid().await {
        let Some(stored_email) = crate::utils::reset_token::consume(&token) else {
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

        if stored_email.to_lowercase() != email.to_lowercase() {
            request
                .notices
                .error(t("reset.invalid_or_expired").to_string())
                .await;
            return Ok(Redirect::to("/").into_response());
        }

        let db = request.engine.db.clone();
        let email_clean = form.cleaned_string("email").unwrap_or_default();
        let new_hash = form.cleaned_string("password").unwrap_or_default();

        match E::update_password(&db, email_clean.trim(), &new_hash).await {
            Ok(()) => {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .auth
                    .as_ref()
                    .and_then(|a| a.reset)
                {
                    crate::runique_log!(level, email = %email_clean, "password reset ok");
                }
                form.clear();
                context_update!(request => {
                    "title"           => t("reset.success_title").as_ref(),
                    "reset_form"      => &*form,
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
    }

    context_update!(request => {
        "title"           => t("reset.reset_title").as_ref(),
        "reset_form"      => &*form,
        "token"           => &token,
        "encrypted_email" => &encrypted_email,
    });
    request.render(template)
}

// ─── Builder — auto-registered routes ──────────────────────────────────────

/// Type erasure trait for the staging builder.
pub trait PasswordResetHandler: Send + Sync + 'static {
    fn build_router(&self, config: Arc<PasswordResetConfig>) -> Router;
}

/// Generic adapter: implements `PasswordResetHandler` for any E: `UserEntity`.
pub struct PasswordResetAdapter<E: UserEntity>(PhantomData<E>);

impl<E: UserEntity + 'static> PasswordResetAdapter<E> {
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
    let mut form: ForgotPasswordForm = request.form();
    handle_forgot_password::<E>(
        &mut request,
        &mut form,
        &state.config.forgot_template,
        &state.config.forgot_route,
        &state.config.reset_route,
        state.config.base_url.as_deref(),
        state.config.email_template.as_deref(),
    )
    .await
}

async fn reset_view<E: UserEntity + 'static>(
    State(state): State<ResetState>,
    Path((token, encrypted_email)): Path<(String, String)>,
    mut request: Request,
) -> AppResult<Response> {
    let mut form: PasswordResetForm = request.form();
    handle_password_reset::<E>(
        &mut request,
        &mut form,
        token,
        encrypted_email,
        &state.config.reset_template,
    )
    .await
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
    pub handler: Box<dyn PasswordResetHandler>,
    pub config: PasswordResetConfig,
}
