//! What the commands need to know of the framework's own tables, copied from
//! `runique::auth` so that the command builds without the framework.
//!
//! A copy has to stay the copy of something: the tests at the end of this
//! file compare each piece with the one of `runique`.

/// The framework's own migrations, as `runique::auth::migrations_table` names
/// them.
pub(crate) mod migrations_table {
    /// The names the framework's own migrations are recorded under in
    /// `seaql_migrations`.
    pub const FRAMEWORK_MIGRATION_NAMES: [&str; 5] = [
        "m000000_000001_runique_eihwaz_users",
        "m000000_000002_runique_admin_table",
        "m000000_000003_runique_eihwaz_sessions",
        "m000000_000004_runique_eihwaz_history",
        "m000000_000005_runique_reset_tokens",
    ];
}

/// The accounts table (`eihwaz_users`): its name and its columns, as
/// `runique::auth::user` declares them.
pub(crate) mod user {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "eihwaz_users")]
    pub struct Model {
        // The key has the type of the project (`i32`, `i64` or a UUID), which
        // the command reads from the project's `Cargo.toml`. It is never
        // decoded through this model: the commands read it as text.
        #[sea_orm(primary_key)]
        pub id: i32,
        pub username: String,
        pub email: String,
        pub password: String,
        pub is_active: bool,
        pub is_dirigeant: bool,
        pub is_admin: bool,
        pub is_superuser: bool,
        pub created_at: Option<chrono::NaiveDateTime>,
        pub updated_at: Option<chrono::NaiveDateTime>,
        pub activated_at: Option<chrono::NaiveDateTime>,
        pub session_version: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{EntityName, IdenStatic, Iterable};

    // The copy names the same migrations, in the same order.
    #[test]
    fn the_migration_names_are_those_of_runique() {
        assert_eq!(
            migrations_table::FRAMEWORK_MIGRATION_NAMES,
            runique::auth::migrations_table::FRAMEWORK_MIGRATION_NAMES
        );
    }

    // Same table, same columns, in the same order: a column added to
    // `runique` and forgotten here makes this test fail.
    #[test]
    fn the_accounts_table_is_the_one_of_runique() {
        assert_eq!(
            user::Entity.table_name(),
            runique::auth::user::Entity.table_name()
        );
        let copied: Vec<&str> = user::Column::iter().map(|c| c.as_str()).collect();
        let original: Vec<&str> = runique::auth::user::Column::iter()
            .map(|c| c.as_str())
            .collect();
        assert_eq!(copied, original);
    }
}
