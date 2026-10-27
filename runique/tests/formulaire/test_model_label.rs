//! `meta: { label: champ }` of `model!{}`: the column a row is known by in the
//! history (`ModelMeta::LABEL`), none when the model does not say.
use runique::auth::meta::ModelMeta;
use runique::sea_orm::IdenStatic;

mod labelled {
    use runique::prelude::*;

    model! {
        Article,
        table: "label_articles",
        pk: id => i32,
        {
            titre: text [required],
            body: text,
        },
        meta: { label: titre }
    }
}

mod labelled_last_of_many {
    use runique::prelude::*;

    model! {
        Memo,
        table: "label_memos",
        pk: id => i32,
        {
            sujet: text [required],
        },
        meta: { ordering: [-id], label: sujet }
    }
}

mod unlabelled {
    use runique::prelude::*;

    model! {
        Plain,
        table: "label_plains",
        pk: id => i32,
        {
            titre: text [required],
        }
    }
}

mod meta_without_label {
    use runique::prelude::*;

    model! {
        Other,
        table: "label_others",
        pk: id => i32,
        {
            titre: text [required],
        },
        meta: { ordering: [-id] }
    }
}

#[test]
fn a_model_with_a_label_names_its_column() {
    let column = <labelled::Entity as ModelMeta>::LABEL.expect("a label");
    assert_eq!(column.as_str(), "titre");
    assert_ne!(column.as_str(), "body");
}

#[test]
fn the_label_is_kept_beside_other_meta_keys() {
    let column = <labelled_last_of_many::Entity as ModelMeta>::LABEL.expect("a label");
    assert_eq!(column.as_str(), "sujet");
}

#[test]
fn a_model_without_a_label_has_none() {
    assert!(<unlabelled::Entity as ModelMeta>::LABEL.is_none());
    assert!(<meta_without_label::Entity as ModelMeta>::LABEL.is_none());
}

#[test]
fn the_label_does_not_change_the_other_rules() {
    assert_eq!(<labelled::Entity as ModelMeta>::KEY, "label_articles");
    assert!(<labelled::Entity as ModelMeta>::OWNER.is_none());
    assert!(<labelled::Entity as ModelMeta>::SECRET.is_empty());
}

#[test]
fn the_framework_users_are_known_by_their_username() {
    let column = <runique::auth::user::Entity as ModelMeta>::LABEL.expect("a label");
    assert_eq!(column.as_str(), "username");
}
