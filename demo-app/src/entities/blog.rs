use runique::prelude::*;

model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    enums: {
        BlogStatus: [Draft="Draft", Published="Published", Archived="Archived"],
    },
    {
        title:   text [required],
        email:   email [required],
        website: url [nullable],
        summary: textarea [rows: 3, required],
        content: richtext [rows: 15, required],
        status:     choice [enum(BlogStatus), default: "Draft", required],
        view_count: int [nullable, default: 0],
    }
}

#[cfg(test)]
mod model_meta_tests {
    use runique::admin::refonte_admin::meta::ModelMeta;

    #[test]
    fn test_model_meta() {
        assert_eq!(<super::Entity as ModelMeta>::KEY, "blog");
    }
}
