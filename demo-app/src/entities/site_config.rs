use runique::prelude::*;

model! {
    SiteConfig,
    table: "site_config",
    pk: id => Pk,
    fields: {
        key: String [required],
        value: String [required],
        description: String [nullable],
    }
}
