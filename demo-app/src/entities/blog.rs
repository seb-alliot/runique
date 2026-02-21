pub use crate::runique::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};

#[allow(dead_code)]
pub fn blog_schema() -> ModelSchema {
    model!("Blog")
        .table_name("blog")
        .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
        .column(ColumnDef::new("title").string().required())
        .column(ColumnDef::new("email").string().required())
        .column(ColumnDef::new("website").string().nullable())
        .column(ColumnDef::new("summary").text().required())
        .column(ColumnDef::new("content").text().required())
        .column(ColumnDef::new("created_at").auto_now())
        .build()
        .unwrap()
}
