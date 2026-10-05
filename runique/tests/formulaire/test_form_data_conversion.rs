//! `admin_from_form` / `admin_partial_update`: form data into the entity.
//! A value that can't be read is an error the admin shows on the form — it
//! used to be replaced by a default (`0`, empty, unchanged) without a word.
use runique::forms::FormDataError;
use runique::sea_orm::ActiveValue;
use runique::utils::aliases::StrMap;

mod items {
    use runique::prelude::*;

    model! {
        Item,
        table: "conv_items",
        pk: id => i32,
        {
            qty: int [required],
            note: int [nullable],
            title: text [required],
            secret: password [nullable],
            photo: image [nullable, upload_to: "photos/"],
            active: bool [required],
        }
    }
}

fn data(pairs: &[(&str, &str)]) -> StrMap {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn an_unreadable_value_is_an_error_not_a_default() {
    let err = items::admin_from_form(&data(&[("qty", "abc"), ("title", "t")]), None).unwrap_err();
    assert_eq!(err, FormDataError::Invalid("qty".into()));
}

#[test]
fn a_missing_required_number_is_an_error_not_zero() {
    let err = items::admin_from_form(&data(&[("title", "t")]), None).unwrap_err();
    assert_eq!(err, FormDataError::Required("qty".into()));
}

#[test]
fn optional_and_text_and_bool_keep_their_empty_meaning() {
    let model = items::admin_from_form(&data(&[("qty", "3"), ("title", "")]), None).unwrap();
    assert_eq!(model.qty, ActiveValue::Set(3));
    assert_eq!(model.note, ActiveValue::Set(None), "nullable: NULL");
    assert_eq!(model.title, ActiveValue::Set(String::new()));
    assert_eq!(
        model.active,
        ActiveValue::Set(false),
        "unchecked box isn't sent"
    );
}

#[test]
fn nothing_uploaded_or_typed_leaves_the_stored_value() {
    let model = items::admin_from_form(&data(&[("qty", "3"), ("secret", ""), ("photo", "")]), None)
        .unwrap();
    assert!(
        matches!(model.secret, ActiveValue::NotSet),
        "no new password typed"
    );
    assert!(matches!(model.photo, ActiveValue::NotSet), "no new upload");
}

#[test]
fn a_password_is_hashed_and_a_hash_kept() {
    let typed =
        items::admin_from_form(&data(&[("qty", "1"), ("secret", "hunter2")]), None).unwrap();
    let ActiveValue::Set(Some(stored)) = typed.secret else {
        panic!("password set");
    };
    assert!(stored.starts_with("$argon2"), "hashed: {stored}");

    let hashed = items::admin_from_form(&data(&[("qty", "1"), ("secret", &stored)]), None).unwrap();
    assert_eq!(
        hashed.secret,
        ActiveValue::Set(Some(stored)),
        "already a hash: kept"
    );
}

#[test]
fn partial_update_only_touches_what_is_sent() {
    let model = items::admin_partial_update(&data(&[("note", "7")]), 1).unwrap();
    assert_eq!(model.note, ActiveValue::Set(Some(7)));
    assert!(matches!(model.qty, ActiveValue::NotSet));
    assert!(matches!(model.title, ActiveValue::NotSet));

    let err = items::admin_partial_update(&data(&[("qty", "x")]), 1).unwrap_err();
    assert_eq!(err, FormDataError::Invalid("qty".into()));
}
