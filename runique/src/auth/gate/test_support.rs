//! Test fixtures for the gate: proofs come from the real gate, never from
//! an opened `Authorized::new`.

/// A `notes` table with an owner column, `eihwaz_users` with one account per
/// rank: superuser `pk(30)`, lambda `pk(31)`, dirigeant `pk(32)`, admin
/// `pk(33)`. Notes: 1 owned by `pk(10)`, 2 owned by `pk(20)`, 3 without owner.
#[cfg(feature = "sqlite")]
pub(crate) mod fixtures {
    use crate::auth::current_user::CurrentUser;
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
    /// The notes' admin form: the owner, typed like the key it points to.
    #[derive(serde::Serialize, Debug, Clone)]
    #[serde(transparent)]
    pub struct NoteForm {
        pub form: crate::forms::form::Forms,
    }

    impl crate::forms::field::RuniqueForm for NoteForm {
        fn register_fields(form: &mut crate::forms::form::Forms) {
            #[cfg(feature = "pk-uuid")]
            form.field(&crate::forms::fields::UUIDField::new("owner_id"));
            #[cfg(not(feature = "pk-uuid"))]
            form.field(&crate::forms::fields::NumericField::integer("owner_id"));
        }
        crate::impl_form_access!();
    }

    impl crate::auth::meta::ModelMeta for note::Entity {
        const KEY: &'static str = "notes";
        type Form = NoteForm;
        const OWNER: Option<note::Column> = Some(note::Column::OwnerId);
        fn schema() -> crate::migration::schema::ModelSchema {
            crate::migration::ModelSchema::new("Note")
                .table_name("notes")
                .primary_key(
                    crate::migration::PrimaryKeyDef::new("id")
                        .i32()
                        .auto_increment(),
                )
                .column(
                    crate::migration::ColumnDef::new("owner_id")
                        .integer()
                        .nullable(),
                )
                .build()
                .unwrap()
        }
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
        // (superuser, dirigeant, admin)
        for (id, name, (su, dirigeant, admin)) in [
            (pk(30), "crawler", (true, false, false)),
            (pk(31), "member", (false, false, false)),
            (pk(32), "boss", (false, true, false)),
            (pk(33), "delegate", (false, false, true)),
        ] {
            crate::auth::user::ActiveModel {
                id: Set(id),
                username: Set(name.into()),
                email: Set(format!("{name}@example.com")),
                password: Set("x".into()),
                is_active: Set(false),
                is_dirigeant: Set(dirigeant),
                is_admin: Set(admin),
                is_superuser: Set(su),
                ..Default::default()
            }
            .insert(&conn)
            .await
            .unwrap();
        }
        ADb::from_connection(conn)
    }

    /// A staff member whose only group grants `rights` on `resource`, every
    /// column of the fixtures' tables ticked: a table-level test reads as it
    /// always did, a column test sets its own `colonnes`.
    pub(crate) fn staff(id: Pk, resource: &str, rights: [bool; 7]) -> CurrentUser {
        use crate::auth::permissions::{ColumnRight, Groupe, Permission};
        use sea_orm::{IdenStatic, Iterable};
        let [c, r, ro, u, d, uo, dw] = rights;
        let colonnes = note::Column::iter()
            .map(|col| col.as_str())
            .chain(crate::auth::user::Column::iter().map(|col| col.as_str()))
            .map(|name| ColumnRight {
                colonne: name.into(),
                can_read: true,
                can_create: true,
                can_update: true,
            })
            .collect();
        CurrentUser {
            id,
            username: "staff".into(),
            rang: crate::auth::permissions::rang::Rang::Lambda,
            groupes: vec![Groupe {
                id: 0,
                nom: "g".into(),
                is_staff: true,
                permissions: vec![Permission {
                    resource_key: resource.into(),
                    can_create: c,
                    can_read: r,
                    can_read_own: ro,
                    can_update: u,
                    can_delete: d,
                    can_update_own: uo,
                    can_delete_own: dw,
                    colonnes,
                }],
            }],
            droits: Vec::new(),
        }
    }
}
