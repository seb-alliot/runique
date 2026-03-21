use runique::prelude::*;

model! {
    SiteConfig,
    table: "site_config",
    pk: id => i32,
    fields: {
        key: String [required],
        value: String [required],
        description: String [nullable],
    }
}
