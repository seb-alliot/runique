pub use crate::formulaire::user::Entity;
pub use crate::runique::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};

pub fn eihwaz_users_schema() -> ModelSchema {
    model!("EihwazUsers")
        .table_name("eihwaz_users")
        .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
        .column(ColumnDef::new("username").string().required())
        .column(ColumnDef::new("email").string().required())
        .column(ColumnDef::new("password").string().required())
        .column(ColumnDef::new("is_active").boolean().required())
        .column(ColumnDef::new("is_staff").boolean().required())
        .column(ColumnDef::new("is_superuser").boolean().required())
        .column(ColumnDef::new("roles").string().nullable())
        .column(ColumnDef::new("created_at").datetime().nullable())
        .column(ColumnDef::new("updated_at").datetime().nullable())
        .build()
        .unwrap()
}
