/// Entry point of the builder
///
/// # Example
///
/// ```rust
/// use runique::model;
/// use runique::migration::{
/// PrimaryKeyDef, ColumnDef, ForeignKeyDef, RelationDef, IndexDef, HooksDef};
///
/// use sea_query::ForeignKeyAction;
///
/// let article = model!("Article")
///     .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
///     .column(ColumnDef::new("title").string().required())
///     .column(ColumnDef::new("content").text().required())
///     .column(ColumnDef::new("user_id").integer().required())
///     .column(ColumnDef::new("created_at").auto_now())
///     .column(ColumnDef::new("updated_at").auto_now_update())
///     .foreign_key(
///         ForeignKeyDef::new("user_id")
///             .references("user")
///             .on_delete(ForeignKeyAction::Cascade)
///     )
///     .relation(RelationDef::belongs_to("user", "user_id", "id"))
///     .relation(RelationDef::has_many("comment"))
///     .index(IndexDef::new(vec!["title"]).unique())
///     .hooks(HooksDef::from_file("src/hooks/article.rs"))
///     .build()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! model {
    ($name:expr) => {
        $crate::migration::ModelSchema::new($name)
    };
}
