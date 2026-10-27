//! `ValidationForm<F>`: proof by type that a form has passed validation. A form
//! of an entity is written from it (`save`, `delete`); any other action stays
//! in the caller's business logic.
use crate::auth::gate::change::{WriteKind, changes};
use crate::auth::gate::save::save_as;
use crate::auth::permissions::rights::ColumnRights;
use crate::context::Request;
use crate::forms::FormEntity;
use crate::forms::field::RuniqueForm;
use crate::forms::form::SaveComportement;
use crate::forms::save::{SaveAction, not_found, on_fields, url_key};
use crate::utils::aliases::ADb;
use crate::utils::constante::session_key::session::FORM_NAME_KEY;
use sea_orm::sea_query::IntoValueTuple;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, Iterable, PrimaryKeyToColumn,
    TransactionTrait,
};

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

/// Writing a validated form of an entity. The row comes from the URL:
/// `{<name>_<pk>}`, else `{<pk>}`, where `name` is the one given to
/// [`ValidationForm::try_form`]; without one, `save` creates. Inside the
/// admin's constraint the gate writes, as the checked account, with its
/// history; elsewhere SeaORM does, and checking that the user may touch that
/// row is the handler's job.
///
/// Both give the form back: `Ok` once written (redirect, or `clear()` it and
/// render again), `Err` with the reason on its fields or its errors.
///
/// ```rust,ignore
/// match ValidationForm::try_form(form, &request, "article").await {
///     Ok(form) => match form.save(&request.engine.db).await {
///         Ok(_) => Ok(Redirect::to("/articles").into_response()),
///         Err(form) => { context_update!(request => { "form" => &form }); request.render(template) }
///     },
///     Err(form) => { context_update!(request => { "form" => &form }); request.render(template) }
/// }
/// ```
impl<F> ValidationForm<F>
where
    F: RuniqueForm + FormEntity,
    <F::Entity as EntityTrait>::ActiveModel: Send,
    <F::Entity as EntityTrait>::Model: IntoActiveModel<<F::Entity as EntityTrait>::ActiveModel>,
{
    /// Creates the row, or updates the one the URL points at. Outside the
    /// admin: every column the form carries, minus Runique's floor (secrets,
    /// locked fields, the key on an update).
    pub async fn save(self, db: &ADb) -> Result<F, F> {
        let mut form = self.form;
        // An error set since the validation (`database_error`) stops it too.
        if !form.get_form().is_save_allowed() {
            return Err(form);
        }
        let forms = form.get_form_mut();
        let result = if matches!(forms.save_comportement, SaveComportement::Gated { .. }) {
            save_as::<F::Entity>(forms, SaveAction::Save, db).await
        } else {
            async {
                let key = url_key::<F::Entity>(forms).map_err(|()| not_found())?;
                let kind = match key {
                    Some(_) => WriteKind::Update,
                    None => WriteKind::Create,
                };
                let mut model = changes::<F::Entity>(forms, &ColumnRights::All, kind)
                    .map_err(|e| on_fields(forms, e))?;
                let Some(id) = key else {
                    // `insert`, not `save`: a key the form itself carries would
                    // turn `save` into an update of that row.
                    let txn = db.begin().await?;
                    model.insert(&txn).await?;
                    return txn.commit().await;
                };
                if !model.is_changed() {
                    return Ok(());
                }
                for (column, value) in <F::Entity as EntityTrait>::PrimaryKey::iter()
                    .zip(id.into_value_tuple())
                {
                    model.set(column.into_column(), value);
                }
                // The model's own hooks write through the same transaction.
                let txn = db.begin().await?;
                match model.save(&txn).await {
                    Ok(_) => txn.commit().await,
                    Err(DbErr::RecordNotUpdated) => Err(not_found()),
                    Err(e) => Err(e),
                }
            }
            .await
        };
        written(form, result)
    }

    /// Deletes the row the URL points at.
    pub async fn delete(self, db: &ADb) -> Result<F, F> {
        let mut form = self.form;
        if !form.get_form().is_save_allowed() {
            return Err(form);
        }
        let forms = form.get_form_mut();
        let result = if matches!(forms.save_comportement, SaveComportement::Gated { .. }) {
            save_as::<F::Entity>(forms, SaveAction::Delete, db).await
        } else {
            async {
                let Ok(Some(id)) = url_key::<F::Entity>(forms) else {
                    return Err(not_found());
                };
                let txn = db.begin().await?;
                let deleted = <F::Entity as EntityTrait>::delete_by_id(id)
                    .exec(&txn)
                    .await?;
                if deleted.rows_affected == 0 {
                    return Err(not_found());
                }
                txn.commit().await
            }
            .await
        };
        written(form, result)
    }
}

/// The form back from its write: `Ok` once written, `Err` with the reason on
/// it — refused fields already carry theirs; otherwise the refusal's message,
/// or the database's error.
fn written<F: RuniqueForm>(mut form: F, result: Result<(), DbErr>) -> Result<F, F> {
    let Err(err) = result else {
        return Ok(form);
    };
    tracing::warn!(error = %err, "form write failed");
    let forms = form.get_form_mut();
    if !forms.has_errors() {
        match &err {
            DbErr::Custom(message) => forms.errors.push(message.clone()),
            other => forms.database_error(other),
        }
    }
    Err(form)
}

impl<F: RuniqueForm> std::ops::Deref for ValidationForm<F> {
    type Target = F;
    fn deref(&self) -> &F {
        &self.form
    }
}
