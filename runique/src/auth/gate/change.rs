//! The columns a gated write sets, read from a validated form: Runique's
//! floor first, then the account's rights, then each field's own value.
use crate::auth::meta::ModelMeta;
use crate::auth::permissions::rights::ColumnRights;
use crate::forms::column_value::null_value;
use crate::forms::form::Forms;
use crate::utils::trad::t;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IdenStatic, Iterable, PrimaryKeyToColumn,
};

/// Creating the row or updating it: the rights and the floor differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WriteKind {
    Create,
    Update,
}

/// The fields the write refused, with the reason, to show on the form.
pub(crate) type FieldErrors = Vec<(String, String)>;

/// The `ActiveModel` of `E` that `form` may write as `kind`. A column stays
/// `NotSet` when it is secret, the key of an updated row, not allowed by
/// `rights`, or not a field of the form.
pub(crate) fn changes<E: ModelMeta>(
    form: &Forms,
    rights: &ColumnRights,
    kind: WriteKind,
) -> Result<E::ActiveModel, FieldErrors> {
    let mut model = <E::ActiveModel as ActiveModelTrait>::default();
    let mut errors = FieldErrors::new();
    for column in E::Column::iter() {
        let name = column.as_str();
        if E::SECRET.iter().any(|c| c.as_str() == name)
            || (kind == WriteKind::Update && is_primary_key::<E>(name))
        {
            continue;
        }
        let allowed = match kind {
            WriteKind::Create => rights.can_create(name),
            WriteKind::Update => rights.can_update(name),
        };
        let Some(field) = form.fields.get(name) else {
            continue;
        };
        if !allowed || field.is_locked() {
            continue;
        }

        let def = column.def();
        let value = match field.to_value() {
            Ok(Some(value)) => value,
            Ok(None) if field.keeps_when_empty() => continue,
            Ok(None) if def.is_null() => match null_value(def.get_column_type()) {
                Some(null) => null,
                None => {
                    errors.push((
                        name.to_string(),
                        t("forms.value_unconvertible").into_owned(),
                    ));
                    continue;
                }
            },
            // Not nullable: the database keeps its value, or applies its
            // default, or refuses the insert.
            Ok(None) => continue,

            Err(message) => {
                errors.push((name.to_string(), message));
                continue;
            }
        };
        if model.try_set(column, value).is_err() {
            errors.push((
                name.to_string(),
                t("forms.value_unconvertible").into_owned(),
            ));
        }
    }

    if errors.is_empty() {
        Ok(model)
    } else {
        Err(errors)
    }
}

