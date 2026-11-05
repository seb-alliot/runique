// Tests pour RuniqueError, ErrorContext et ErrorType

use axum::http::StatusCode;
use runique::errors::error::{ErrorContext, ErrorType, RuniqueError};
use runique::middleware::RequestInfoHelper;

// ═══════════════════════════════════════════════════════════════
// RuniqueError — Display
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_error_internal_display() {
    let err = RuniqueError::Internal;
    assert!(!err.to_string().is_empty());
}

#[test]
fn test_runique_error_forbidden_display() {
    let err = RuniqueError::Forbidden;
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("interdit")
            || msg.to_lowercase().contains("forbidden")
            || msg.to_lowercase().contains("access denied"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_runique_error_not_found_display() {
    let err = RuniqueError::NotFound;
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("introuvable") || msg.to_lowercase().contains("not found"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_runique_error_validation_display() {
    let err = RuniqueError::Validation("Champ invalide".to_string());
    let msg = err.to_string();
    assert!(msg.contains("Champ invalide"), "Message inattendu: {}", msg);
}

#[test]
fn test_runique_error_database_display() {
    let err = RuniqueError::Database("connexion refusée".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("connexion refusée"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_runique_error_io_display() {
    let err = RuniqueError::Io("fichier introuvable".to_string());
    let msg = err.to_string();
    assert!(
        msg.contains("fichier introuvable"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_runique_error_template_display() {
    let err = RuniqueError::Template {
        message: "template.html manquant".to_string(),
        line: None,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("template.html manquant"),
        "Message inattendu: {}",
        msg
    );
}

#[test]
fn test_runique_error_custom_display() {
    let err = RuniqueError::Custom {
        message: "Erreur personnalisée".to_string(),
        source: None,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("Erreur personnalisée"),
        "Message inattendu: {}",
        msg
    );
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError — Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_error_clone_internal() {
    let err = RuniqueError::Internal;
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_validation() {
    let err = RuniqueError::Validation("msg".to_string());
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_database() {
    let err = RuniqueError::Database("db err".to_string());
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_custom_perd_source() {
    let err = RuniqueError::Custom {
        message: "msg".to_string(),
        source: None,
    };
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError — From conversions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "fichier manquant");
    let err = RuniqueError::from(io_err);
    assert!(matches!(err, RuniqueError::Io(_)));
    assert!(err.to_string().contains("fichier manquant"));
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError — to_error_context (codes HTTP)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_error_context_not_found_404() {
    let err = RuniqueError::NotFound;
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 404);
}

#[test]
fn test_to_error_context_forbidden_403() {
    let err = RuniqueError::Forbidden;
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 403);
}

#[test]
fn test_to_error_context_validation_400() {
    let err = RuniqueError::Validation("invalide".to_string());
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 400);
}

#[test]
fn test_to_error_context_database_500() {
    let err = RuniqueError::Database("erreur db".to_string());
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_to_error_context_internal_500() {
    let err = RuniqueError::Internal;
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_to_error_context_template_500() {
    let err = RuniqueError::Template {
        message: "erreur template".to_string(),
        line: None,
    };
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 500);
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext — constructeurs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_error_context_new_champs_de_base() {
    let ctx = ErrorContext::new(
        ErrorType::Internal,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Titre",
        "Message d'erreur",
    );
    assert_eq!(ctx.status_code, 500);
    assert_eq!(ctx.title, "Titre");
    assert_eq!(ctx.message, "Message d'erreur");
    assert!(ctx.details.is_none());
    assert!(ctx.stack_trace.is_empty());
}

#[test]
fn test_error_context_not_found() {
    let ctx = ErrorContext::not_found("/ma-page");
    assert_eq!(ctx.status_code, 404);
    assert!(
        ctx.message.contains("/ma-page"),
        "Le path doit être dans le message"
    );
}

#[test]
fn test_error_context_generic_400() {
    let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "Requête invalide");
    assert_eq!(ctx.status_code, 400);
    assert_eq!(ctx.message, "Requête invalide");
}

#[test]
fn test_error_context_generic_500() {
    let ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "Erreur serveur");
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_error_context_timestamp_non_vide() {
    let ctx = ErrorContext::generic(StatusCode::OK, "ok");
    assert!(!ctx.timestamp.is_empty());
}

#[test]
fn test_error_context_environment_info() {
    let ctx = ErrorContext::generic(StatusCode::OK, "ok");
    assert!(!ctx.environment.rust_version.is_empty());
    assert_eq!(ctx.environment.runique_version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_app_package_reads_the_project_manifest() {
    use crate::helpers::temp::TempDir;
    use runique::errors::error::app_package;

    let dir = TempDir::new("rq_app_pkg");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"mon-site\"\nversion = \"2.3.1\"\n",
    )
    .unwrap();
    let (name, version) = app_package(dir.path().to_str().unwrap());
    assert_eq!(name.as_deref(), Some("mon-site"));
    assert_eq!(version.as_deref(), Some("2.3.1"));
}

#[test]
fn test_app_package_follows_workspace_version() {
    use crate::helpers::temp::TempDir;
    use runique::errors::error::app_package;

    let root = TempDir::new("rq_app_ws");
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\n[workspace.package]\nversion = \"7.0.0\"\n",
    )
    .unwrap();
    std::fs::create_dir(root.join("app")).unwrap();
    std::fs::write(
        root.join("app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion.workspace = true\n",
    )
    .unwrap();
    let (name, version) = app_package(root.join("app").to_str().unwrap());
    assert_eq!(name.as_deref(), Some("app"));
    assert_eq!(version.as_deref(), Some("7.0.0"));
}

#[test]
fn test_app_package_without_manifest_gives_nothing() {
    use crate::helpers::temp::TempDir;
    use runique::errors::error::app_package;

    let dir = TempDir::new("rq_app_none");
    assert_eq!(app_package(dir.path().to_str().unwrap()), (None, None));

    // A manifest without `[package]` (a bare workspace) gives nothing either.
    std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
    assert_eq!(app_package(dir.path().to_str().unwrap()), (None, None));
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext — build_stack_trace
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_build_stack_trace_peuple_la_trace() {
    let mut ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "erreur");
    let io_err = std::io::Error::other("cause");
    ctx.build_stack_trace(&io_err);
    assert!(!ctx.stack_trace.is_empty());
    assert_eq!(ctx.stack_trace[0].level, 0);
    assert!(ctx.stack_trace[0].message.contains("cause"));
}

#[test]
fn test_build_stack_trace_debug_repr_presente() {
    let mut ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "erreur");
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "accès refusé");
    ctx.build_stack_trace(&io_err);
    assert!(ctx.debug_repr.is_some());
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext — from_anyhow
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_error_context_from_anyhow() {
    let err = anyhow::anyhow!("Erreur anyhow de test");
    let ctx = ErrorContext::from_anyhow(&err);
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.message.contains("Erreur anyhow de test"));
    assert!(!ctx.stack_trace.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError — Clone variantes restantes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_error_clone_forbidden() {
    let err = RuniqueError::Forbidden;
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_not_found() {
    let err = RuniqueError::NotFound;
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_io() {
    let err = RuniqueError::Io("disk full".to_string());
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

#[test]
fn test_runique_error_clone_template() {
    let err = RuniqueError::Template {
        message: "template.html".to_string(),
        line: None,
    };
    let cloned = err.clone();
    assert_eq!(cloned.to_string(), err.to_string());
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError — to_error_context (Io et Custom)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_error_context_io_500() {
    let err = RuniqueError::Io("disk error".to_string());
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_to_error_context_custom_500() {
    let err = RuniqueError::Custom {
        message: "custom".to_string(),
        source: None,
    };
    let ctx = err.to_error_context();
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_to_error_context_stack_trace_presente() {
    let err = RuniqueError::Validation("val".to_string());
    let ctx = err.to_error_context();
    assert!(!ctx.stack_trace.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext — from_runique_error
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_from_runique_error_internal() {
    let err = RuniqueError::Internal;
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 500);
}

#[test]
fn test_from_runique_error_forbidden() {
    let err = RuniqueError::Forbidden;
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 403);
}

#[test]
fn test_from_runique_error_not_found_avec_path() {
    let err = RuniqueError::NotFound;
    let ctx = ErrorContext::from_runique_error(&err, Some("/ma-page"), None, None, None);
    assert_eq!(ctx.status_code, 404);
    assert!(ctx.message.contains("/ma-page"));
}

#[test]
fn test_from_runique_error_not_found_sans_path() {
    let err = RuniqueError::NotFound;
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 404);
}

#[test]
fn test_from_runique_error_validation() {
    let err = RuniqueError::Validation("champ invalide".to_string());
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 400);
    assert!(ctx.message.contains("champ invalide"));
}

#[test]
fn test_from_runique_error_io() {
    let err = RuniqueError::Io("disk full".to_string());
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.message.contains("disk full"));
}

#[test]
fn test_from_runique_error_template() {
    let err = RuniqueError::Template {
        message: "rendu.html".to_string(),
        line: None,
    };
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.message.contains("rendu.html"));
}

#[test]
fn test_from_runique_error_custom() {
    let err = RuniqueError::Custom {
        message: "erreur custom".to_string(),
        source: None,
    };
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.message.contains("erreur custom"));
}

#[test]
fn test_from_runique_error_a_stack_trace() {
    let err = RuniqueError::Internal;
    let ctx = ErrorContext::from_runique_error(&err, None, None, None, None);
    assert!(!ctx.stack_trace.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// read_template_source
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_read_template_source_inexistant_retourne_none() {
    use runique::errors::error::read_template_source;
    let result = read_template_source("template_qui_nexiste_pas.html");
    assert!(result.is_none());
}

#[test]
fn test_read_template_source_lit_un_template_reel() {
    use runique::errors::error::read_template_source;
    assert!(read_template_source("errors/corps-error/template-info.html").is_some());
}

/// Le nom peut venir d'une requête : la page de debug ne doit pas montrer un
/// fichier hors de `templates/`. `../Cargo.toml` existe bien, il serait lu sans la garde.
#[test]
fn test_read_template_source_refuse_de_sortir_de_templates() {
    use runique::errors::error::read_template_source;
    assert!(std::path::Path::new("templates/../Cargo.toml").exists());
    assert!(read_template_source("../Cargo.toml").is_none());
    assert!(read_template_source("errors/../../Cargo.toml").is_none());
    assert!(read_template_source("/etc/hostname").is_none());
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext — database
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_error_context_database_500() {
    let ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "db error");
    assert_eq!(ctx.status_code, 500);
    assert_eq!(ctx.message, "db error");
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext::database — constructeur réel
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_error_context_database_constructor() {
    use sea_orm::DbErr;
    let ctx = ErrorContext::database(DbErr::Custom("connexion perdue".to_string()));
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.message.contains("connexion perdue"));
    assert!(!ctx.stack_trace.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError::log — ne doit pas paniquer
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_log_all_variants_no_panic() {
    use runique::app::error_build::BuildError;
    RuniqueError::Internal.log();
    RuniqueError::Forbidden.log();
    RuniqueError::NotFound.log();
    RuniqueError::Validation("champ".to_string()).log();
    RuniqueError::Database("db".to_string()).log();
    RuniqueError::Io("io".to_string()).log();
    RuniqueError::Template {
        message: "tmpl".to_string(),
        line: None,
    }
    .log();
    RuniqueError::Custom {
        message: "msg".to_string(),
        source: None,
    }
    .log();
    RuniqueError::Build(BuildError::validation("v")).log();
}

// ═══════════════════════════════════════════════════════════════
// RuniqueError::into_response — codes HTTP
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_into_response_not_found_404() {
    use axum::response::IntoResponse;
    let resp = RuniqueError::NotFound.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_into_response_forbidden_403() {
    use axum::response::IntoResponse;
    let resp = RuniqueError::Forbidden.into_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_into_response_internal_500() {
    use axum::response::IntoResponse;
    let resp = RuniqueError::Internal.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_into_response_validation_400() {
    use axum::response::IntoResponse;
    let resp = RuniqueError::Validation("bad".to_string()).into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════
// From<BuildError> for RuniqueError
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_from_build_error() {
    use runique::app::error_build::BuildError;
    let build_err = BuildError::validation("config manquante");
    let err = RuniqueError::from(build_err);
    assert!(matches!(err, RuniqueError::Build(_)));
    assert!(err.to_string().contains("config manquante"));
}

#[test]
fn test_runique_error_build_display() {
    use runique::app::error_build::BuildError;
    let err = RuniqueError::Build(BuildError::database_missing());
    assert!(!err.to_string().is_empty());
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext::with_request_helper
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_with_request_helper() {
    let helper = RequestInfoHelper {
        method: "POST".to_string(),
        path: "/api/test".to_string(),
        query: Some("foo=bar".to_string()),
        headers: Default::default(),
    };
    let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "err").with_request_helper(&helper);
    let info = ctx.request_info.unwrap();
    assert_eq!(info.method, "POST");
    assert_eq!(info.path, "/api/test");
    assert_eq!(info.query, Some("foo=bar".to_string()));
}

#[test]
fn test_with_request_helper_no_query() {
    let helper = RequestInfoHelper {
        method: "GET".to_string(),
        path: "/page".to_string(),
        query: None,
        headers: Default::default(),
    };
    let ctx =
        ErrorContext::generic(StatusCode::NOT_FOUND, "not found").with_request_helper(&helper);
    assert!(ctx.request_info.unwrap().query.is_none());
}

// ═══════════════════════════════════════════════════════════════
// ErrorContext::from_tera_error
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_from_tera_error_status_500() {
    let tera = tera::Tera::default();
    let tera_err =
        tera::Tera::one_off("{{ invalid syntax }", &tera::Context::new(), false).unwrap_err();
    let ctx = ErrorContext::from_tera_error(&tera_err, "broken.html", &tera);
    assert_eq!(ctx.status_code, 500);
    assert!(ctx.template_info.is_some());
}

#[test]
fn test_from_tera_error_template_name_stored() {
    let tera = tera::Tera::default();
    let tera_err = tera::Tera::one_off("{% for %}", &tera::Context::new(), false).unwrap_err();
    let ctx = ErrorContext::from_tera_error(&tera_err, "mytemplate.html", &tera);
    let info = ctx.template_info.unwrap();
    assert_eq!(info.name, "mytemplate.html");
}

// Written from cargo-mutants survivors (2026-10-02).
#[test]
fn test_to_error_context_keeps_the_error_category() {
    let db = RuniqueError::Database("down".into()).to_error_context();
    assert!(matches!(db.error_type, ErrorType::Database));
    let tpl = RuniqueError::Template {
        message: "broken".to_string(),
        line: None,
    }
    .to_error_context();
    assert!(matches!(tpl.error_type, ErrorType::Template));
    assert_ne!(db.title, tpl.title);
}

/// Builds a real Tera syntax error (an unterminated expression carries a span),
/// so the span's `start_line` is a genuine 1-based line and not parsed from text.
fn tera_syntax_error(source: &str) -> tera::Error {
    tera::Tera::default()
        .add_raw_template("broken.html", source)
        .unwrap_err()
}

#[test]
fn test_from_tera_error_reads_the_line_and_hides_internal_templates() {
    let mut tera = tera::Tera::default();
    tera.add_raw_template("home.html", "x").unwrap();
    tera.add_raw_template("404.html", "y").unwrap(); // a Runique builtin name

    // Error on the first line.
    let ctx = ErrorContext::from_tera_error(&tera_syntax_error("{{ x"), "home.html", &tera);
    let info = ctx.template_info.expect("template info");
    assert_eq!(info.line_number, Some(1));
    assert_eq!(info.available_templates, ["home.html"]);

    // Error on a later line: the line is read, not hardcoded.
    let multiline =
        ErrorContext::from_tera_error(&tera_syntax_error("ok\n{{ x"), "home.html", &tera);
    assert_eq!(multiline.template_info.unwrap().line_number, Some(2));

    // A kind without a span (`Msg`) yields no line.
    let no_line = ErrorContext::from_tera_error(&tera::Error::message("oops"), "home.html", &tera);
    assert_eq!(no_line.template_info.unwrap().line_number, None);
}

#[test]
fn test_runique_error_template_carries_the_tera_line() {
    // The typed constructor fills both the message and the span's line.
    let err = RuniqueError::template(&tera_syntax_error("ok\n{{ x"));
    match err {
        RuniqueError::Template { line, .. } => assert_eq!(line, Some(2)),
        other => panic!("expected Template, got {other:?}"),
    }

    // No span (`Msg`) → no line, but still a Template error with its message.
    let err = RuniqueError::template(&tera::Error::message("boom"));
    match err {
        RuniqueError::Template { message, line } => {
            assert_eq!(line, None);
            assert!(message.contains("boom"));
        }
        other => panic!("expected Template, got {other:?}"),
    }
}
