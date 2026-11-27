//! `AppError`: the error a handler returns. It carries an [`ErrorContext`] and
//! becomes the HTTP response, which the error middleware then renders.
use super::error::ErrorContext;
use crate::impl_from_error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use std::sync::Arc;
use tracing::error;

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

impl_from_error!(
    anyhow::Error => from_anyhow,
    DbErr => database,
    tower_sessions::session::Error => database
);

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
