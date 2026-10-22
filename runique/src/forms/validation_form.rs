//! `ValidationForm<F>`: proof by type that a form has passed validation, without
//! imposing any action (save/delete/...) — that stays in the caller's business logic.
use crate::context::Request;
use crate::forms::field::RuniqueForm;
use crate::utils::constante::session_key::session::FORM_NAME_KEY;

/// A form that has passed validation. The only way to obtain one is
/// [`ValidationForm::try_form`], so a function taking `ValidationForm<F>` instead
/// of `&mut F` proves at the type level that its caller already validated —
/// no need to re-check `is_valid()` or trust a convention.
pub struct ValidationForm<F: RuniqueForm> {
    form: F,
}

impl<F: RuniqueForm> ValidationForm<F> {
    /// Names the form, registers dynamic fields, then validates it — only when
    /// the request sends this same name back and the HTTP method allows it
    /// (`allow_get`/`allow_post`).
    ///
    /// `name` is rendered as the form's hidden `FORM_NAME_KEY` field, and
    /// prefixes its fields' HTML ids. Every handler that renders the form goes
    /// through here, GET included: an unnamed form fails to render.
    ///
    /// A submission carrying another name (another form on the same page), or
    /// none, returns `Err(form)` without running `is_valid()`: no field error is
    /// set on a form the user didn't send. On a failed validation, `form` comes
    /// back with its field errors so the caller can re-render it.
    pub async fn try_form(mut form: F, request: &Request, name: &'static str) -> Result<Self, F> {
        form.register_dynamic_fields(request).await;
        form.get_form_mut().set_name(name);

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
