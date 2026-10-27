//! `save_as`: a form's save inside the admin's constraint. The account the
//! middleware checked, the row `edit` pointed at, the gate's proof and the
//! columns the rights allow, written in one transaction.
use super::change::{FieldErrors, WriteKind, changes};
use super::check::{GateCtx, authorize_create, authorize_row};
use super::operation::{Delete, Update};
use super::refusal::{Refusal, RefusalReason};
use super::rights::Crud;
use super::{PkOf, write};
use crate::auth::meta::ModelMeta;
use crate::forms::field::SaveContext;
use crate::forms::form::{Forms, SaveComportement};
use crate::utils::aliases::ADb;
use crate::utils::trad::t;
use sea_orm::{ActiveModelTrait, DbErr, IdenStatic, Iterable, TransactionTrait, Value};
use std::str::FromStr;

fn refused(refusal: Refusal) -> DbErr {
    DbErr::Custom(refusal.message())
}

/// Puts each refused field's reason on its field, for the form shown again.
fn on_fields(form: &mut Forms, errors: FieldErrors) -> DbErr {
    for (name, message) in errors {
        if let Some(field) = form.fields.get_mut(&name) {
            field.set_error(message);
        }
    }
    DbErr::Custom("save_as(): fields refused by the gate".to_string())
}

/// The gated write of `form` on `E`, as the account its constraint carries.
/// `Create` without a target, `Update` / `Delete` with the one `edit` set:
/// any other pairing is the handler's mistake, refused.
pub(crate) async fn save_as<E: ModelMeta>(
    form: &mut Forms,
    ctx: SaveContext,
    db: &ADb,
) -> Result<(), DbErr>
where
    E::ActiveModel: Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
    let SaveComportement::Gated { user } = &form.save_comportement else {
        return Err(refused(Refusal::new(RefusalReason::NoRight)));
    };
    let user = user.clone();
    let target = match &form.target {
        Some(text) => Some(
            PkOf::<E>::from_str(text)
                .map_err(|_| refused(Refusal::new(RefusalReason::NotFound)))?,
        ),
        None => None,
    };
    let gate = GateCtx { db, user: &user };
    let rights = user.rights_on(E::KEY);

    match (ctx, target) {
        (SaveContext::Create, None) => {
            let proof = authorize_create::<E>(&gate).map_err(refused)?;
            let mut model = changes::<E>(form, &rights.columns, WriteKind::Create)
                .map_err(|e| on_fields(form, e))?;
            // Whoever creates the row owns it, whatever the form sent.
            if let Some(owner) = E::OWNER {
                model.try_set(owner, Value::from(user.id))?;
            }
            missing_required::<E>(&model).map_err(|e| on_fields(form, e))?;
            let txn = db.begin().await?;
            write::insert::<E, E::ActiveModel, _>(proof, model, &txn)
                .await
                .map_err(refused)?;
            txn.commit().await
        }
        (SaveContext::Update, Some(id)) => {
            let proof = authorize_row::<Update, E>(&gate, id)
                .await
                .map_err(refused)?;
            let mut model = changes::<E>(form, &rights.columns, WriteKind::Update)
                .map_err(|e| on_fields(form, e))?;
            // Through `_own` alone, the owner column is what grants the
            // right: writing it would hand the row over, or take another's.
            if !rights.crud.has(Crud::Update)
                && let Some(owner) = E::OWNER
            {
                model.not_set(owner);
            }
            if !model.is_changed() {
                return Ok(());
            }
            let txn = db.begin().await?;
            write::update(proof, model, &txn).await.map_err(refused)?;
            txn.commit().await
        }
        (SaveContext::Delete, Some(id)) => {
            let proof = authorize_row::<Delete, E>(&gate, id)
                .await
                .map_err(refused)?;
            let txn = db.begin().await?;
            write::delete(proof, &txn).await.map_err(refused)?;
            txn.commit().await
        }
        (SaveContext::Create, Some(_)) => Err(DbErr::Custom(
            "save_as(): Create on a form pointed at a row by edit()".to_string(),
        )),
        (SaveContext::Update | SaveContext::Delete, None) => Err(DbErr::Custom(
            "save_as(): Update or Delete needs the row, set by edit()".to_string(),
        )),
    }
}

