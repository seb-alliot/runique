use runique::prelude::*;

model! {
    Contributions,
    table: "contributions",
    pk: id => Pk,
    enums: {
        ContributionType: [Runique="Runique", Cours="Cours"],
    },
    {
        user_id:           Pk [required],
        contribution_type: choice [enum(ContributionType), required],
        title:             text [max_length: 200, required],
        content:           richtext [required],
        created_at:        datetime [auto_now],
    },
    relations: {
        belongs_to: eihwaz_users via user_id [cascade],
    },
    meta: { owner: user_id },
}

#[cfg(test)]
mod model_meta_tests {
    use runique::admin::refonte_admin::meta::ModelMeta;

    #[test]
    fn the_owner_is_the_declared_user_column() {
        assert_eq!(<super::Entity as ModelMeta>::KEY, "contributions");
        assert!(matches!(
            <super::Entity as ModelMeta>::OWNER,
            Some(super::Column::UserId)
        ));
    }
}
