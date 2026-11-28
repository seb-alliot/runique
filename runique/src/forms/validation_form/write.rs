//! `save` and `delete` of a validated form: through the gate on a route
//! declared `admin`, through SeaORM elsewhere.
use super::ValidationForm;
use crate::{
    auth::{
        gate::{
            change::{RowWrite, Secrets, changes, replaced_files},
            refusal::{Refusal, RefusalReason},
            save::save_as,
            write::linked_delete_error,
        },
        permissions::rights::ColumnRights,
    },
    db::DbWrite,
    forms::{
        FormEntity, Forms,
        data::save::{SaveAction, not_found, on_fields, url_key},
        form::{SaveMode, write_replacing_uploads, write_with_uploads},
        traits::runique_form::RuniqueForm,
    },
};
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, Iterable, PrimaryKeyToColumn,
    sea_query::IntoValueTuple,
};

/// Writing a validated form of an entity. The row comes from the URL:
/// `{<name>_<pk>}`, where `name` is the one given to
/// [`Request::form`](crate::http::Request::form); without it, `save` creates.
/// A plain `{<pk>}` is not the form's row: read it with
/// [`Request::get_path_as`](crate::http::Request::get_path_as) when the handler needs it. On a route
/// declared `admin` the gate writes, as the checked account, with its
/// history; elsewhere SeaORM does, and checking that the user may touch that
/// row is the handler's job.
///
/// Both give the form back: `Ok` once written (redirect, or `clear()` it and
/// render again), `Err` with the reason on its fields or its errors.
///
/// `db` is the connection, or a transaction from
/// [`Request::db_atomic`](crate::http::Request::db_atomic) to tie the write
/// to the handler's own: nothing is real before its `commit()`. A file moved by
/// the save stays on disk if that transaction is never committed. A form on a
/// route declared `admin` refuses a transaction: the gate decides on what it
/// reads through the connection.
///
/// ```rust,ignore
/// let form = request.form::<ArticleForm>("article");
/// match ValidationForm::try_form(form, &request).await {
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
                Some(_) => RowWrite::Update,
                None => RowWrite::Create,
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
                <F::Entity as EntityTrait>::PrimaryKey::iter().zip(id.clone().into_value_tuple())
            {
                model.set(column.into_column(), value);
            }
            // Read only for a field declared with `.replace()` that is saving
            // an upload: what the row held is the file to remove.
            let replaces = forms.awaits_replaced_file();
            // The model's own hooks write through the same transaction.
            write_replacing_uploads(forms, async {
                let txn = db.begin().await?;
                let replaced = match replaces {
                    true => <F::Entity as EntityTrait>::find_by_id(id)
                        .one(&txn)
                        .await?
                        .map(|old| replaced_files::<F::Entity>(&old, &model))
                        .unwrap_or_default(),
                    false => Vec::new(),
                };
                match model.save(&txn).await {
                    Ok(_) => Ok((txn, replaced)),
                    Err(DbErr::RecordNotUpdated) => Err(not_found()),
                    Err(e) => Err(e),
                }
            })
            .await
        }
        .await;
        written(form, result).await
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
        written(form, result).await
    }
}

fn gated(forms: &Forms) -> bool {
    matches!(forms.save_mode, SaveMode::Gated { .. })
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
    written(form, result).await
}

/// The form back from its write: `Ok` once written, `Err` with the reason on
/// it — refused fields already carry theirs; otherwise the refusal's message,
/// or the database's error.
async fn written<F: RuniqueForm>(mut form: F, result: Result<(), DbErr>) -> Result<F, F> {
    let Err(err) = result else {
        return Ok(form);
    };
    tracing::warn!(error = %err, "{}", crate::translation::t(runique_i18n::keys::forms::log::FORM_WRITE_FAILED));
    let forms = form.forms_mut();
    // A refusal before the write (unreadable key, refused column) never went
    // through the upload's own rollback: the field would keep the destination
    // of a file that was never moved.
    forms.rollback_uploads().await;
    if !forms.has_errors() {
        match &err {
            DbErr::Custom(message) => forms.errors.push(message.clone()),
            other => forms.database_error(other),
        }
    }
    Err(form)
}

#[cfg(all(test, feature = "sqlite"))]
#[path = "tests.rs"]
mod tests;