fn is_primary_key<E: EntityTrait>(name: &str) -> bool {
    E::PrimaryKey::iter().any(|key| key.into_column().as_str() == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::permissions::ColumnRight;
    use crate::forms::fields::{FileField, NumericField, TextField};
    use chrono::NaiveDate;
    use sea_orm::{ActiveValue, Value};

    mod item {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "chg_items")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub id: i32,
            pub secret: String,
            pub nick: Option<String>,
            pub score: Option<i32>,
            pub born: Option<Date>,
            pub title: String,
            pub pass: Option<String>,
            pub avatar: Option<String>,
            // A column type no field fills: its NULL can't be typed.
            #[sea_orm(column_type = "Inet")]
            pub addr: Option<String>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[derive(serde::Serialize, Debug, Clone)]
    #[serde(transparent)]
    pub struct ItemForm {
        form: Forms,
    }

    impl crate::forms::field::FormAccess for ItemForm {
        fn register_fields(_form: &mut Forms) {}
        crate::impl_form_access!();
    }

    impl crate::forms::field::RuniqueForm for ItemForm {}

    impl ModelMeta for item::Entity {
        const KEY: &'static str = "chg_items";
        type Form = ItemForm;
        const SECRET: &'static [item::Column] = &[item::Column::Secret];
        fn schema() -> crate::migration::schema::ModelSchema {
            crate::migration::ModelSchema::new("Item")
                .table_name("chg_items")
                .primary_key(crate::migration::PrimaryKeyDef::new("id").i32())
                .build()
                .unwrap()
        }
    }

    use item::Column as C;
    use item::Entity as E;

    /// Every column of the entity as a field, typed like its column.
    fn full_form() -> Forms {
        let mut form = Forms::new("");
        form.field(&NumericField::integer("id"));
        form.field(&TextField::text("secret"));
        form.field(&TextField::text("nick"));
        form.field(&NumericField::integer("score"));
        form.field(&TextField::text("born"));
        form.field(&TextField::text("title"));
        form.field(&TextField::password("pass"));
        form.field(&FileField::image("avatar"));
        type_fields(&mut form);
        form
    }

    fn type_fields(form: &mut Forms) {
        for column in <C as sea_orm::Iterable>::iter() {
            let name = sea_orm::IdenStatic::as_str(&column);
            if let Some(field) = form.fields.get_mut(name) {
                field.apply_column_type(sea_orm::ColumnTrait::def(&column).get_column_type());
            }
        }
    }

    fn with(mut form: Forms, values: &[(&str, &str)]) -> Forms {
        for (name, value) in values {
            form.add_value(name, value);
        }
        form
    }

    fn only(ticks: &[(&str, bool, bool)]) -> ColumnRights {
        ColumnRights::Only(
            ticks
                .iter()
                .map(|(name, create, update)| ColumnRight {
                    colonne: (*name).into(),
                    can_read: true,
                    can_create: *create,
                    can_update: *update,
                })
                .collect(),
        )
    }

    fn built(
        form: &Forms,
        rights: &ColumnRights,
        kind: WriteKind,
    ) -> <E as EntityTrait>::ActiveModel {
        changes::<E>(form, rights, kind).unwrap()
    }

    /// The value set on `column`, `None` while it is `NotSet`.
    fn set(model: &item::ActiveModel, column: C) -> Option<Value> {
        match model.get(column) {
            ActiveValue::Set(v) => Some(v),
            _ => None,
        }
    }

    fn text(v: &str) -> Option<Value> {
        Some(Value::from(v.to_string()))
    }

    fn unconvertible() -> String {
        t("forms.value_unconvertible").into_owned()
    }

    #[test]
    fn a_ticked_column_is_set_to_the_value_of_its_field() {
        let form = with(full_form(), &[("title", "Hello"), ("score", "42")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Title), text("Hello"));
        assert_eq!(set(&model, C::Score), Some(Value::Int(Some(42))));
    }

    #[test]
    fn a_date_is_read_as_a_date() {
        let form = with(full_form(), &[("born", "2001-02-03")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(
            set(&model, C::Born),
            Some(Value::from(NaiveDate::from_ymd_opt(2001, 2, 3).unwrap()))
        );
    }

    #[test]
    fn a_secret_column_is_never_set_even_with_every_right_and_a_value() {
        let form = with(full_form(), &[("secret", "hunter2"), ("title", "t")]);
        for kind in [WriteKind::Create, WriteKind::Update] {
            let model = built(&form, &ColumnRights::All, kind);
            assert_eq!(set(&model, C::Secret), None);
            assert_eq!(set(&model, C::Title), text("t"));
        }
    }

    #[test]
    fn a_secret_column_is_not_set_when_ticked_either() {
        let rights = only(&[("secret", true, true), ("title", true, true)]);
        let form = with(full_form(), &[("secret", "hunter2"), ("title", "t")]);
        let model = built(&form, &rights, WriteKind::Create);
        assert_eq!(set(&model, C::Secret), None);
        assert_eq!(set(&model, C::Title), text("t"));
    }

    #[test]
    fn the_key_is_set_on_create_when_the_field_and_the_right_are_there() {
        let form = with(full_form(), &[("id", "7")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Id), Some(Value::Int(Some(7))));
    }

    #[test]
    fn the_key_is_ignored_on_update_even_with_every_right_and_a_value() {
        let form = with(full_form(), &[("id", "7"), ("title", "t")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Id), None);
        assert_eq!(set(&model, C::Title), text("t"));
    }

    #[test]
    fn the_key_on_create_follows_the_create_right() {
        let form = with(full_form(), &[("id", "7")]);
        let denied = only(&[("id", false, true)]);
        assert_eq!(set(&built(&form, &denied, WriteKind::Create), C::Id), None);
        let granted = only(&[("id", true, false)]);
        assert_eq!(
            set(&built(&form, &granted, WriteKind::Create), C::Id),
            Some(Value::Int(Some(7)))
        );
    }

    #[test]
    fn an_empty_key_on_create_is_left_to_the_database() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Id), None);
    }

    #[test]
    fn a_column_ticked_for_create_only_is_set_on_create_and_ignored_on_update() {
        let rights = only(&[("title", true, false)]);
        let form = with(full_form(), &[("title", "t")]);
        assert_eq!(
            set(&built(&form, &rights, WriteKind::Create), C::Title),
            text("t")
        );
        assert_eq!(
            set(&built(&form, &rights, WriteKind::Update), C::Title),
            None
        );
    }

    #[test]
    fn a_column_ticked_for_update_only_is_set_on_update_and_ignored_on_create() {
        let rights = only(&[("title", false, true)]);
        let form = with(full_form(), &[("title", "t")]);
        assert_eq!(
            set(&built(&form, &rights, WriteKind::Create), C::Title),
            None
        );
        assert_eq!(
            set(&built(&form, &rights, WriteKind::Update), C::Title),
            text("t")
        );
    }

    #[test]
    fn only_the_ticked_columns_are_set() {
        let rights = only(&[("title", true, true)]);
        let form = with(
            full_form(),
            &[("title", "t"), ("nick", "n"), ("score", "5")],
        );
        let model = built(&form, &rights, WriteKind::Update);
        assert_eq!(set(&model, C::Title), text("t"));
        assert_eq!(set(&model, C::Nick), None);
        assert_eq!(set(&model, C::Score), None);
    }

    #[test]
    fn no_ticked_column_sets_nothing() {
        let rights = ColumnRights::Only(Vec::new());
        let form = with(full_form(), &[("title", "t")]);
        let model = built(&form, &rights, WriteKind::Create);
        assert_eq!(set(&model, C::Title), None);
    }

    #[test]
    fn a_column_without_a_field_stays_not_set() {
        let mut form = Forms::new("");
        form.field(&TextField::text("title"));
        let form = with(form, &[("title", "t")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Title), text("t"));
        assert_eq!(set(&model, C::Nick), None);
        assert_eq!(set(&model, C::Score), None);
    }

    #[test]
    fn a_readonly_field_is_not_set_even_with_a_value() {
        let mut form = with(full_form(), &[("title", "forged"), ("nick", "n")]);
        form.field_readonly("title", true);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Title), None);
        assert_eq!(set(&model, C::Nick), text("n"));
    }

    #[test]
    fn a_disabled_field_is_not_set_even_with_a_value() {
        let mut form = with(full_form(), &[("title", "forged"), ("nick", "n")]);
        form.field_disabled("title", true);
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Title), None);
        assert_eq!(set(&model, C::Nick), text("n"));
    }

    #[test]
    fn a_field_unlocked_again_is_set() {
        let mut form = with(full_form(), &[("title", "t")]);
        form.field_readonly("title", true);
        form.field_readonly("title", false);
        form.field_disabled("title", true);
        form.field_disabled("title", false);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Title), text("t"));
    }

    #[test]
    fn an_emptied_nullable_text_column_is_set_to_a_typed_null() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Nick), Some(Value::String(None)));
    }

    #[test]
    fn an_emptied_nullable_integer_column_is_set_to_an_integer_null() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Score), Some(Value::Int(None)));
    }

    #[test]
    fn an_emptied_nullable_date_column_is_set_to_a_date_null() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Create);
        assert_eq!(set(&model, C::Born), Some(Value::from(None::<NaiveDate>)));
    }

    #[test]
    fn an_emptied_non_nullable_column_stays_not_set() {
        let form = full_form();
        for kind in [WriteKind::Create, WriteKind::Update] {
            let model = built(&form, &ColumnRights::All, kind);
            assert_eq!(set(&model, C::Title), None);
        }
    }

    #[test]
    fn an_empty_password_keeps_the_column_instead_of_clearing_it() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Pass), None);
    }

    #[test]
    fn a_password_not_hashed_yet_is_never_written_in_clear() {
        let form = with(full_form(), &[("pass", "hunter2")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Pass), None);
    }

    #[test]
    fn a_hashed_password_is_written() {
        let form = with(full_form(), &[("pass", "$argon2id$v=19$m=1,t=1,p=1$c$h")]);
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Pass), text("$argon2id$v=19$m=1,t=1,p=1$c$h"));
    }

    // The form registers a `FileField` wrapped in a `GenericField`: it is that
    // wrapper's answer that counts.
    #[test]
    fn an_empty_file_keeps_the_column_instead_of_clearing_it() {
        let form = full_form();
        let model = built(&form, &ColumnRights::All, WriteKind::Update);
        assert_eq!(set(&model, C::Avatar), None);
    }

    #[test]
    fn a_nullable_column_whose_null_cannot_be_typed_is_an_error() {
        let form = with_addr(full_form());
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Update).unwrap_err();
        assert_eq!(errors, vec![("addr".to_string(), unconvertible())]);
    }

    /// `form` with an empty `addr`, on a column that can't take a typed NULL.
    fn with_addr(mut form: Forms) -> Forms {
        form.field(&TextField::text("addr"));
        type_fields(&mut form);
        form
    }

    #[test]
    fn an_unreadable_value_is_an_error_naming_its_field() {
        let form = with(full_form(), &[("score", "abc")]);
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Create).unwrap_err();
        assert_eq!(errors, vec![("score".to_string(), unconvertible())]);
    }

    #[test]
    fn an_integer_one_above_the_column_width_is_an_error() {
        let form = with(full_form(), &[("score", "2147483647")]);
        assert!(changes::<E>(&form, &ColumnRights::All, WriteKind::Create).is_ok());
        let form = with(full_form(), &[("score", "2147483648")]);
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Create).unwrap_err();
        assert_eq!(errors, vec![("score".to_string(), unconvertible())]);
    }

    #[test]
    fn a_value_the_column_cannot_hold_is_an_error() {
        // Untyped: the text field hands a string to an integer column.
        let mut form = Forms::new("");
        form.field(&TextField::text("score"));
        let form = with(form, &[("score", "5")]);
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Update).unwrap_err();
        assert_eq!(errors, vec![("score".to_string(), unconvertible())]);
    }

    #[test]
    fn every_error_is_reported_not_only_the_first() {
        let form = with_addr(with(
            full_form(),
            &[("score", "abc"), ("born", "not a date")],
        ));
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Update).unwrap_err();
        assert_eq!(
            errors,
            vec![
                ("score".to_string(), unconvertible()),
                ("born".to_string(), unconvertible()),
                ("addr".to_string(), unconvertible()),
            ]
        );
    }

    #[test]
    fn an_error_on_a_column_not_ticked_is_not_reported() {
        let rights = only(&[("title", true, true)]);
        let form = with(full_form(), &[("score", "abc"), ("title", "t")]);
        let model = built(&form, &rights, WriteKind::Create);
        assert_eq!(set(&model, C::Title), text("t"));
    }

    #[test]
    fn an_error_on_a_locked_field_is_not_reported() {
        let mut form = with(full_form(), &[("score", "abc")]);
        form.field_readonly("score", true);
        assert!(changes::<E>(&form, &ColumnRights::All, WriteKind::Update).is_ok());
    }

    #[test]
    fn an_error_on_a_secret_or_a_key_on_update_is_not_reported() {
        let form = with(full_form(), &[("id", "abc"), ("secret", "x")]);
        assert!(changes::<E>(&form, &ColumnRights::All, WriteKind::Update).is_ok());
        let errors = changes::<E>(&form, &ColumnRights::All, WriteKind::Create).unwrap_err();
        assert_eq!(errors, vec![("id".to_string(), unconvertible())]);
    }

    #[test]
    fn a_valid_form_gives_the_model() {
        let form = with(full_form(), &[("title", "t")]);
        assert!(changes::<E>(&form, &ColumnRights::All, WriteKind::Update).is_ok());
    }

    #[test]
    fn the_write_kind_is_compared_by_value() {
        assert_eq!(WriteKind::Create, WriteKind::Create);
        assert_ne!(WriteKind::Create, WriteKind::Update);
    }
}
