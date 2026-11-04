//! The entity a generated form carries (`FormEntity`): the right one for
//! `#[form(schema = …)]`, `model!{}`, `extend!{}` and the framework's own
//! account form, and `objects` / `search!(@Form => …)` working from it.
use crate::helpers::db;
use runique::db::ADb;
use runique::forms::FormEntity;
use std::any::TypeId;

mod blog {
    use runique::prelude::*;

    model! {
        Post,
        table: "fe_posts",
        pk: id => i32,
        {
            title: text [required],
        }
    }
}

mod other {
    use runique::prelude::*;

    model! {
        Note,
        table: "fe_notes",
        pk: id => i32,
        {
            body: text [required],
        }
    }
}

mod groups {
    use runique::prelude::*;

    extend! {
        table: "eihwaz_groupes",
        fields: {
            tagline: text [nullable],
        }
    }
}

mod typed {
    use runique::prelude::*;

    model! {
        Typed,
        table: "fe_typed",
        pk: id => i32,
        {
            title: text [max_length: 20, required],
            views: int [nullable],
        }
    }
}

#[runique::prelude::form(schema = typed, fields = All)]
pub struct TypedForm;
impl runique::forms::RuniqueForm for TypedForm {}

#[runique::prelude::form(schema = blog, fields = All)]
pub struct SchemaForm;
impl runique::forms::RuniqueForm for SchemaForm {}

#[runique::prelude::form(schema = blog, fields = [title])]
pub struct PartialSchemaForm;
impl runique::forms::RuniqueForm for PartialSchemaForm {}

fn entity_of<F: FormEntity<Entity = E>, E>() {}

fn is_entity<F: FormEntity, E: 'static>() -> bool
where
    F::Entity: 'static,
{
    TypeId::of::<F::Entity>() == TypeId::of::<E>()
}

#[test]
fn a_schema_form_carries_the_entity_of_its_schema() {
    entity_of::<SchemaForm, blog::Entity>();
    entity_of::<PartialSchemaForm, blog::Entity>();
    assert!(is_entity::<SchemaForm, blog::Entity>());
    assert!(!is_entity::<SchemaForm, other::Entity>());
}

#[test]
fn a_model_form_carries_the_entity_of_its_module() {
    entity_of::<blog::PostModelForm, blog::Entity>();
    entity_of::<other::NoteModelForm, other::Entity>();
    assert!(is_entity::<blog::PostModelForm, blog::Entity>());
    assert!(!is_entity::<blog::PostModelForm, other::Entity>());
    assert!(!is_entity::<other::NoteModelForm, blog::Entity>());
}

#[test]
fn an_extend_form_carries_the_entity_of_its_module() {
    entity_of::<groups::EihwazGroupesModelForm, groups::Entity>();
    assert!(!is_entity::<groups::EihwazGroupesModelForm, blog::Entity>());
}

#[test]
fn the_accounts_form_carries_the_users_entity() {
    entity_of::<runique::auth::form::UserModelForm, runique::auth::user::Entity>();
    assert!(!is_entity::<runique::auth::form::UserModelForm, blog::Entity>());
}

const DDL: &str = "CREATE TABLE fe_posts (id INTEGER PRIMARY KEY, title TEXT NOT NULL)";

async fn seeded() -> ADb {
    let conn = db::fresh_db_with_schema(DDL).await;
    db::exec(&conn, "INSERT INTO fe_posts (id, title) VALUES (1, 'a')").await;
    db::exec(&conn, "INSERT INTO fe_posts (id, title) VALUES (2, 'b')").await;
    db::exec(&conn, "INSERT INTO fe_posts (id, title) VALUES (3, 'c')").await;
    ADb::from_connection(conn)
}

