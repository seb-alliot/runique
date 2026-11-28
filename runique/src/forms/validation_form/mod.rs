//! `ValidationForm<F>`: proof by type that a form has passed validation. A form
//! of an entity is written from it (`save`, `delete`); any other action stays
//! in the caller's business logic.
use crate::{forms::traits::runique_form::RuniqueForm, http::Request};
use sea_orm::DbErr;
use std::ops::Deref;

mod write;

/// A form that has passed validation. The only way to obtain one is
/// [`ValidationForm::try_form`], so a function taking `ValidationForm<F>` instead
/// of `&mut F` proves at the type level that its caller already validated —
/// no need to re-check `is_valid()` or trust a convention.
pub struct ValidationForm<F: RuniqueForm> {
    form: F,
}

impl<F: RuniqueForm> ValidationForm<F> {
    /// Registers dynamic fields, then validates the form — only when the
    /// request sends back the name `request.form(name)` gave it, and the HTTP
    /// method allows it (`allow_get`/`allow_post`).
    ///
    /// A submission carrying another name (another form on the same page), or
    /// none, returns `Err(form)` without running `is_valid()`: that form was
    /// never filled from the request, no field error is set on it, and its
    /// `is_submitted()` is false. On
    /// a failed validation, `form` comes back with its field errors so the
    /// caller can re-render it — `is_submitted()` true, to show its message:
    ///
    /// ```rust,ignore
    /// Err(form) if form.is_submitted() => { /* sent and refused: errors + message */ }
    /// Err(form) => { /* not sent: plain display */ }
    /// ```
    pub async fn try_form(mut form: F, request: &Request) -> Result<Self, F> {
        let sent = form.forms().sent_by(request);
        // A value the handler put in (`add_value`, a prefill) marks the form
        // submitted too: only the form whose name came back is.
        form.forms_mut().keep_submitted_if(sent);
        let validate = match request.method.is_safe() {
            true => form.allow_get(request),
            false => form.allow_post(request),
        };

        if validate && sent && form.is_valid().await {
            Ok(Self { form })
        } else {
            Err(form)
        }
    }

    /// Reclaims the validated form to act on it (custom save, dispatch email, etc.).
    pub fn into_form(self) -> F {
        self.form
    }

    /// Records a DB error on the validated form's error state — e.g. a unique
    /// constraint violation caught by the caller's own save logic, after
    /// `is_valid()` already passed. Lets the caller re-render the form with
    /// that error without unwrapping it via `into_form()` first.
    pub fn database_error(&mut self, err: &DbErr) {
        self.form.database_error(err);
    }
}

impl<F: RuniqueForm> Deref for ValidationForm<F> {
    type Target = F;
    fn deref(&self) -> &F {
        &self.form
    }
}
