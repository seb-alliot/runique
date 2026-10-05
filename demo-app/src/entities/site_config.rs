use runique::prelude::*;

model! {
    SiteConfig,
    table: "site_config",
    pk: id => Pk,
    {
        key:         text [required],
        value:       text [required],
        description: text [nullable],
        is_public:   bool [nullable, default: true],
        sort_order:  int [nullable, default: 0],
    }
}
