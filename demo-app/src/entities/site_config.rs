use runique::prelude::*;

model! {
    SiteConfig,
    table: "site_config",
    pk: id => Pk,
    {
        key:         text [required],
        value:       text [required],
        description: text,
        is_public:   bool [default: true],
        sort_order:  int [default: 0],
    }
}