#[tokio::test]
async fn objects_of_a_schema_form_reads_its_entity_without_a_model_attribute() {
    let adb = seeded().await;
    assert_eq!(SchemaForm::objects.get(&adb, 2).await.unwrap().title, "b");
    assert_eq!(PartialSchemaForm::objects.count(&adb).await.unwrap(), 3);
    assert!(SchemaForm::objects.get(&adb, 99).await.is_err());
    assert!(
        SchemaForm::objects
            .get_optional(&adb, 99)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn search_on_a_schema_form_returns_the_matching_row_only() {
    let adb = seeded().await;
    let rows = runique::search!(@SchemaForm => Title eq "b")
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 2);
    assert_eq!(rows[0].title, "b");

    let none = runique::search!(@SchemaForm => Title eq "absent")
        .all(&adb)
        .await
        .unwrap();
    assert!(none.is_empty());

    let not_b = runique::search!(@PartialSchemaForm => ! Title eq "b")
        .all(&adb)
        .await
        .unwrap();
    assert_eq!(not_b.len(), 2);
    assert!(not_b.iter().all(|r| r.title != "b"));
}

/// Each field of a schema form knows its column's type, as a handler gets the
/// form (`request.form()`, then `try_form`): the type `is_valid` checks a
/// value against before the write, and the one `cleaned` could check against.
#[cfg(feature = "test-utils")]
#[tokio::test]
async fn a_schema_form_types_each_field_like_its_column() {
    use crate::helpers::{handler_req::build_handler_req, server::build_engine};
    use runique::forms::field::FormAccess;
    use runique::prelude::ValidationForm;
    use runique::sea_orm::ColumnTrait;

    let request = build_handler_req(
        build_engine().await,
        None,
        std::collections::HashMap::new(),
        axum::http::Method::GET,
    )
    .await;
    let columns = [
        ("title", typed::Column::Title),
        ("views", typed::Column::Views),
    ];

    let form: TypedForm = request.form("typed");
    for (name, column) in &columns {
        assert_eq!(
            form.forms().fields[*name].column_type(),
            Some(column.def().get_column_type()),
            "after request.form(): {name}"
        );
    }

    let form = ValidationForm::try_form(form, &request)
        .await
        .err()
        .expect("a bare GET never validates");
    for (name, column) in &columns {
        assert_eq!(
            form.forms().fields[*name].column_type(),
            Some(column.def().get_column_type()),
            "after try_form: {name}"
        );
    }
}

/// `cleaned` reads through the field's column: `T` must be that column's Rust
/// type, whatever the value — an `i64` or `i16` on an `i32` column is `None`
/// even for a value both could hold. A text column of any length reads as
/// `String`, and a value from the URL reads the same way as one posted.
#[cfg(feature = "test-utils")]
#[tokio::test]
async fn cleaned_reads_through_the_field_column() {
    use crate::helpers::{handler_req::build_handler_req, server::build_engine};
    use runique::forms::{RuniqueForm, field::FormAccess};
    use std::collections::HashMap;

    let body = HashMap::from([
        (
            runique::utils::constante::session_key::session::FORM_NAME_KEY.to_string(),
            "typed".to_string(),
        ),
        ("title".to_string(), "abc".to_string()),
        ("views".to_string(), "42".to_string()),
    ]);
    let request =
        build_handler_req(build_engine().await, None, body, axum::http::Method::POST).await;
    let form: TypedForm = request.form("typed");

    assert_eq!(form.cleaned::<i32>("views"), Some(42));
    assert_eq!(form.cleaned::<i64>("views"), None, "an i32 column");
    assert_eq!(form.cleaned::<i16>("views"), None, "an i32 column");
    assert_eq!(form.cleaned::<String>("views"), None, "an i32 column");
    assert_eq!(form.cleaned::<String>("title"), Some("abc".to_string()));
    assert_eq!(form.cleaned::<i32>("title"), None, "a text column");

    // Nothing posted: the URL's value, read through the same column.
    let request = build_handler_req(
        build_engine().await,
        None,
        HashMap::new(),
        axum::http::Method::GET,
    )
    .await;
    let mut form: TypedForm = request.form("typed");
    form.forms_mut().set_url_params(
        &HashMap::new(),
        &HashMap::from([("views".to_string(), "7".to_string())]),
    );
    assert_eq!(form.cleaned::<i32>("views"), Some(7));
    assert_eq!(form.cleaned::<i64>("views"), None);
}
