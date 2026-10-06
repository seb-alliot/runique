//! `extend!{}` fields go through the same generators as `model!{}` fields.
//! Regression: it had its own conversion, which put a `String` into every
//! field it didn't know (`i16`, `f32`, `timestamp`, `timestamp_tz`…), so an
//! `extend!{}` using one of them didn't compile; its integers had no bounds.
use runique::forms::Forms;
use runique::forms::field::RuniqueForm;
use runique::utils::aliases::StrMap;

mod groups {
    use runique::prelude::*;

    extend! {
        table: "eihwaz_groupes",
        fields: {
            rank: i16 [required],
            weight: f32 [nullable],
            seen_at: timestamp [nullable],
            seen_tz: timestamp_tz [nullable],
            code: char [nullable],
        }
    }
}

#[tokio::test]
async fn extend_integers_are_held_to_their_type() {
    let mut form = Forms::new("csrf");
    groups::EihwazGroupesAdminForm::register_fields(&mut form);
    let field = form.fields.get_mut("rank").expect("rank registered");
    field.set_value("40000");
    assert!(!field.validate().await, "40000 doesn't fit an i16");
    field.set_value("12");
    assert!(field.validate().await);
}

#[test]
fn extend_converts_every_declared_type() {
    let mut data = StrMap::new();
    data.insert("rank".into(), "12".into());
    data.insert("weight".into(), "1.5".into());
    data.insert("seen_tz".into(), "2026-09-30T14:30".into());
    let model = groups::admin_from_form(&data, None).expect("every value converts");
    assert_eq!(model.rank.clone().unwrap(), 12i16);
    assert_eq!(model.weight.clone().unwrap(), Some(1.5f32));
    assert!(model.seen_tz.clone().unwrap().is_some());
}

// The admin code generated for a resource calls these on every model, an
// `extend!{}` included (it has no list fields: they do nothing).
#[test]
fn an_extend_model_has_the_admin_list_methods() {
    let _save = groups::Model::admin_save_lists::<runique::sea_orm::DatabaseConnection>;
    let _read = groups::Model::admin_list_values::<runique::sea_orm::DatabaseConnection>;
}
