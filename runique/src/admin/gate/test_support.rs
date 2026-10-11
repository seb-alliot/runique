//! Test fixtures for the gate: proofs come from the real gate, never from
//! an opened `Authorized::new`.

/// A `notes` table with an owner column, `eihwaz_users` with one superuser
/// (`pk(30)`) and one ordinary account (`pk(31)`). Notes: 1 owned by
/// `pk(10)`, 2 owned by `pk(20)`, 3 without owner.
#[cfg(feature = "sqlite")]
pub(crate) mod fixtures {
    use crate::auth::session::CurrentUser;
    use crate::utils::aliases::ADb;
    use crate::utils::pk::Pk;
    use sea_orm::{ActiveModelTrait, ConnectionTrait, Schema, Set};

    pub(crate) mod note {
        use crate::utils::pk::Pk;
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "notes")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub owner_id: Option<Pk>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    impl crate::admin::meta::ModelMeta for note::Entity {
        const KEY: &'static str = "notes";
        const OWNER: Option<note::Column> = Some(note::Column::OwnerId);
    }

    #[cfg(feature = "pk-uuid")]
    pub(crate) fn pk(n: u8) -> Pk {
        uuid::Uuid::from_u128(n as u128)
    }
    #[cfg(not(feature = "pk-uuid"))]
    pub(crate) fn pk(n: u8) -> Pk {
        n.into()
    }

    pub(crate) async fn db() -> ADb {
        let conn = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        conn.execute(&schema.create_table_from_entity(note::Entity))
            .await
            .unwrap();
        conn.execute(&schema.create_table_from_entity(crate::auth::user::Entity))
            .await
            .unwrap();
        for (id, owner) in [(1, Some(pk(10))), (2, Some(pk(20))), (3, None)] {
            note::ActiveModel {
                id: Set(id),
                owner_id: Set(owner),
            }
            .insert(&conn)
            .await
            .unwrap();
        }
        for (id, name, su) in [(pk(30), "crawler", true), (pk(31), "member", false)] {
            crate::auth::user::ActiveModel {
                id: Set(id),
                username: Set(name.into()),
                email: Set(format!("{name}@example.com")),
                password: Set("x".into()),
                is_active: Set(false),
                is_staff: Set(false),
                is_superuser: Set(su),
                ..Default::default()
            }
            .insert(&conn)
            .await
            .unwrap();
        }
        ADb::from_connection(conn)
    }

    /// A staff member whose only group grants `rights` on `resource`.
    pub(crate) fn staff(id: Pk, resource: &str, rights: [bool; 6]) -> CurrentUser {
        use crate::auth::permissions::{Groupe, Permission};
        let [c, r, u, d, uo, dw] = rights;
        CurrentUser {
            id,
            username: "staff".into(),
            is_staff: true,
            is_superuser: false,
            groupes: vec![Groupe {
                id: 0,
                nom: "g".into(),
                permissions: vec![Permission {
                    resource_key: resource.into(),
                    can_create: c,
                    can_read: r,
                    can_update: u,
                    can_delete: d,
                    can_update_own: uo,
                    can_delete_own: dw,
                }],
            }],
        }
    }
}
