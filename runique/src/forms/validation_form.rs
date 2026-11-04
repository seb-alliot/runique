//! `ValidationForm<F>`: proof by type that a form has passed validation. A form
//! of an entity is written from it (`save`, `delete`); any other action stays
//! in the caller's business logic.
use crate::auth::gate::change::{Secrets, WriteKind, changes};
use crate::auth::gate::refusal::{Refusal, RefusalReason};
use crate::auth::gate::save::save_as;
use crate::auth::gate::write::linked_delete_error;
use crate::auth::permissions::rights::ColumnRights;
use crate::context::Request;
use crate::db::DbWrite;
use crate::forms::FormEntity;
use crate::forms::field::RuniqueForm;
use crate::forms::form::{SaveComportement, write_with_uploads};
use crate::forms::save::{SaveAction, not_found, on_fields, url_key};
use sea_orm::sea_query::IntoValueTuple;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, Iterable, PrimaryKeyToColumn,
};

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
        form.register_dynamic_fields(request).await;

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
    pub fn database_error(&mut self, err: &sea_orm::DbErr) {
        self.form.database_error(err);
    }
}

/// Writing a validated form of an entity. The row comes from the URL:
/// `{<name>_<pk>}`, else `{<pk>}`, where `name` is the one given to
/// [`ValidationForm::try_form`]; without one, `save` creates. On a route
/// declared `admin` the gate writes, as the checked account, with its
/// history; elsewhere SeaORM does, and checking that the user may touch that
/// row is the handler's job.
///
/// Both give the form back: `Ok` once written (redirect, or `clear()` it and
/// render again), `Err` with the reason on its fields or its errors.
///
/// `db` is the connection, or a transaction from
/// [`Request::db_atomic`](crate::context::Request::db_atomic) to tie the write
/// to the handler's own: nothing is real before its `commit()`. A file moved by
/// the save stays on disk if that transaction is never committed. A form on a
/// route declared `admin` refuses a transaction: the gate decides on what it
/// reads through the connection.
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
    /// admin: every column the form carries, secret ones included — a password
    /// field only as its hash, an empty one on an update leaving it as it is —
    /// minus the locked fields and the key on an update.
    pub async fn save(self, db: &impl DbWrite) -> Result<F, F> {
        let mut form = self.form;
        // An error set since the validation (`database_error`) stops it too.
        if !form.forms().is_save_allowed() {
            return Err(form);
        }
        if gated(form.forms()) {
            return gated_write(form, SaveAction::Save, db).await;
        }
        let forms = form.forms_mut();
        let result = async {
            let key = url_key::<F::Entity>(forms).map_err(|()| not_found())?;
            let kind = match key {
                Some(_) => WriteKind::Update,
                None => WriteKind::Create,
            };
            let mut model = changes::<F::Entity>(forms, &ColumnRights::All, kind, Secrets::Write)
                .map_err(|e| on_fields(forms, e))?;
            let Some(id) = key else {
                // `insert`, not `save`: a key the form itself carries would
                // turn `save` into an update of that row.
                return write_with_uploads(forms, async {
                    let txn = db.begin().await?;
                    model.insert(&txn).await?;
                    Ok(txn)
                })
                .await;
            };
            if !model.is_changed() {
                return Ok(());
            }
            for (column, value) in
                <F::Entity as EntityTrait>::PrimaryKey::iter().zip(id.into_value_tuple())
            {
                model.set(column.into_column(), value);
            }
            // The model's own hooks write through the same transaction.
            write_with_uploads(forms, async {
                let txn = db.begin().await?;
                match model.save(&txn).await {
                    Ok(_) => Ok(txn),
                    Err(DbErr::RecordNotUpdated) => Err(not_found()),
                    Err(e) => Err(e),
                }
            })
            .await
        }
        .await;
        written(form, result)
    }

    /// Deletes the row the URL points at.
    pub async fn delete(self, db: &impl DbWrite) -> Result<F, F> {
        let mut form = self.form;
        if !form.forms().is_save_allowed() {
            return Err(form);
        }
        if gated(form.forms()) {
            return gated_write(form, SaveAction::Delete, db).await;
        }
        let forms = form.forms_mut();
        let result = async {
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
        .map_err(linked_delete_error);
        written(form, result)
    }
}

fn gated(forms: &crate::forms::Forms) -> bool {
    matches!(forms.save_comportement, SaveComportement::Gated { .. })
}

/// The gate's write, through the connection only.
async fn gated_write<F>(mut form: F, action: SaveAction, db: &impl DbWrite) -> Result<F, F>
where
    F: RuniqueForm + FormEntity,
    <F::Entity as EntityTrait>::ActiveModel: Send,
    <F::Entity as EntityTrait>::Model: IntoActiveModel<<F::Entity as EntityTrait>::ActiveModel>,
{
    let result = match db.as_adb() {
        Some(db) => save_as::<F::Entity>(form.forms_mut(), action, db).await,
        None => Err(DbErr::Custom(
            Refusal::new(RefusalReason::InTransaction).message(),
        )),
    };
    written(form, result)
}

/// The form back from its write: `Ok` once written, `Err` with the reason on
/// it — refused fields already carry theirs; otherwise the refusal's message,
/// or the database's error.
fn written<F: RuniqueForm>(mut form: F, result: Result<(), DbErr>) -> Result<F, F> {
    let Err(err) = result else {
        return Ok(form);
    };
    tracing::warn!(error = %err, "form write failed");
    let forms = form.forms_mut();
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
