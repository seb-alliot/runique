use runique::prelude::*;

model! {
    SiteConfig,
    table: "site_config",
    pk: id => Pk,
    {
        key:         text [required],
        value:       text [required],
        description: text,
    }
}
