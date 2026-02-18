use crate::runique::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};

pub fn users_booster_schema() -> ModelSchema {
    model!("UsersBooster")
        .table_name("users_booster")
        .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
        .column(ColumnDef::new("username").string().required())
        .column(ColumnDef::new("email").string().required())
        .column(ColumnDef::new("password").string().required())
        .column(ColumnDef::new("bio").string().nullable())
        .column(ColumnDef::new("website").string().nullable())
        .column(ColumnDef::new("is_active").boolean().required())
        .column(ColumnDef::new("created_at").auto_now())
        .column(ColumnDef::new("updated_at").auto_now_update())
        .build()
        .unwrap()
}
