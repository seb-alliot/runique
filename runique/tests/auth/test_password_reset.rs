//! Tests — auth/password.rs
//! Couvre : PasswordResetConfig builder, PasswordResetForm::clean() (5 branches),
//!          ForgotPasswordForm, PasswordResetAdapter

use runique::auth::{
    ForgotPasswordForm, PasswordResetConfig, PasswordResetForm, password::PasswordResetAdapter,
};
use runique::forms::{field::RuniqueForm, form::Forms};
use runique::utils::reset_token;

// ═══════════════════════════════════════════════════════════════
// PasswordResetConfig — builder
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_config_default_values() {
    let cfg = PasswordResetConfig::new();
    assert_eq!(cfg.forgot_route, "/forgot-password");
    assert_eq!(cfg.reset_route, "/reset-password");
    assert_eq!(cfg.success_redirect, "/");
    assert_eq!(cfg.max_requests, 5);
    assert_eq!(cfg.retry_after, 300);
    assert!(cfg.base_url.is_none());
}

#[test]
fn test_config_forgot_route() {
    let cfg = PasswordResetConfig::new().forgot_route("/mot-de-passe-oublie");
    assert_eq!(cfg.forgot_route, "/mot-de-passe-oublie");
}

#[test]
fn test_config_reset_route() {
    let cfg = PasswordResetConfig::new().reset_route("/reinitialisation");
    assert_eq!(cfg.reset_route, "/reinitialisation");
}

#[test]
fn test_config_forgot_template() {
    let cfg = PasswordResetConfig::new().forgot_template("auth/custom_forgot.html");
    assert_eq!(cfg.forgot_template, "auth/custom_forgot.html");
}

#[test]
fn test_config_reset_template() {
    let cfg = PasswordResetConfig::new().reset_template("auth/custom_reset.html");
    assert_eq!(cfg.reset_template, "auth/custom_reset.html");
}

#[test]
fn test_config_success_redirect() {
    let cfg = PasswordResetConfig::new().success_redirect("/dashboard");
    assert_eq!(cfg.success_redirect, "/dashboard");
}

#[test]
fn test_config_base_url() {
    let cfg = PasswordResetConfig::new().base_url("https://example.com");
    assert_eq!(cfg.base_url, Some("https://example.com".to_string()));
}

#[test]
fn test_config_chained_builders() {
    let cfg = PasswordResetConfig::new()
        .forgot_route("/forgot")
        .reset_route("/reset")
        .base_url("https://example.com")
        .success_redirect("/home");
    assert_eq!(cfg.forgot_route, "/forgot");
    assert_eq!(cfg.reset_route, "/reset");
    assert_eq!(cfg.base_url, Some("https://example.com".to_string()));
    assert_eq!(cfg.success_redirect, "/home");
}

// ═══════════════════════════════════════════════════════════════
// ForgotPasswordForm — construction
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_forgot_password_form_has_email_field() {
    let mut form = ForgotPasswordForm {
        form: Forms::new("csrf"),
    };
    ForgotPasswordForm::register_fields(&mut form.form);
    assert!(form.form.fields.contains_key("email"));
}

// ═══════════════════════════════════════════════════════════════
// PasswordResetAdapter
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_password_reset_adapter_new() {
    use runique::auth::BuiltinUserEntity;
    let _adapter = PasswordResetAdapter::<BuiltinUserEntity>::new();
}

// ═══════════════════════════════════════════════════════════════
// PasswordResetForm::clean() — 5 branches
// ═══════════════════════════════════════════════════════════════

fn make_reset_form() -> PasswordResetForm {
    let mut form = PasswordResetForm {
        form: Forms::new("csrf"),
    };
    PasswordResetForm::register_fields(&mut form.form);
    form
}

#[tokio::test]
async fn test_reset_form_valid() {
    let email = "test@example.com";
    let token = reset_token::generate(email);
    let encrypted = reset_token::encrypt_email(&token, email);

    let mut form = make_reset_form();
    form.form.add_value("token", &token);
    form.form.add_value("encrypted_email", &encrypted);
    form.form.add_value("email", email);
    form.form.add_value("password", "strongpassword1");
    form.form.add_value("confirm", "strongpassword1");

    assert!(form.is_valid().await);
}

#[tokio::test]
async fn test_reset_form_invalid_token_decrypt_fails() {
    // Token inexistant → decrypt_email retourne None
    let mut form = make_reset_form();
    form.form.add_value("token", "inexistant-token");
    form.form.add_value("encrypted_email", "bad-data");
    form.form.add_value("email", "test@example.com");
    form.form.add_value("password", "strongpassword1");
    form.form.add_value("confirm", "strongpassword1");

    assert!(!form.is_valid().await);
    assert!(form.form.errors().contains_key("token"));
}

#[tokio::test]
async fn test_reset_form_email_mismatch() {
    // Token valide mais l'email fourni ne correspond pas à l'email chiffré
    let real_email = "real@example.com";
    let token = reset_token::generate(real_email);
    let encrypted = reset_token::encrypt_email(&token, real_email);

    let mut form = make_reset_form();
    form.form.add_value("token", &token);
    form.form.add_value("encrypted_email", &encrypted);
    form.form.add_value("email", "wrong@example.com");
    form.form.add_value("password", "strongpassword1");
    form.form.add_value("confirm", "strongpassword1");

    assert!(!form.is_valid().await);
    assert!(form.form.errors().contains_key("email"));
}

#[tokio::test]
async fn test_reset_form_password_too_short() {
    let email = "test@example.com";
    let token = reset_token::generate(email);
    let encrypted = reset_token::encrypt_email(&token, email);

    let mut form = make_reset_form();
    form.form.add_value("token", &token);
    form.form.add_value("encrypted_email", &encrypted);
    form.form.add_value("email", email);
    form.form.add_value("password", "short");
    form.form.add_value("confirm", "short");

    assert!(!form.is_valid().await);
    assert!(form.form.errors().contains_key("password"));
}

#[tokio::test]
async fn test_reset_form_password_mismatch() {
    let email = "test@example.com";
    let token = reset_token::generate(email);
    let encrypted = reset_token::encrypt_email(&token, email);

    let mut form = make_reset_form();
    form.form.add_value("token", &token);
    form.form.add_value("encrypted_email", &encrypted);
    form.form.add_value("email", email);
    form.form.add_value("password", "strongpassword1");
    form.form.add_value("confirm", "differentpassword1");

    assert!(!form.is_valid().await);
    assert!(form.form.errors().contains_key("confirm"));
}
