use super::*;
use crate::{
    auth::gate::test_support::fixtures::{NoteForm, db, note, pk, staff},
    db::ADb,
    forms::{form::Forms, traits::runique_form::FormAccess},
};
use sea_orm::{EntityTrait, TransactionTrait};

/// A note's form on a route declared `admin`, for an account with every
/// right on notes; `target` is the row in the URL.
fn gated_note(target: Option<&str>) -> NoteForm {
    let mut forms = Forms::new("");
    NoteForm::register_fields(&mut forms);
    forms.save_mode = SaveMode::Gated {
        user: staff(pk(10), "notes", [true; 7]),
    };
    if let Some(id) = target {
        forms.set_name("note");
        forms.path_params.insert(
            crate::forms::data::save::row_param("note", "id"),
            id.to_string(),
        );
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
    forms.save_mode = SaveMode::Gated { user: user.clone() };
    if let Some(id) = target {
        forms.set_name("shelf");
        forms.path_params.insert(
            crate::forms::data::save::row_param("shelf", "id"),
            id.to_string(),
        );
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
    as_rank(32, crate::auth::permissions::rank::Rang::Dirigeant)
}

/// An admin who may create and update shelves, on the fields `ticked`
/// alone (name, create, update).
#[cfg(feature = "admin")]
fn delegate(ticked: &[(&str, bool, bool)]) -> crate::auth::current_user::CurrentUser {
    use crate::auth::{
        gate::test_support::fixtures::as_rank,
        permissions::{ColumnRight, Permission},
    };

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
    let mut user = as_rank(33, crate::auth::permissions::rank::Rang::Admin);
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