/// On create, once the gate set its own values: the NOT NULL columns without
/// a default still unset. Shown as required on their field instead of the
/// database refusing the insert.
fn missing_required<E: ModelMeta>(model: &E::ActiveModel) -> Result<(), FieldErrors> {
    let schema = E::schema();
    let missing: FieldErrors = E::Column::iter()
        .filter(|column| {
            schema
                .columns
                .iter()
                .find(|c| c.name == column.as_str())
                .is_some_and(|c| {
                    !(c.nullable
                        || c.default.is_some()
                        || c.auto_now
                        || c.auto_now_update
                        || c.ignored)
                })
                && model.get(*column).is_not_set()
        })
        .map(|column| {
            (
                column.as_str().to_string(),
                t("forms.required").into_owned(),
            )
        })
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::auth::current_user::CurrentUser;
    use crate::auth::gate::test_support::fixtures::{db, note, pk, staff};
    use crate::auth::permissions::ColumnRight;
    use crate::auth::permissions::rang::Rang;
    #[cfg(not(feature = "pk-uuid"))]
    use crate::forms::fields::NumericField;
    use crate::forms::fields::TextField;
    #[cfg(feature = "pk-uuid")]
    use crate::forms::fields::UUIDField;
    use crate::utils::pk::Pk;
    use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryOrder, Schema, Set};

    /// `save_tickets`: a NOT NULL `title` without default, a NOT NULL `level`
    /// with one, and three columns the schema exempts from "required"
    /// (`auto_now`, `auto_now_update`, ignored). Its `before_save` stamps
    /// `stamp` on an update, like an `auto_now_update` column.
    mod ticket {
        use crate::utils::pk::Pk;
        use sea_orm::entity::prelude::*;
        use sea_orm::{ConnectionTrait, DbErr};

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "save_tickets")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub owner_id: Option<Pk>,
            pub title: String,
            pub body: Option<String>,
            #[sea_orm(default_value = 0)]
            pub level: i32,
            pub stamp: Option<String>,
            pub made: Option<String>,
            pub edited: Option<String>,
            pub hidden: Option<String>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        #[async_trait::async_trait]
        impl ActiveModelBehavior for ActiveModel {
            async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
            where
                C: ConnectionTrait,
            {
                if !insert {
                    self.stamp = sea_orm::ActiveValue::Set(Some("stamped".to_string()));
                }
                Ok(self)
            }
        }
    }

    #[derive(serde::Serialize, Debug, Clone)]
    #[serde(transparent)]
    pub struct TicketForm {
        form: Forms,
    }

    impl crate::forms::field::FormAccess for TicketForm {
        fn register_fields(_form: &mut Forms) {}
        crate::impl_form_access!();
    }

    impl crate::forms::field::RuniqueForm for TicketForm {}

    impl ModelMeta for ticket::Entity {
        const KEY: &'static str = "tickets";
        type Form = TicketForm;
        const OWNER: Option<ticket::Column> = Some(ticket::Column::OwnerId);
        fn schema() -> crate::migration::schema::ModelSchema {
            use crate::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};
            ModelSchema::new("Ticket")
                .table_name("save_tickets")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("owner_id").integer().nullable())
                .column(ColumnDef::new("title").text())
                .column(ColumnDef::new("body").text().nullable())
                .column(
                    ColumnDef::new("level")
                        .integer()
                        .default(sea_orm::Value::Int(Some(0))),
                )
                .column(ColumnDef::new("stamp").text().nullable())
                .column(ColumnDef::new("made").text().auto_now())
                .column(ColumnDef::new("edited").text().auto_now_update())
                .column(ColumnDef::new("hidden").text().ignore())
                .build()
                .unwrap()
        }
    }

    /// Tickets: 1 owned by `pk(10)` titled "one", 2 owned by `pk(20)` titled "two".
    async fn setup() -> ADb {
        let db = db().await;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        db.execute(&schema.create_table_from_entity(ticket::Entity))
            .await
            .unwrap();
        for (id, owner, title) in [(1, pk(10), "one"), (2, pk(20), "two")] {
            ticket::ActiveModel {
                id: Set(id),
                owner_id: Set(Some(owner)),
                title: Set(title.into()),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }
        db
    }

    fn all_columns<C: ColumnTrait>() -> Vec<ColumnRight> {
        C::iter()
            .map(|c| ColumnRight {
                colonne: IdenStatic::as_str(&c).into(),
                can_read: true,
                can_create: true,
                can_update: true,
            })
            .collect()
    }

    /// A lambda whose only group grants `rights` on `tickets`, every column ticked.
    /// Order: create, read, read_own, update, delete, update_own, delete_own.
    fn member(id: Pk, rights: [bool; 7]) -> CurrentUser {
        let mut user = staff(id, "tickets", rights);
        user.groupes[0].permissions[0].colonnes = all_columns::<ticket::Column>();
        user
    }

    fn ranked(mut user: CurrentUser, rang: Rang) -> CurrentUser {
        user.rang = rang;
        user.droits = user.groupes[0].permissions.clone();
        user
    }

    fn only_columns(mut user: CurrentUser, ticks: &[(&str, bool, bool)]) -> CurrentUser {
        user.groupes[0].permissions[0].colonnes = ticks
            .iter()
            .map(|(name, create, update)| ColumnRight {
                colonne: (*name).into(),
                can_read: true,
                can_create: *create,
                can_update: *update,
            })
            .collect();
        user
    }

    const CREATE: [bool; 7] = [true, false, false, false, false, false, false];
    const UPDATE: [bool; 7] = [false, false, false, true, false, false, false];
    const UPDATE_OWN: [bool; 7] = [false, false, false, false, false, true, false];
    const DELETE: [bool; 7] = [false, false, false, false, true, false, false];
    const DELETE_OWN: [bool; 7] = [false, false, false, false, false, false, true];
    const NOTHING: [bool; 7] = [false; 7];
    const EVERYTHING: [bool; 7] = [true, true, true, true, true, true, true];

    fn register(form: &mut Forms, name: &str) {
        if name == "owner_id" {
            #[cfg(feature = "pk-uuid")]
            form.field(&UUIDField::new(name));
            #[cfg(not(feature = "pk-uuid"))]
            form.field(&NumericField::integer(name));
        } else {
            form.field(&TextField::text(name));
        }
    }

    /// A form with one field per name in `values`, typed like `E`'s columns.
    fn form_for<E: EntityTrait>(
        user: Option<CurrentUser>,
        target: Option<&str>,
        values: &[(&str, &str)],
    ) -> Forms {
        let mut form = Forms::new("");
        for (name, _) in values {
            register(&mut form, name);
        }
        for column in E::Column::iter() {
            if let Some(field) = form.fields.get_mut(IdenStatic::as_str(&column)) {
                field.apply_column_type(column.def().get_column_type());
            }
        }
        form.save_comportement = match user {
            Some(user) => SaveComportement::Gated { user },
            None => SaveComportement::Open,
        };
        form.target = target.map(str::to_string);
        for (name, value) in values {
            form.add_value(name, value);
        }
        form
    }

    fn gated(user: &CurrentUser, target: Option<&str>, values: &[(&str, &str)]) -> Forms {
        form_for::<ticket::Entity>(Some(user.clone()), target, values)
    }

    async fn run(form: &mut Forms, ctx: SaveContext, db: &ADb) -> Result<(), DbErr> {
        save_as::<ticket::Entity>(form, ctx, db).await
    }

    async fn tickets(db: &ADb) -> Vec<ticket::Model> {
        ticket::Entity::find()
            .order_by_asc(ticket::Column::Id)
            .all(&**db)
            .await
            .unwrap()
    }

    async fn ticket_of(db: &ADb, id: i32) -> Option<ticket::Model> {
        ticket::Entity::find_by_id(id).one(&**db).await.unwrap()
    }

    fn refusal_message() -> String {
        Refusal::new(RefusalReason::NoRight).message()
    }

    fn assert_refused(result: Result<(), DbErr>) {
        match result {
            Err(DbErr::Custom(message)) => assert_eq!(message, refusal_message()),
            other => panic!("expected the gate's refusal, got {other:?}"),
        }
    }

    fn assert_handler_error(result: Result<(), DbErr>) {
        match result {
            Err(DbErr::Custom(message)) => {
                assert!(message.starts_with("save_as():"), "{message}");
                assert_ne!(message, refusal_message());
            }
            other => panic!("expected the handler's error, got {other:?}"),
        }
    }

    fn assert_fields_refused(result: Result<(), DbErr>) {
        match result {
            Err(DbErr::Custom(message)) => assert!(message.contains("fields refused"), "{message}"),
            other => panic!("expected the fields' refusal, got {other:?}"),
        }
    }

    // --- the form must be gated ---

    #[tokio::test]
    async fn an_open_form_is_refused_on_every_context_and_writes_nothing() {
        let db = setup().await;
        let before = tickets(&db).await;
        let mut form = form_for::<ticket::Entity>(None, None, &[("title", "x")]);
        assert_refused(run(&mut form, SaveContext::Create, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[("title", "x")]);
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[]);
        assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    // --- create ---

    #[tokio::test]
    async fn a_create_writes_the_row_and_the_column_defaults_apply() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "fresh"), ("body", "text")]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        let rows = tickets(&db).await;
        assert_eq!(rows.len(), 3);
        let row = &rows[2];
        assert_eq!(row.title, "fresh");
        assert_eq!(row.body.as_deref(), Some("text"));
        assert_eq!(row.owner_id, Some(pk(10)));
        assert_eq!(row.level, 0, "the column default, not a required error");
        assert_eq!(row.stamp, None, "an insert is not an update");
    }

    #[tokio::test]
    async fn the_creator_owns_the_row_whatever_the_form_sends() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let forged = pk(20).to_string();
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", &forged)]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn the_creator_owns_the_row_when_the_form_has_no_owner_field() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "t")]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn the_owner_is_imposed_even_when_its_column_is_not_ticked() {
        let db = setup().await;
        let creator = only_columns(member(pk(10), CREATE), &[("title", true, true)]);
        let forged = pk(20).to_string();
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", &forged)]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn a_create_without_the_create_right_is_refused_and_writes_nothing() {
        let db = setup().await;
        for rights in [NOTHING, UPDATE, UPDATE_OWN, DELETE] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, None, &[("title", "t")]);
            assert_refused(run(&mut form, SaveContext::Create, &db).await);
        }
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_missing_required_column_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", ""), ("body", "b")]);
        assert_fields_refused(run(&mut form, SaveContext::Create, &db).await);
        let expected = t("forms.required").into_owned();
        assert_eq!(form.fields.get("title").unwrap().error(), Some(&expected));
        assert_eq!(form.fields.get("body").unwrap().error(), None);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_required_column_without_a_field_is_refused_too() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("body", "b")]);
        assert_fields_refused(run(&mut form, SaveContext::Create, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_required_column_the_rights_do_not_tick_is_missing_not_written() {
        let db = setup().await;
        let creator = only_columns(member(pk(10), CREATE), &[("body", true, true)]);
        let mut form = gated(&creator, None, &[("title", "forged"), ("body", "b")]);
        assert_fields_refused(run(&mut form, SaveContext::Create, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn the_exempt_columns_are_not_reported_as_required() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        // Only `title` is declared: `level` (default), `body` (nullable),
        // `made`, `edited`, `hidden` (exempt) stay unset and the insert goes.
        let mut form = gated(&creator, None, &[("title", "t")]);
        assert!(run(&mut form, SaveContext::Create, &db).await.is_ok());
        assert_eq!(tickets(&db).await.len(), 3);
    }

    #[tokio::test]
    async fn an_unreadable_value_on_create_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveContext::Create, &db).await);
        assert!(form.fields.get("owner_id").unwrap().error().is_some());
        assert_eq!(form.fields.get("title").unwrap().error(), None);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    // --- update ---

    #[tokio::test]
    async fn a_full_update_writes_the_columns_and_may_reassign_the_row() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let to = pk(30).to_string();
        let mut form = gated(
            &editor,
            Some("2"),
            &[("title", "renamed"), ("body", "b"), ("owner_id", &to)],
        );
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        let row = ticket_of(&db, 2).await.unwrap();
        assert_eq!(row.title, "renamed");
        assert_eq!(row.body.as_deref(), Some("b"));
        assert_eq!(row.owner_id, Some(pk(30)), "the full right may reassign");
        assert_eq!(row.stamp.as_deref(), Some("stamped"), "before_save ran");
        assert_eq!(ticket_of(&db, 1).await.unwrap().title, "one");
    }

    #[tokio::test]
    async fn the_full_right_beside_the_own_right_still_reassigns() {
        let db = setup().await;
        let editor = member(pk(10), [false, false, false, true, false, true, false]);
        let to = pk(30).to_string();
        let mut form = gated(&editor, Some("1"), &[("owner_id", &to)]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        assert_eq!(ticket_of(&db, 1).await.unwrap().owner_id, Some(pk(30)));
    }

    #[tokio::test]
    async fn an_own_update_writes_the_columns_but_never_hands_the_row_over() {
        let db = setup().await;
        let owner = member(pk(10), UPDATE_OWN);
        let to = pk(20).to_string();
        let mut form = gated(&owner, Some("1"), &[("title", "mine"), ("owner_id", &to)]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        let row = ticket_of(&db, 1).await.unwrap();
        assert_eq!(row.title, "mine");
        assert_eq!(row.owner_id, Some(pk(10)), "the row stays the account's");
        assert_eq!(row.stamp.as_deref(), Some("stamped"));
    }

    #[tokio::test]
    async fn an_own_update_sending_only_the_owner_changes_nothing_and_succeeds() {
        let db = setup().await;
        let owner = member(pk(10), UPDATE_OWN);
        let to = pk(20).to_string();
        let before = tickets(&db).await;
        let mut form = gated(&owner, Some("1"), &[("owner_id", &to)]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        assert_eq!(tickets(&db).await, before, "not even the stamp");
    }

    #[tokio::test]
    async fn an_own_update_cannot_take_a_row_of_another() {
        let db = setup().await;
        let owner = member(pk(10), UPDATE_OWN);
        let mine = pk(10).to_string();
        let before = tickets(&db).await;
        let mut form = gated(
            &owner,
            Some("2"),
            &[("title", "taken"), ("owner_id", &mine)],
        );
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn an_update_without_any_right_is_refused() {
        let db = setup().await;
        let before = tickets(&db).await;
        for rights in [NOTHING, CREATE, DELETE, DELETE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[("title", "x")]);
            assert_refused(run(&mut form, SaveContext::Update, &db).await);
        }
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn an_update_without_change_succeeds_and_writes_nothing() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let before = tickets(&db).await;
        let mut form = gated(&editor, Some("1"), &[]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        assert_eq!(tickets(&db).await, before, "no stamp: nothing was written");
    }

    #[tokio::test]
    async fn an_update_without_change_still_goes_through_the_gate() {
        let db = setup().await;
        let before = tickets(&db).await;
        let user = member(pk(10), NOTHING);
        let mut form = gated(&user, Some("1"), &[]);
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        let owner = member(pk(10), UPDATE_OWN);
        let mut form = gated(&owner, Some("2"), &[]);
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn an_update_only_writes_the_columns_ticked_for_update() {
        let db = setup().await;
        let editor = only_columns(
            member(pk(10), UPDATE),
            &[("title", false, true), ("body", true, false)],
        );
        let to = pk(30).to_string();
        let mut form = gated(
            &editor,
            Some("1"),
            &[("title", "t"), ("body", "b"), ("owner_id", &to)],
        );
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        let row = ticket_of(&db, 1).await.unwrap();
        assert_eq!(row.title, "t");
        assert_eq!(row.body, None);
        assert_eq!(row.owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn an_update_never_writes_the_key_nor_a_missing_row() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let before = tickets(&db).await;
        let mut form = gated(&editor, Some("99"), &[("title", "ghost")]);
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        assert_eq!(tickets(&db).await, before);

        let mut form = gated(&editor, Some("1"), &[("id", "2"), ("title", "t")]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        assert_eq!(ticket_of(&db, 1).await.unwrap().title, "t");
        assert_eq!(ticket_of(&db, 2).await.unwrap().title, "two");
    }

    #[tokio::test]
    async fn an_unreadable_value_on_update_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let before = tickets(&db).await;
        let mut form = gated(&editor, Some("1"), &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveContext::Update, &db).await);
        assert!(form.fields.get("owner_id").unwrap().error().is_some());
        assert_eq!(form.fields.get("title").unwrap().error(), None);
        assert_eq!(tickets(&db).await, before);
    }

    // --- delete ---

    #[tokio::test]
    async fn a_delete_removes_the_row_and_only_that_one() {
        let db = setup().await;
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("2"), &[]);
        run(&mut form, SaveContext::Delete, &db).await.unwrap();
        assert!(ticket_of(&db, 2).await.is_none());
        assert!(ticket_of(&db, 1).await.is_some());
    }

    #[tokio::test]
    async fn an_own_delete_removes_the_row_of_the_account_and_keeps_another() {
        let db = setup().await;
        let user = member(pk(10), DELETE_OWN);
        let mut form = gated(&user, Some("2"), &[]);
        assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        assert!(ticket_of(&db, 2).await.is_some(), "another's row kept");
        let mut form = gated(&user, Some("1"), &[]);
        run(&mut form, SaveContext::Delete, &db).await.unwrap();
        assert!(ticket_of(&db, 1).await.is_none());
    }

    #[tokio::test]
    async fn a_delete_without_the_right_is_refused_and_keeps_the_rows() {
        let db = setup().await;
        let before = tickets(&db).await;
        for rights in [NOTHING, CREATE, UPDATE, UPDATE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[]);
            assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        }
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn deleting_a_missing_row_is_refused() {
        let db = setup().await;
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("99"), &[]);
        assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    // --- the target ---

    #[tokio::test]
    async fn an_unreadable_target_is_refused_like_the_gate_and_writes_nothing() {
        let db = setup().await;
        let user = member(pk(10), EVERYTHING);
        let before = tickets(&db).await;
        for target in [
            "abc",
            "",
            "1x",
            "1/2",
            "../1",
            "1 OR 1=1",
            "99999999999",
            "-",
        ] {
            for ctx in [
                SaveContext::Update,
                SaveContext::Delete,
                SaveContext::Create,
            ] {
                let mut form = gated(&user, Some(target), &[("title", "x")]);
                assert_refused(run(&mut form, ctx, &db).await);
            }
        }
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn a_very_long_target_is_refused() {
        let db = setup().await;
        let user = member(pk(10), EVERYTHING);
        let long = "9".repeat(10_000);
        let mut form = gated(&user, Some(&long), &[]);
        assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_wrong_pairing_is_the_handlers_error_and_writes_nothing() {
        let db = setup().await;
        let user = member(pk(10), EVERYTHING);
        let before = tickets(&db).await;
        let mut form = gated(&user, Some("1"), &[("title", "x")]);
        assert_handler_error(run(&mut form, SaveContext::Create, &db).await);
        let mut form = gated(&user, None, &[("title", "x")]);
        assert_handler_error(run(&mut form, SaveContext::Update, &db).await);
        let mut form = gated(&user, None, &[]);
        assert_handler_error(run(&mut form, SaveContext::Delete, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    // --- ranks ---

    #[tokio::test]
    async fn the_superuser_and_the_dirigeant_do_everything_without_a_group() {
        for (rang, id) in [(Rang::Superuser, 30), (Rang::Dirigeant, 32)] {
            let db = setup().await;
            let mut user = ranked(member(pk(id), NOTHING), rang);
            user.groupes.clear();
            let to = pk(20).to_string();

            let mut form = gated(&user, None, &[("title", "t"), ("owner_id", &to)]);
            run(&mut form, SaveContext::Create, &db).await.unwrap();
            assert_eq!(tickets(&db).await[2].owner_id, Some(pk(id)));

            let mut form = gated(&user, Some("1"), &[("title", "t"), ("owner_id", &to)]);
            run(&mut form, SaveContext::Update, &db).await.unwrap();
            let row = ticket_of(&db, 1).await.unwrap();
            assert_eq!((row.title.as_str(), row.owner_id), ("t", Some(pk(20))));

            let mut form = gated(&user, Some("2"), &[]);
            run(&mut form, SaveContext::Delete, &db).await.unwrap();
            assert!(ticket_of(&db, 2).await.is_none());
        }
    }

    #[tokio::test]
    async fn an_admin_writes_with_the_box_ticked_and_is_refused_without() {
        let db = setup().await;
        let before = tickets(&db).await;
        let without = ranked(member(pk(33), NOTHING), Rang::Admin);
        let mut form = gated(&without, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveContext::Create, &db).await);
        let mut form = gated(&without, Some("1"), &[("title", "t")]);
        assert_refused(run(&mut form, SaveContext::Update, &db).await);
        let mut form = gated(&without, Some("1"), &[]);
        assert_refused(run(&mut form, SaveContext::Delete, &db).await);
        assert_eq!(tickets(&db).await, before);

        let with = ranked(member(pk(33), EVERYTHING), Rang::Admin);
        let mut form = gated(&with, None, &[("title", "t")]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        let mut form = gated(&with, Some("1"), &[("title", "u")]);
        run(&mut form, SaveContext::Update, &db).await.unwrap();
        let mut form = gated(&with, Some("2"), &[]);
        run(&mut form, SaveContext::Delete, &db).await.unwrap();
        let rows = tickets(&db).await;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "u");
    }

    #[tokio::test]
    async fn an_admin_rights_come_from_his_account_not_from_a_group() {
        let db = setup().await;
        let mut admin = ranked(member(pk(33), NOTHING), Rang::Admin);
        admin.groupes[0].permissions[0].can_create = true;
        let mut form = gated(&admin, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveContext::Create, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_lambda_of_a_group_granting_the_right_writes_and_one_without_is_refused() {
        let db = setup().await;
        let granted = member(pk(31), CREATE);
        let mut form = gated(&granted, None, &[("title", "t")]);
        run(&mut form, SaveContext::Create, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(31)));

        let mut other_key = member(pk(31), CREATE);
        other_key.groupes[0].permissions[0].resource_key = "other".into();
        let mut form = gated(&other_key, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveContext::Create, &db).await);

        let mut no_group = member(pk(31), CREATE);
        no_group.groupes.clear();
        let mut form = gated(&no_group, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveContext::Create, &db).await);
        assert_eq!(tickets(&db).await.len(), 3);
    }

    // --- an entity with the default behavior ---

    #[tokio::test]
    async fn an_entity_without_a_custom_before_save_is_written_as_sent() {
        let db = setup().await;
        let boss = ranked(staff(pk(32), "notes", NOTHING), Rang::Dirigeant);
        let to = pk(20).to_string();
        let mut form = form_for::<note::Entity>(Some(boss), Some("1"), &[("owner_id", &to)]);
        save_as::<note::Entity>(&mut form, SaveContext::Update, &db)
            .await
            .unwrap();
        let rows = note::Entity::find()
            .order_by_asc(note::Column::Id)
            .all(&*db)
            .await
            .unwrap();
        assert_eq!(rows[0].owner_id, Some(pk(20)));
        assert_eq!(rows[1].owner_id, Some(pk(20)));
        assert_eq!(rows[2].owner_id, None);
    }

    #[tokio::test]
    async fn an_own_update_on_the_notes_cannot_give_the_row_away() {
        let db = setup().await;
        let owner = staff(pk(10), "notes", UPDATE_OWN);
        let to = pk(20).to_string();
        let mut form = form_for::<note::Entity>(Some(owner), Some("1"), &[("owner_id", &to)]);
        save_as::<note::Entity>(&mut form, SaveContext::Update, &db)
            .await
            .unwrap();
        let row = note::Entity::find_by_id(1)
            .one(&*db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.owner_id, Some(pk(10)));
    }
}
