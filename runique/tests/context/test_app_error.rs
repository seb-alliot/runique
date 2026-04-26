// Tests pour AppError — IntoResponse, From<anyhow::Error>, From<DbErr>, impl_from_error!

use axum::http::StatusCode;
use axum::response::IntoResponse;
use runique::context::AppError;
use runique::errors::{ErrorContext, ErrorType};

// ── AppError::new ────────────────────────────────────────────────

#[test]
fn test_app_error_new_stores_context() {
    let ctx = ErrorContext::not_found("/test");
    let err = AppError::new(ctx);
    assert_eq!(err.context.status_code, 404);
}

// ── AppError::into_response ──────────────────────────────────────

#[test]
fn test_app_error_into_response_status_404() {
    let ctx = ErrorContext::not_found("/missing");
    let err = AppError::new(ctx);
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_app_error_into_response_status_500() {
    let ctx = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "crash");
    let err = AppError::new(ctx);
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_box_app_error_into_response() {
    let ctx = ErrorContext::not_found("/box");
    let boxed: Box<AppError> = Box::new(AppError::new(ctx));
    let resp = boxed.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── From<anyhow::Error> ──────────────────────────────────────────

#[test]
fn test_app_error_from_anyhow() {
    let anyhow_err = runique::anyhow::anyhow!("something went wrong");
    let err = AppError::from(anyhow_err);
    assert_eq!(err.context.status_code, 500);
}

#[test]
fn test_box_app_error_from_anyhow() {
    let anyhow_err = runique::anyhow::anyhow!("boxed error");
    let boxed: Box<AppError> = Box::from(anyhow_err);
    assert_eq!(boxed.context.status_code, 500);
}

// ── From<DbErr> ──────────────────────────────────────────────────

#[test]
fn test_app_error_from_dberr() {
    use runique::sea_orm::DbErr;
    let db_err = DbErr::Custom("db failure".to_string());
    let err = AppError::from(db_err);
    assert_eq!(err.context.status_code, 500);
}

#[test]
fn test_box_app_error_from_dberr() {
    use runique::sea_orm::DbErr;
    let db_err = DbErr::Custom("boxed db".to_string());
    let boxed: Box<AppError> = Box::from(db_err);
    assert_eq!(boxed.context.status_code, 500);
}

// ── ErrorContext helpers ──────────────────────────────────────────

#[test]
fn test_error_context_not_found() {
    let ctx = ErrorContext::not_found("/foo");
    assert_eq!(ctx.status_code, 404);
}

#[test]
fn test_error_context_generic_bad_request() {
    let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "invalid input");
    assert_eq!(ctx.status_code, 400);
}

#[test]
fn test_error_context_new() {
    let ctx = ErrorContext::new(
        ErrorType::Internal,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal error",
        "Details here",
    );
    assert_eq!(ctx.status_code, 500);
    assert_eq!(ctx.title, "Internal error");
    assert_eq!(ctx.message, "Details here");
}

#[test]
fn test_error_context_from_anyhow() {
    let err = runique::anyhow::anyhow!("anyhow test");
    let ctx = ErrorContext::from_anyhow(&err);
    assert!(!ctx.message.is_empty());
    assert_eq!(ctx.status_code, 500);
}
