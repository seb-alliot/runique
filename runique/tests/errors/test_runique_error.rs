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
        msg.to_lowercase().contains("interdit") || msg.to_lowercase().contains("forbidden"),
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
    let err = RuniqueError::Template("template.html manquant".to_string());
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
    let err = RuniqueError::Template("erreur template".to_string());
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
fn test_error_context_with_details() {
    let ctx =
        ErrorContext::generic(StatusCode::BAD_REQUEST, "err").with_details("Détail supplémentaire");
    assert_eq!(ctx.details, Some("Détail supplémentaire".to_string()));
}

#[test]
fn test_error_context_timestamp_non_vide() {
    let ctx = ErrorContext::generic(StatusCode::OK, "ok");
    assert!(!ctx.timestamp.is_empty());
}

#[test]
fn test_error_context_environment_info() {
    let ctx = ErrorContext::generic(StatusCode::OK, "ok");
    assert!(!ctx.environment.rust_version.is_empty() || ctx.environment.rust_version == "N/A");
    assert!(!ctx.environment.app_version.is_empty());
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
    let err = RuniqueError::Template("template.html".to_string());
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
    let err = RuniqueError::Template("rendu.html".to_string());
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
    RuniqueError::Template("tmpl".to_string()).log();
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
// ErrorContext::with_request
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_with_request_captures_method_and_path() {
    use axum::body::Body;
    use axum::http::Request;
    let req = Request::builder()
        .method("GET")
        .uri("/resource?id=1")
        .header("x-trace-id", "abc")
        .body(Body::empty())
        .unwrap();
    let ctx = ErrorContext::generic(StatusCode::NOT_FOUND, "not found").with_request(&req);
    let info = ctx.request_info.unwrap();
    assert_eq!(info.method, "GET");
    assert_eq!(info.path, "/resource");
    assert_eq!(info.query, Some("id=1".to_string()));
}

#[test]
fn test_with_request_filters_sensitive_headers() {
    use axum::body::Body;
    use axum::http::Request;
    let req = Request::builder()
        .method("POST")
        .uri("/login")
        .header("authorization", "Bearer secret")
        .header("cookie", "session=abc")
        .header("x-request-id", "123")
        .body(Body::empty())
        .unwrap();
    let ctx = ErrorContext::generic(StatusCode::FORBIDDEN, "interdit").with_request(&req);
    let headers = ctx.request_info.unwrap().headers;
    assert!(!headers.contains_key("authorization"));
    assert!(!headers.contains_key("cookie"));
    assert!(headers.contains_key("x-request-id"));
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
