//! Tests — `ValidationForm<F>` (`forms/validation_form.rs`) et les hooks
//! `register_dynamic_fields`/`allow_get`/`allow_post` de `RuniqueForm`.
//!
//! Chaque test vérifie un résultat concret (Ok/Err, contenu des erreurs, valeur
//! nettoyée), pas seulement "ça n'a pas planté".

use crate::helpers::{request::build_handler_req, server::build_engine};
use axum::http::Method;
use runique::forms::{field::RuniqueForm, fields::text::TextField, form::Forms};
use runique::prelude::ValidationForm;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// Formulaire minimal — un seul champ requis. `allow_post` retombe sur le
// défaut (`is_submitted()`) ; `allow_get` est explicitement surchargé —
// son défaut est `false` (sécurité), un formulaire de recherche GET est le
// cas qui doit explicitement opter pour l'auto-validation.
// ═══════════════════════════════════════════════════════════════

struct SearchForm {
    form: Forms,
}

impl RuniqueForm for SearchForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("q").required());
    }
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
    fn allow_get(&self, _request: &runique::context::Request) -> bool {
        self.is_submitted()
    }
}

#[tokio::test]
async fn get_without_data_is_err_and_has_no_field_errors() {
    let engine = build_engine().await;
    let request = build_handler_req(engine, None, HashMap::new(), Method::GET).await;
    let form: SearchForm = request.form();

    match ValidationForm::try_new(form, &request).await {
        Ok(_) => panic!("a bare GET with no data must not validate"),
        Err(form) => {
            // allow_get() returned false before is_valid() ever ran — no
            // "required" error should have been set on the field.
            assert!(
                !form.get_form().has_errors(),
                "no validation attempt should mean no field errors"
            );
        }
    }
}

#[tokio::test]
async fn get_with_data_validates_and_returns_ok() {
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("q".to_string(), "rust".to_string());
    let request = build_handler_req(engine, None, data, Method::GET).await;
    let form: SearchForm = request.form();

    match ValidationForm::try_new(form, &request).await {
        Ok(validated) => {
            assert_eq!(validated.cleaned_string("q"), Some("rust".to_string()));
        }
        Err(_) => panic!("a GET search form with data must validate"),
    }
}

#[tokio::test]
async fn post_missing_required_field_is_err_with_field_error() {
    let engine = build_engine().await;
    let request = build_handler_req(engine, None, HashMap::new(), Method::POST).await;
    let form: SearchForm = request.form();

    match ValidationForm::try_new(form, &request).await {
        Ok(_) => panic!("POST with a missing required field must not validate"),
        Err(form) => {
            let errors = form.get_form().errors();
            assert!(
                errors.contains_key("q"),
                "the required field must carry its own error, got: {errors:?}"
            );
        }
    }
}

#[tokio::test]
async fn post_with_data_validates_and_into_form_returns_the_form() {
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("q".to_string(), "runique".to_string());
    let request = build_handler_req(engine, None, data, Method::POST).await;
    let form: SearchForm = request.form();

    let validated = ValidationForm::try_new(form, &request)
        .await
        .unwrap_or_else(|_| panic!("POST with valid data must validate"));

    let form = validated.into_form();
    assert_eq!(form.cleaned_string("q"), Some("runique".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// register_dynamic_fields — le champ ajouté doit être visible AVANT
// que la validation ne s'exécute
// ═══════════════════════════════════════════════════════════════

struct DynamicForm {
    form: Forms,
}

#[async_trait::async_trait]
impl RuniqueForm for DynamicForm {
    fn register_fields(_form: &mut Forms) {}

    async fn register_dynamic_fields(&mut self, _request: &runique::context::Request) {
        self.get_form_mut()
            .field(&TextField::text("late_field").required());
        self.get_form_mut().add_value("late_field", "present");
    }

    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

#[tokio::test]
async fn register_dynamic_fields_runs_before_validation() {
    let engine = build_engine().await;
    // POST with no body data: the only way `is_valid()` can pass is if the
    // required "late_field" — added and filled exclusively inside
    // `register_dynamic_fields` — was actually registered before validation ran.
    let request = build_handler_req(engine, None, HashMap::new(), Method::POST).await;
    let form: DynamicForm = request.form();

    match ValidationForm::try_new(form, &request).await {
        Ok(validated) => {
            assert_eq!(
                validated.cleaned_string("late_field"),
                Some("present".to_string())
            );
        }
        Err(form) => panic!(
            "late_field must have been registered+filled by register_dynamic_fields before is_valid(), errors: {:?}",
            form.get_form().errors()
        ),
    }
}

// ═══════════════════════════════════════════════════════════════
// allow_get/allow_post — surcharge explicite
// ═══════════════════════════════════════════════════════════════

struct AlwaysPostOnlyForm {
    form: Forms,
}

impl RuniqueForm for AlwaysPostOnlyForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("name"));
    }
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
    // Explicit override matching the (now-default) `false` — kept to assert
    // the override mechanism itself still works, independent of the default.
    fn allow_get(&self, _request: &runique::context::Request) -> bool {
        false
    }
}

#[tokio::test]
async fn explicit_allow_get_false_blocks_validation_even_with_data() {
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("name".to_string(), "alice".to_string());
    let request = build_handler_req(engine, None, data, Method::GET).await;
    let form: AlwaysPostOnlyForm = request.form();

    match ValidationForm::try_new(form, &request).await {
        Ok(_) => panic!("allow_get() override returning false must block validation"),
        Err(form) => {
            // Never attempted: no error should have been recorded either.
            assert!(!form.get_form().has_errors());
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Honeypot — `Request::form()`'s anti-bot trap detection (regression test
// for the `self.is_post()` → `self.method == Method::POST` inlining in
// `context/template.rs`, done when `Request::is_get/is_post/is_put/is_delete`
// were removed as dead API surface).
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn honeypot_filled_on_post_forces_invalid() {
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("q".to_string(), "rust".to_string());
    data.insert("hp_trap".to_string(), "i_am_a_bot".to_string());
    let mut request = build_handler_req(engine, None, data, Method::POST).await;
    request.honeypot_field_name = Some("hp_trap".to_string());

    let mut form: SearchForm = request.form();
    assert!(
        !form.is_valid().await,
        "a filled honeypot field must force the form invalid on POST"
    );
}

#[tokio::test]
async fn honeypot_empty_on_post_does_not_force_invalid() {
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("q".to_string(), "rust".to_string());
    let mut request = build_handler_req(engine, None, data, Method::POST).await;
    request.honeypot_field_name = Some("hp_trap".to_string());

    let mut form: SearchForm = request.form();
    assert!(
        form.is_valid().await,
        "an empty honeypot field must not force the form invalid"
    );
}

#[tokio::test]
async fn honeypot_filled_on_get_does_not_force_invalid() {
    // The honeypot check is POST-only — a filled trap field on a GET (e.g. a
    // crawler replaying query params back) must not trip it. This is exactly
    // the branch that used to read `self.is_post()`.
    let engine = build_engine().await;
    let mut data = HashMap::new();
    data.insert("q".to_string(), "rust".to_string());
    data.insert("hp_trap".to_string(), "i_am_a_bot".to_string());
    let mut request = build_handler_req(engine, None, data, Method::GET).await;
    request.honeypot_field_name = Some("hp_trap".to_string());

    let mut form: SearchForm = request.form();
    assert!(
        form.is_valid().await,
        "honeypot check is POST-only — a filled trap on GET must not force invalid"
    );
}
