//! `ValidationForm<F>`: proof by type that a form has passed validation. A form
//! of an entity is written from it (`save`, `delete`); any other action stays
//! in the caller's business logic.
use crate::auth::gate::change::{Secrets, WriteKind, changes, replaced_files};
use crate::auth::gate::refusal::{Refusal, RefusalReason};
use crate::auth::gate::save::save_as;
use crate::auth::gate::write::linked_delete_error;
use crate::auth::permissions::rights::ColumnRights;
use crate::context::Request;
use crate::db::DbWrite;
use crate::forms::FormEntity;
use crate::forms::field::RuniqueForm;
use crate::forms::form::{SaveComportement, write_replacing_uploads, write_with_uploads};
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
/// `{<name>_<pk>}`, where `name` is the one given to
/// [`Request::form`](crate::context::Request::form); without it, `save` creates.
/// A plain `{<pk>}` is not the form's row: read it with
/// [`Request::get_path_as`](crate::context::Request::get_path_as) when the handler needs it. On a route
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
    written(form, result).await
}

/// The form back from its write: `Ok` once written, `Err` with the reason on
/// it — refused fields already carry theirs; otherwise the refusal's message,
/// or the database's error.
async fn written<F: RuniqueForm>(mut form: F, result: Result<(), DbErr>) -> Result<F, F> {
    let Err(err) = result else {
        return Ok(form);
    };
    tracing::warn!(error = %err, "form write failed");
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

impl<F: RuniqueForm> std::ops::Deref for ValidationForm<F> {
    type Target = F;
    fn deref(&self) -> &F {
        &self.form
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::auth::gate::test_support::fixtures::{NoteForm, db, note, pk, staff};
    use crate::db::ADb;
    use crate::forms::field::FormAccess;
    use crate::forms::form::Forms;
    use sea_orm::{EntityTrait, TransactionTrait};

    /// A note's form on a route declared `admin`, for an account with every
    /// right on notes; `target` is the row in the URL.
    fn gated_note(target: Option<&str>) -> NoteForm {
        let mut forms = Forms::new("");
        NoteForm::register_fields(&mut forms);
        forms.save_comportement = SaveComportement::Gated {
            user: staff(pk(10), "notes", [true; 7]),
        };
        if let Some(id) = target {
            forms.set_name("note");
            forms
                .path_params
                .insert(crate::forms::save::row_param("note", "id"), id.to_string());
        }
        NoteForm::from_form(forms)
    }

    async fn note_ids(db: &ADb) -> Vec<i32> {
        let mut ids: Vec<i32> = note::Entity::find()
            .all(&**db)
            .await
            .unwrap()
            .into_iter()
            .map(|n| n.id)
            .collect();
        ids.sort_unstable();
        ids
    }

    /// The gate decides on what it reads through the connection: inside a
    /// transaction it would judge what the handler hasn't committed yet.
    #[tokio::test]
    async fn a_gated_form_refuses_a_transaction_and_writes_nothing() {
        let db = db().await;
        let before = note_ids(&db).await;
        let message = Refusal::new(RefusalReason::InTransaction).message();

        for (action, target) in [
            (SaveAction::Save, None),
            (SaveAction::Save, Some("1")),
            (SaveAction::Delete, Some("1")),
        ] {
            let txn = db.begin().await.unwrap();
            let Err(refused) = gated_write(gated_note(target), action, &txn).await else {
                panic!("refused in a transaction");
            };
            assert_eq!(refused.forms().errors, vec![message.clone()]);
            txn.commit().await.unwrap();
            assert_eq!(note_ids(&db).await, before);
        }
    }

    /// The other side: the same form and account write through the connection,
    /// so the refusal above comes from the transaction alone.
    #[tokio::test]
    async fn a_gated_form_writes_through_the_connection() {
        let db = db().await;
        let before = note_ids(&db).await;

        let created = gated_write(gated_note(None), SaveAction::Save, &db).await;
        assert!(created.is_ok(), "created through the gate");
        assert_eq!(note_ids(&db).await.len(), before.len() + 1);

        let deleted = gated_write(gated_note(Some("1")), SaveAction::Delete, &db).await;
        assert!(deleted.is_ok(), "deleted through the gate");
        assert!(!note_ids(&db).await.contains(&1));
    }

    /// A model with a list field: `genres` has no column on `gate_shelves`,
    /// its values live in `gate_shelves_genres`.
    // `model!` also writes `Shelf::objects`, which this test doesn't use.
    #[cfg(feature = "admin")]
    #[allow(dead_code)]
    mod shelf {
        use runique::prelude::*;

        model! {
            Shelf,
            table: "gate_shelves",
            pk: id => Pk,
            enums: {
                Genre: [Roman, Policier],
            },
            {
                title: text [required],
                genres: checkbox [enum(Genre)],
            }
        }
    }

    #[cfg(feature = "admin")]
    async fn shelf_db() -> ADb {
        use sea_orm::{ConnectionTrait, Schema};
        let conn = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        conn.execute(&schema.create_table_from_entity(shelf::Entity))
            .await
            .unwrap();
        conn.execute(&schema.create_table_from_entity(shelf::genres::Entity))
            .await
            .unwrap();
        // Every gated write leaves its history row in the same transaction.
        conn.execute(&crate::auth::migrations_table::create_eihwaz_history_table())
            .await
            .unwrap();
        ADb::from_connection(conn)
    }

    /// The model's own form, as the admin builds it, written as `user`;
    /// `target` is the row in the URL (none: a create).
    #[cfg(feature = "admin")]
    fn shelf_form(
        user: &crate::auth::current_user::CurrentUser,
        target: Option<&str>,
        values: &[(&str, &str)],
    ) -> shelf::Form {
        let mut forms = Forms::new("");
        <shelf::Form as FormAccess>::register_fields(&mut forms);
        crate::auth::gate::change::type_fields::<shelf::Entity>(&mut forms);
        forms.save_comportement = SaveComportement::Gated { user: user.clone() };
        if let Some(id) = target {
            forms.set_name("shelf");
            forms
                .path_params
                .insert(crate::forms::save::row_param("shelf", "id"), id.to_string());
        }
        for (name, value) in values {
            forms.add_value(name, value);
        }
        <shelf::Form as FormAccess>::from_form(forms)
    }

    /// A dirigeant: every right on every field.
    #[cfg(feature = "admin")]
    fn boss() -> crate::auth::current_user::CurrentUser {
        use crate::auth::gate::test_support::fixtures::as_rank;
        as_rank(32, crate::auth::permissions::rang::Rang::Dirigeant)
    }

    /// An admin who may create and update shelves, on the fields `ticked`
    /// alone (name, create, update).
    #[cfg(feature = "admin")]
    fn delegate(ticked: &[(&str, bool, bool)]) -> crate::auth::current_user::CurrentUser {
        use crate::auth::gate::test_support::fixtures::as_rank;
        use crate::auth::permissions::{ColumnRight, Permission};

        let mut right = Permission::zeroed("gate_shelves".to_string());
        right.can_create = true;
        right.can_read = true;
        right.can_update = true;
        right.colonnes = ticked
            .iter()
            .map(|(name, create, update)| ColumnRight {
                colonne: (*name).to_string(),
                can_read: true,
                can_create: *create,
                can_update: *update,
            })
            .collect();
        let mut user = as_rank(33, crate::auth::permissions::rang::Rang::Admin);
        user.droits = vec![right];
        user
    }

    #[cfg(feature = "admin")]
    async fn write(db: &ADb, form: shelf::Form) {
        if let Err(refused) = gated_write(form, SaveAction::Save, db).await {
            panic!("the gate refused the form: {:?}", refused.forms().errors);
        }
    }

    /// The only shelf of the table, with its genres.
    #[cfg(feature = "admin")]
    async fn the_shelf(db: &ADb) -> (shelf::Model, Vec<shelf::Genre>) {
        let mut rows = shelf::Entity::find().all(&**db).await.unwrap();
        assert_eq!(rows.len(), 1, "one shelf in the table");
        let row = rows.remove(0);
        let genres = row.genres(&**db).await.unwrap();
        (row, genres)
    }

    /// The admin writes a model's form through this path (`save_through_gate`
    /// → `ValidationForm::save` → `gated_write`). The values ticked in a list
    /// field must be written with their row, in the list's own table.
    #[cfg(feature = "admin")]
    #[tokio::test]
    async fn a_gated_form_writes_the_values_of_a_list_field() {
        use shelf::Genre;

        let db = shelf_db().await;
        let form = shelf_form(
            &boss(),
            None,
            &[("title", "Dune"), ("genres", "Roman,Policier")],
        );
        assert_eq!(
            form.forms().fields.get("genres").map(|f| f.value()),
            Some("Roman,Policier"),
            "the form holds both values before the write"
        );
        write(&db, form).await;

        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune");
        assert_eq!(
            genres,
            [Genre::Roman, Genre::Policier],
            "the list ticked in the form is written with its row"
        );

        // What the edit page reads back, by the row's key.
        let read = <shelf::Entity as crate::auth::meta::ModelMeta>::list_values(row.id, &*db)
            .await
            .unwrap();
        assert_eq!(read, [("genres", "Roman,Policier".to_string())]);
    }

    /// On update the list is replaced by what the form holds, even when no
    /// column of the row changes; nothing ticked clears it.
    #[cfg(feature = "admin")]
    #[tokio::test]
    async fn a_gated_form_replaces_then_clears_a_list_on_update() {
        use shelf::Genre;

        let db = shelf_db().await;
        write(
            &db,
            shelf_form(
                &boss(),
                None,
                &[("title", "Dune"), ("genres", "Roman,Policier")],
            ),
        )
        .await;
        let id = the_shelf(&db).await.0.id.to_string();

        // The list alone: no column is set, the row keeps its title.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("genres", "Policier")]),
        )
        .await;
        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune");
        assert_eq!(genres, [Genre::Policier]);

        // A column and the list together.
        write(
            &db,
            shelf_form(
                &boss(),
                Some(&id),
                &[("title", "Dune II"), ("genres", "Roman")],
            ),
        )
        .await;
        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune II");
        assert_eq!(genres, [Genre::Roman]);

        // Nothing ticked: the form holds an empty value, the list is cleared.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("title", "Dune III")]),
        )
        .await;
        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune III");
        assert!(genres.is_empty(), "{genres:?}");
        let read = <shelf::Entity as crate::auth::meta::ModelMeta>::list_values(row.id, &*db)
            .await
            .unwrap();
        assert_eq!(
            read,
            [("genres", String::new())],
            "an empty list reads empty"
        );
    }

    /// The history rows, oldest first: the action and its summary as JSON.
    #[cfg(feature = "admin")]
    async fn history(db: &ADb) -> Vec<(String, Option<serde_json::Value>)> {
        use crate::auth::history;
        use sea_orm::QueryOrder;
        history::Entity::find()
            .order_by_asc(history::Column::Id)
            .all(&**db)
            .await
            .unwrap()
            .into_iter()
            .map(|row| {
                let summary = row
                    .summary
                    .map(|text| serde_json::from_str(&text).expect("a JSON summary"));
                (row.action, summary)
            })
            .collect()
    }

    /// The trace of an update names a list like a column: its values before
    /// and after, only when they differ.
    #[cfg(feature = "admin")]
    #[tokio::test]
    async fn an_update_traces_a_list_only_when_it_changes() {
        use serde_json::json;

        let db = shelf_db().await;
        write(
            &db,
            shelf_form(
                &boss(),
                None,
                &[("title", "Dune"), ("genres", "Roman,Policier")],
            ),
        )
        .await;
        let id = the_shelf(&db).await.0.id.to_string();
        assert_eq!(history(&db).await, [("create".to_string(), None)]);

        // The list alone changes.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("genres", "Policier")]),
        )
        .await;
        let rows = history(&db).await;
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1],
            (
                "update".to_string(),
                Some(json!({ "genres": { "old": "Roman,Policier", "new": "Policier" } }))
            )
        );

        // The same list again, no column: nothing written, nothing traced.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("genres", "Policier")]),
        )
        .await;
        assert_eq!(
            history(&db).await.len(),
            2,
            "an unchanged list leaves no trace"
        );

        // A column changes, the list doesn't: the list stays out of the trace.
        write(
            &db,
            shelf_form(
                &boss(),
                Some(&id),
                &[("title", "Dune II"), ("genres", "Policier")],
            ),
        )
        .await;
        let rows = history(&db).await;
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[2].1,
            Some(json!({ "title": { "old": "Dune", "new": "Dune II" } }))
        );

        // Both change, the list is cleared: both are in the trace.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("title", "Dune III")]),
        )
        .await;
        let rows = history(&db).await;
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows[3].1,
            Some(json!({
                "title": { "old": "Dune II", "new": "Dune III" },
                "genres": { "old": "Policier", "new": "" },
            }))
        );
        assert!(the_shelf(&db).await.1.is_empty());
    }

    /// A list follows the rights of its name, like a column: an admin not
    /// ticked for it writes the row and leaves the list alone.
    #[cfg(feature = "admin")]
    #[tokio::test]
    async fn a_list_is_written_only_with_the_right_on_its_name() {
        use shelf::Genre;
        let values = [("title", "Dune"), ("genres", "Roman,Policier")];

        // Create: `title` ticked, `genres` not.
        let db = shelf_db().await;
        let without = delegate(&[("title", true, true)]);
        write(&db, shelf_form(&without, None, &values)).await;
        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune", "the row is written");
        assert!(genres.is_empty(), "no right on the list: {genres:?}");

        // Update: `genres` ticked for create only is not enough. The list
        // holds a value first, so an unwanted write would show.
        let id = row.id.to_string();
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("genres", "Policier")]),
        )
        .await;
        let create_only = delegate(&[("title", true, true), ("genres", true, false)]);
        write(
            &db,
            shelf_form(
                &create_only,
                Some(&id),
                &[("title", "Dune II"), ("genres", "Roman")],
            ),
        )
        .await;
        let (row, genres) = the_shelf(&db).await;
        assert_eq!(row.title, "Dune II");
        assert_eq!(genres, [Genre::Policier], "create is not update");

        // The other side: ticked for update, the same form writes the list.
        let with = delegate(&[("title", true, true), ("genres", true, true)]);
        write(&db, shelf_form(&with, Some(&id), &[("genres", "Roman")])).await;
        assert_eq!(the_shelf(&db).await.1, [Genre::Roman]);

        // And ticked for create, a new row gets its list.
        let db = shelf_db().await;
        write(&db, shelf_form(&with, None, &values)).await;
        assert_eq!(the_shelf(&db).await.1, [Genre::Roman, Genre::Policier]);
    }

    /// A locked field is shown, never written: a list too, whatever the
    /// request sends for it.
    #[cfg(feature = "admin")]
    #[tokio::test]
    async fn a_locked_list_field_is_not_written() {
        use shelf::Genre;

        let db = shelf_db().await;
        write(
            &db,
            shelf_form(&boss(), None, &[("title", "Dune"), ("genres", "Roman")]),
        )
        .await;
        let id = the_shelf(&db).await.0.id.to_string();

        for readonly in [true, false] {
            let mut form = shelf_form(
                &boss(),
                Some(&id),
                &[("title", "x"), ("genres", "Policier")],
            );
            if readonly {
                form.forms_mut().field_readonly("genres", true);
            } else {
                form.forms_mut().field_disabled("genres", true);
            }
            write(&db, form).await;
            assert_eq!(the_shelf(&db).await.1, [Genre::Roman], "locked: untouched");
        }

        // The other side: unlocked, the same form replaces the list.
        write(
            &db,
            shelf_form(&boss(), Some(&id), &[("genres", "Policier")]),
        )
        .await;
        assert_eq!(the_shelf(&db).await.1, [Genre::Policier]);
    }
}
