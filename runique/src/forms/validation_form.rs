//! `ValidationForm<F>`: proof by type that a form has passed validation, without
//! imposing any action (save/delete/...) — that stays in the caller's business logic.
use crate::context::Request;
use crate::forms::field::RuniqueForm;
use crate::utils::constante::session_key::session::FORM_NAME_KEY;

/// A form that has passed validation. The only way to obtain one is
/// [`ValidationForm::try_new`], so a function taking `ValidationForm<F>` instead
/// of `&mut F` proves at the type level that its caller already validated —
/// no need to re-check `is_valid()` or trust a convention.
pub struct ValidationForm<F: RuniqueForm> {
    form: F,
}

impl<F: RuniqueForm> ValidationForm<F> {
    /// Registers dynamic fields, decides whether to attempt validation based on
    /// the HTTP method (`allow_get`/`allow_post`), then validates.
    /// Returns `form` unchanged on failure (method not applicable, or invalid
    /// fields) so the caller can re-render it with its field errors.
    pub async fn try_new(mut form: F, request: &Request) -> Result<Self, F> {
        form.register_dynamic_fields(request).await;

        let validate = match request.method.is_safe() {
            true => form.allow_get(request),
            false => form.allow_post(request),
        };

        if validate && form.is_valid().await {
            Ok(Self { form })
        } else {
            Err(form)
        }
    }

    /// [`try_new`](Self::try_new) for a form that shares its page with others:
    /// `name` is rendered as a hidden field, and the form is only validated when
    /// the request sends that same name back. Any other submission returns
    /// `Err(form)` without running `is_valid()`, so no field error is set on a
    /// form the user didn't send.
    pub async fn try_named(mut form: F, request: &Request, name: &'static str) -> Result<Self, F> {
        form.get_form_mut().name = Some(name);
        form.register_dynamic_fields(request).await;

        let validate = match request.method.is_safe() {
            true => form.allow_get(request),
            false => form.allow_post(request),
        };
        let sent = request.prisme.data.get(FORM_NAME_KEY).map(String::as_str) == Some(name);

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
    pub fn database_error(&mut self, err: &sea_orm::DbErr) {
        self.form.database_error(err);
    }
}

impl<F: RuniqueForm> std::ops::Deref for ValidationForm<F> {
    type Target = F;
    fn deref(&self) -> &F {
        &self.form
    }
}
