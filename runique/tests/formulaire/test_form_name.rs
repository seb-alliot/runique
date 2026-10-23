//! Tests — nom du formulaire (`FormNameField`, `ValidationForm::try_form`) :
//! champ caché `runique_form`, ordre des champs, `id` préfixés, rendu qui
//! échoue sur un formulaire jamais nommé.
//!
//! Rendus avec les vrais templates de `templates/field_html/` et les filtres
//! Runique : c'est le seul niveau où l'erreur Tera du nom manquant se voit.

use crate::helpers::{handler_req::build_handler_req, server::build_engine};
use axum::http::Method;
use runique::context::register_asset_filters;
use runique::forms::{field::RuniqueForm, fields::text::TextField, form::Forms};
use runique::prelude::ValidationForm;
use runique::utils::aliases::new_registry;
use runique::utils::constante::session_key::session::{CSRF_TOKEN_KEY, FORM_NAME_KEY};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

struct SearchForm {
    form: Forms,
}

impl RuniqueForm for SearchForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("q").label("Q"));
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

/// Tera configurée comme celle du framework, avec les templates de champs réels.
fn tera() -> Arc<Tera> {
    let mut tera = Tera::default();
    tera.autoescape_on(vec!["html", "xml"]);
    register_asset_filters(
        &mut tera,
        "/static".to_string(),
        "/media".to_string(),
        "/runique/static".to_string(),
        "/runique/media".to_string(),
        new_registry(),
    );
    tera.add_raw_templates(vec![
        (
            "base_form_name.html",
            include_str!("../../templates/field_html/base_form_name.html"),
        ),
        (
            "base_string.html",
            include_str!("../../templates/field_html/base_string.html"),
        ),
        ("csrf.html", include_str!("../../templates/csrf/csrf.html")),
    ])
    .unwrap();
    Arc::new(tera)
}

/// Un formulaire nommé par `try_form` (GET nu : jamais validé, revient en `Err`).
async fn named(name: &'static str) -> SearchForm {
    let engine = build_engine().await;
    let request = build_handler_req(engine, None, HashMap::new(), Method::GET).await;
    let form = SearchForm::build(tera(), "tok");
    ValidationForm::try_form(form, &request, name)
        .await
        .err()
        .expect("a bare GET never validates")
}

fn unnamed() -> SearchForm {
    SearchForm::build(tera(), "tok")
}

fn json(form: &SearchForm) -> Value {
    serde_json::to_value(form.get_form()).unwrap()
}

// ── Ordre des champs ─────────────────────────────────────────────────────────

#[test]
fn the_form_name_comes_first_and_csrf_second() {
    let form = unnamed();
    let keys: Vec<&str> = form.get_form().fields.keys().map(String::as_str).collect();
    assert_eq!(&keys[..3], &[FORM_NAME_KEY, CSRF_TOKEN_KEY, "q"]);
}

// ── Rendu complet ────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_named_form_renders_its_name_first_then_csrf_then_its_fields() {
    let html = json(&named("search").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    let name = html
        .find(r#"<input type="hidden" name="runique_form" value="search">"#)
        .expect("the name field must be rendered");
    let csrf = html.find(r#"name="csrf_token""#).expect("csrf rendered");
    let field = html.find(r#"name="q""#).expect("field rendered");
    assert!(
        name < csrf && csrf < field,
        "order must be name, csrf, fields"
    );
    assert!(!html.contains("Render error"));
}

#[test]
fn an_unnamed_form_fails_to_render_visibly() {
    let html = json(&unnamed())["html"].as_str().unwrap().to_string();
    assert!(
        html.contains("Render error"),
        "a form try_form never named must not render silently, got: {html}"
    );
    assert!(!html.contains(r#"name="runique_form""#));
}

#[tokio::test]
async fn the_form_name_is_escaped_in_its_hidden_field() {
    let html = json(&named("a\"b").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(html.contains(r#"value="a&quot;b""#), "got: {html}");
    assert!(!html.contains(r#"value="a"b""#));
}

// ── id HTML préfixés ─────────────────────────────────────────────────────────

#[tokio::test]
async fn field_ids_and_labels_carry_the_form_name() {
    let html = json(&named("search").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(html.contains(r#"id="id_search-q""#), "got: {html}");
    assert!(html.contains(r#"for="id_search-q""#));
    assert!(html.contains(r#"id="container_search-q""#));
    assert!(!html.contains(r#"id="id_q""#));
}

#[tokio::test]
async fn two_forms_with_the_same_field_get_different_ids() {
    let a = json(&named("first").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    let b = json(&named("second").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(a.contains(r#"id="id_first-q""#));
    assert!(b.contains(r#"id="id_second-q""#));
}

#[tokio::test]
async fn the_csrf_field_has_no_id() {
    let html = json(&named("search").await)["html"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!html.contains("id_csrf_token"));
    assert!(html.contains(r#"name="csrf_token""#));
}

// ── Rendu champ par champ ────────────────────────────────────────────────────

#[tokio::test]
async fn a_named_form_pre_renders_its_name_field() {
    let rendered = json(&named("search").await)["rendered_fields"][FORM_NAME_KEY]
        .as_str()
        .unwrap()
        .to_string();
    assert!(rendered.contains(r#"value="search""#), "got: {rendered}");
}

#[test]
fn an_unnamed_form_shows_the_failed_field_instead_of_dropping_it() {
    let rendered = json(&unnamed())["rendered_fields"][FORM_NAME_KEY]
        .as_str()
        .expect("a failed field must stay in rendered_fields")
        .to_string();
    assert!(rendered.contains("Render error"), "got: {rendered}");
}

#[tokio::test]
async fn field_by_field_rendering_injects_the_name_then_csrf_before_the_first_field() {
    let mut page = Tera::default();
    page.register_filter("form", runique::context::tera::form::FormFilter);
    page.add_raw_template("t", "{{ f | form(field='q') }}")
        .unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("f", &json(&named("search").await));
    let out = page.render("t", &ctx).unwrap();

    let name = out.find("runique_form").expect("name injected");
    let csrf = out.find("csrf_token").expect("csrf injected");
    let field = out.find(r#"id="id_search-q""#).expect("field rendered");
    assert!(name < csrf && csrf < field, "got: {out}");
}

// ── Valeur du nom : posée par try_form seulement ─────────────────────────────

#[tokio::test]
async fn clear_values_keeps_the_form_name() {
    let mut form = named("search").await;
    form.get_form_mut().add_value("q", "rust");
    form.get_form_mut().clear_values();
    assert_eq!(form.get_form().fields[FORM_NAME_KEY].value(), "search");
    assert_eq!(form.get_form().fields["q"].value(), "");
}

#[test]
fn setting_the_name_by_hand_does_not_mark_the_form_submitted() {
    let mut form = unnamed();
    form.get_form_mut().add_value(FORM_NAME_KEY, "search");
    assert!(!form.is_submitted());

    let mut form = unnamed();
    form.get_form_mut().add_value("q", "rust");
    assert!(form.is_submitted());
}
