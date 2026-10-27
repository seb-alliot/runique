//! `save_as`: a form's save inside the admin's constraint. The account the
//! middleware checked, the row the URL points at, the gate's proof and the
//! columns the rights allow, written in one transaction with its history row.
use super::change::{FieldErrors, WriteKind, changes};
use super::check::{GateCtx, authorize_create, authorize_row};
use super::operation::{Delete, Update};
use super::refusal::{Refusal, RefusalReason};
use super::rights::Crud;
use super::write;
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action, redact_sensitive};
use crate::auth::meta::ModelMeta;
use crate::forms::form::{Forms, SaveComportement};
use crate::forms::save::{SaveAction, on_fields, url_key};
use crate::utils::aliases::ADb;
use crate::utils::trad::t;
use sea_orm::sea_query::sea_value_to_json_value;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DbErr, IdenStatic, Iterable, ModelTrait,
    PrimaryKeyToColumn, TransactionTrait, Value,
};

fn refused(refusal: Refusal) -> DbErr {
    DbErr::Custom(refusal.message())
}

/// The gated write of `form` on `E`, as the account its constraint carries.
/// Without a key in the URL `save` creates, with one it updates; `delete`
/// needs one. A key that doesn't read as `E`'s is refused as not found.
pub(crate) async fn save_as<E: ModelMeta>(
    form: &mut Forms,
    action: SaveAction,
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
    let target =
        url_key::<E>(form).map_err(|()| refused(Refusal::new(RefusalReason::NotFound)))?;
    let gate = GateCtx { db, user: &user };
    let rights = user.rights_on(E::KEY);

    match (action, target) {
        (SaveAction::Save, None) => {
            let proof = authorize_create::<E>(&gate).map_err(refused)?;
            let mut model = changes::<E>(form, &rights.columns, WriteKind::Create)
                .map_err(|e| on_fields(form, e))?;
            // Whoever creates the row owns it, whatever the form sent.
            if let Some(owner) = E::OWNER {
                model.try_set(owner, Value::from(user.id))?;
            }
            missing_required::<E>(&model).map_err(|e| on_fields(form, e))?;
            let txn = db.begin().await?;
            let row = write::insert::<E, E::ActiveModel, _>(proof, model, &txn)
                .await
                .map_err(refused)?;
            trace(
                &txn,
                &user,
                E::KEY,
                &key_of::<E>(&row),
                &label_of::<E>(&row),
                "create",
                None,
            )
            .await?;
            txn.commit().await
        }
        (SaveAction::Save, Some(id)) => {
            let proof = authorize_row::<Update, E>(&gate, id.clone())
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
            // The row as it was, read in the write's transaction: the trace
            // compares against what the update replaces.
            let Some(old) = E::find_by_id(id).one(&txn).await? else {
                return Err(refused(Refusal::new(RefusalReason::StaleRow)));
            };
            let summary = summary::<E>(&old, &model);
            write::update(proof, model, &txn).await.map_err(refused)?;
            trace(
                &txn,
                &user,
                E::KEY,
                &key_of::<E>(&old),
                &label_of::<E>(&old),
                "update",
                summary,
            )
            .await?;
            txn.commit().await
        }
        (SaveAction::Delete, Some(id)) => {
            let proof = authorize_row::<Delete, E>(&gate, id.clone())
                .await
                .map_err(refused)?;
            let txn = db.begin().await?;
            let Some(old) = E::find_by_id(id).one(&txn).await? else {
                return Err(refused(Refusal::new(RefusalReason::StaleRow)));
            };
            write::delete(proof, &txn).await.map_err(refused)?;
            trace(
                &txn,
                &user,
                E::KEY,
                &key_of::<E>(&old),
                &label_of::<E>(&old),
                "delete",
                None,
            )
            .await?;
            txn.commit().await
        }
        (SaveAction::Delete, None) => Err(refused(Refusal::new(RefusalReason::NotFound))),
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

/// A value as the trace shows it: text without its quotes.
fn shown(value: &Value) -> String {
    match sea_value_to_json_value(value) {
        serde_json::Value::String(text) => text,
        other => other.to_string(),
    }
}

/// The key of `row` as text: the trace's `object_pk`.
fn key_of<E: ModelMeta>(row: &E::Model) -> String {
    E::PrimaryKey::iter()
        .map(|key| shown(&row.get(key.into_column())))
        .collect::<Vec<_>>()
        .join(",")
}

/// The label of `row` as the history keeps it: its `LABEL` column, its key
/// when there is none, it is NULL or empty, or it is a secret.
fn label_of<E: ModelMeta>(row: &E::Model) -> String {
    E::LABEL
        .filter(|column| !E::SECRET.iter().any(|s| s.as_str() == column.as_str()))
        .map(|column| row.get(column))
        .filter(|value| *value != value.as_null())
        .map(|value| shown(&value))
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| key_of::<E>(row))
}

/// `{column: {old, new}}` for the columns `changes` sets to another value
/// than `old` holds, sensitive ones masked; `None` when none differs.
fn summary<E: ModelMeta>(old: &E::Model, changes: &E::ActiveModel) -> Option<String> {
    let mut diff = serde_json::Map::new();
    for column in E::Column::iter() {
        let ActiveValue::Set(new) = changes.get(column) else {
            continue;
        };
        let before = old.get(column);
        if before != new {
            diff.insert(
                column.as_str().to_string(),
                serde_json::json!({ "old": shown(&before), "new": shown(&new) }),
            );
        }
    }
    if diff.is_empty() {
        return None;
    }
    redact_sensitive(&mut diff);
    serde_json::to_string(&diff).ok()
}

/// The write's history row, in its transaction: both land, or neither.
async fn trace<C: ConnectionTrait>(
    db: &C,
    user: &CurrentUser,
    resource_key: &str,
    pk: &str,
    repr: &str,
    action: &str,
    summary: Option<String>,
) -> Result<(), DbErr> {
    log_action(
        db,
        ActionLog {
            user_id: user.id,
            username: &user.username,
            resource_key,
            object_pk: pk,
            object_repr: repr,
            action,
            summary,
            batch_id: None,
        },
    )
    .await
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
        if let Some(key) = target {
            form.path_params.insert("id".to_string(), key.to_string());
        }
        for (name, value) in values {
            form.add_value(name, value);
        }
        form
    }

    fn gated(user: &CurrentUser, target: Option<&str>, values: &[(&str, &str)]) -> Forms {
        form_for::<ticket::Entity>(Some(user.clone()), target, values)
    }

    async fn run(form: &mut Forms, action: SaveAction, db: &ADb) -> Result<(), DbErr> {
        save_as::<ticket::Entity>(form, action, db).await
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
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[("title", "x")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    // --- create ---

    #[tokio::test]
    async fn a_create_writes_the_row_and_the_column_defaults_apply() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "fresh"), ("body", "text")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
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
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn the_creator_owns_the_row_when_the_form_has_no_owner_field() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "t")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn the_owner_is_imposed_even_when_its_column_is_not_ticked() {
        let db = setup().await;
        let creator = only_columns(member(pk(10), CREATE), &[("title", true, true)]);
        let forged = pk(20).to_string();
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", &forged)]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(10)));
    }

    #[tokio::test]
    async fn a_create_without_the_create_right_is_refused_and_writes_nothing() {
        let db = setup().await;
        for rights in [NOTHING, UPDATE, UPDATE_OWN, DELETE] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, None, &[("title", "t")]);
            assert_refused(run(&mut form, SaveAction::Save, &db).await);
        }
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_missing_required_column_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", ""), ("body", "b")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
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
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_required_column_the_rights_do_not_tick_is_missing_not_written() {
        let db = setup().await;
        let creator = only_columns(member(pk(10), CREATE), &[("body", true, true)]);
        let mut form = gated(&creator, None, &[("title", "forged"), ("body", "b")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn the_exempt_columns_are_not_reported_as_required() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        // Only `title` is declared: `level` (default), `body` (nullable),
        // `made`, `edited`, `hidden` (exempt) stay unset and the insert goes.
        let mut form = gated(&creator, None, &[("title", "t")]);
        assert!(run(&mut form, SaveAction::Save, &db).await.is_ok());
        assert_eq!(tickets(&db).await.len(), 3);
    }

    #[tokio::test]
    async fn an_unreadable_value_on_create_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
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
        run(&mut form, SaveAction::Save, &db).await.unwrap();
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
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(ticket_of(&db, 1).await.unwrap().owner_id, Some(pk(30)));
    }

    #[tokio::test]
    async fn an_own_update_writes_the_columns_but_never_hands_the_row_over() {
        let db = setup().await;
        let owner = member(pk(10), UPDATE_OWN);
        let to = pk(20).to_string();
        let mut form = gated(&owner, Some("1"), &[("title", "mine"), ("owner_id", &to)]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
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
        run(&mut form, SaveAction::Save, &db).await.unwrap();
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
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn an_update_without_any_right_is_refused() {
        let db = setup().await;
        let before = tickets(&db).await;
        for rights in [NOTHING, CREATE, DELETE, DELETE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[("title", "x")]);
            assert_refused(run(&mut form, SaveAction::Save, &db).await);
        }
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn an_update_without_change_succeeds_and_writes_nothing() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let before = tickets(&db).await;
        let mut form = gated(&editor, Some("1"), &[]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(tickets(&db).await, before, "no stamp: nothing was written");
    }

    #[tokio::test]
    async fn an_update_without_change_still_goes_through_the_gate() {
        let db = setup().await;
        let before = tickets(&db).await;
        let user = member(pk(10), NOTHING);
        let mut form = gated(&user, Some("1"), &[]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let owner = member(pk(10), UPDATE_OWN);
        let mut form = gated(&owner, Some("2"), &[]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
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
        run(&mut form, SaveAction::Save, &db).await.unwrap();
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
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await, before);

        let mut form = gated(&editor, Some("1"), &[("id", "2"), ("title", "t")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(ticket_of(&db, 1).await.unwrap().title, "t");
        assert_eq!(ticket_of(&db, 2).await.unwrap().title, "two");
    }

    #[tokio::test]
    async fn an_unreadable_value_on_update_is_put_on_its_field_and_nothing_is_written() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let before = tickets(&db).await;
        let mut form = gated(&editor, Some("1"), &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
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
        run(&mut form, SaveAction::Delete, &db).await.unwrap();
        assert!(ticket_of(&db, 2).await.is_none());
        assert!(ticket_of(&db, 1).await.is_some());
    }

    #[tokio::test]
    async fn an_own_delete_removes_the_row_of_the_account_and_keeps_another() {
        let db = setup().await;
        let user = member(pk(10), DELETE_OWN);
        let mut form = gated(&user, Some("2"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert!(ticket_of(&db, 2).await.is_some(), "another's row kept");
        let mut form = gated(&user, Some("1"), &[]);
        run(&mut form, SaveAction::Delete, &db).await.unwrap();
        assert!(ticket_of(&db, 1).await.is_none());
    }

    #[tokio::test]
    async fn a_delete_without_the_right_is_refused_and_keeps_the_rows() {
        let db = setup().await;
        let before = tickets(&db).await;
        for rights in [NOTHING, CREATE, UPDATE, UPDATE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[]);
            assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        }
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn deleting_a_missing_row_is_refused() {
        let db = setup().await;
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("99"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    // --- the key ---

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
            for action in [SaveAction::Save, SaveAction::Delete] {
                let mut form = gated(&user, Some(target), &[("title", "x")]);
                assert_refused(run(&mut form, action, &db).await);
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
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_delete_without_a_key_is_refused_and_writes_nothing() {
        let db = setup().await;
        let user = member(pk(10), EVERYTHING);
        let before = tickets(&db).await;
        let mut form = gated(&user, None, &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
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
            run(&mut form, SaveAction::Save, &db).await.unwrap();
            assert_eq!(tickets(&db).await[2].owner_id, Some(pk(id)));

            let mut form = gated(&user, Some("1"), &[("title", "t"), ("owner_id", &to)]);
            run(&mut form, SaveAction::Save, &db).await.unwrap();
            let row = ticket_of(&db, 1).await.unwrap();
            assert_eq!((row.title.as_str(), row.owner_id), ("t", Some(pk(20))));

            let mut form = gated(&user, Some("2"), &[]);
            run(&mut form, SaveAction::Delete, &db).await.unwrap();
            assert!(ticket_of(&db, 2).await.is_none());
        }
    }

    #[tokio::test]
    async fn an_admin_writes_with_the_box_ticked_and_is_refused_without() {
        let db = setup().await;
        let before = tickets(&db).await;
        let without = ranked(member(pk(33), NOTHING), Rang::Admin);
        let mut form = gated(&without, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&without, Some("1"), &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&without, Some("1"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert_eq!(tickets(&db).await, before);

        let with = ranked(member(pk(33), EVERYTHING), Rang::Admin);
        let mut form = gated(&with, None, &[("title", "t")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let mut form = gated(&with, Some("1"), &[("title", "u")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let mut form = gated(&with, Some("2"), &[]);
        run(&mut form, SaveAction::Delete, &db).await.unwrap();
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
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn a_lambda_of_a_group_granting_the_right_writes_and_one_without_is_refused() {
        let db = setup().await;
        let granted = member(pk(31), CREATE);
        let mut form = gated(&granted, None, &[("title", "t")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert_eq!(tickets(&db).await[2].owner_id, Some(pk(31)));

        let mut other_key = member(pk(31), CREATE);
        other_key.groupes[0].permissions[0].resource_key = "other".into();
        let mut form = gated(&other_key, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);

        let mut no_group = member(pk(31), CREATE);
        no_group.groupes.clear();
        let mut form = gated(&no_group, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert_eq!(tickets(&db).await.len(), 3);
    }

    // --- an entity with the default behavior ---

    #[tokio::test]
    async fn an_entity_without_a_custom_before_save_is_written_as_sent() {
        let db = setup().await;
        let boss = ranked(staff(pk(32), "notes", NOTHING), Rang::Dirigeant);
        let to = pk(20).to_string();
        let mut form = form_for::<note::Entity>(Some(boss), Some("1"), &[("owner_id", &to)]);
        save_as::<note::Entity>(&mut form, SaveAction::Save, &db)
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
        save_as::<note::Entity>(&mut form, SaveAction::Save, &db)
            .await
            .unwrap();
        let row = note::Entity::find_by_id(1)
            .one(&*db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.owner_id, Some(pk(10)));
    }

    // --- the trace: entities with a label, a secret, a token ---

    /// `save_vaults`: `nick` is the LABEL, `hash` is SECRET, `api_token` holds
    /// a sensitive name without being declared SECRET.
    mod vault {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "save_vaults")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub name: String,
            pub nick: Option<String>,
            pub api_token: Option<String>,
            pub hash: Option<String>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    impl ModelMeta for vault::Entity {
        const KEY: &'static str = "vaults";
        type Form = TicketForm;
        const SECRET: &'static [vault::Column] = &[vault::Column::Hash];
        const LABEL: Option<vault::Column> = Some(vault::Column::Nick);
        fn schema() -> crate::migration::schema::ModelSchema {
            use crate::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};
            ModelSchema::new("Vault")
                .table_name("save_vaults")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("name").text())
                .column(ColumnDef::new("nick").text().nullable())
                .column(ColumnDef::new("api_token").text().nullable())
                .column(ColumnDef::new("hash").text().nullable())
                .build()
                .unwrap()
        }
    }

    /// `save_locked`: its LABEL is a SECRET column, which must never be shown.
    mod locked {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "save_locked")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub hash: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    impl ModelMeta for locked::Entity {
        const KEY: &'static str = "locked";
        type Form = TicketForm;
        const SECRET: &'static [locked::Column] = &[locked::Column::Hash];
        const LABEL: Option<locked::Column> = Some(locked::Column::Hash);
        fn schema() -> crate::migration::schema::ModelSchema {
            use crate::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};
            ModelSchema::new("Locked")
                .table_name("save_locked")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("hash").text())
                .build()
                .unwrap()
        }
    }

    /// Tickets of `setup()`, plus vaults 1 (nick "Al", token "tok-old",
    /// hash "H1") and 2 (nothing optional set), and locked 1 (hash "S3CR3T").
    async fn setup_vaults() -> ADb {
        let db = setup().await;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        db.execute(&schema.create_table_from_entity(vault::Entity))
            .await
            .unwrap();
        db.execute(&schema.create_table_from_entity(locked::Entity))
            .await
            .unwrap();
        vault::ActiveModel {
            id: Set(1),
            name: Set("alpha".into()),
            nick: Set(Some("Al".into())),
            api_token: Set(Some("tok-old".into())),
            hash: Set(Some("H1".into())),
        }
        .insert(&db)
        .await
        .unwrap();
        vault::ActiveModel {
            id: Set(2),
            name: Set("beta".into()),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        locked::ActiveModel {
            id: Set(1),
            hash: Set("S3CR3T".into()),
        }
        .insert(&db)
        .await
        .unwrap();
        db
    }

    fn member_on<C: ColumnTrait>(id: Pk, key: &str, rights: [bool; 7]) -> CurrentUser {
        let mut user = staff(id, key, rights);
        user.groupes[0].permissions[0].colonnes = all_columns::<C>();
        user
    }

    async fn run_on<E: ModelMeta>(
        user: &CurrentUser,
        target: Option<&str>,
        values: &[(&str, &str)],
        action: SaveAction,
        db: &ADb,
    ) -> Result<(), DbErr>
    where
        E::ActiveModel: Send,
        E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
    {
        let mut form = form_for::<E>(Some(user.clone()), target, values);
        save_as::<E>(&mut form, action, db).await
    }

    async fn history(db: &ADb) -> Vec<crate::auth::history::Model> {
        crate::auth::history::Entity::find()
            .order_by_asc(crate::auth::history::Column::Id)
            .all(&**db)
            .await
            .unwrap()
    }

    async fn vault_of(db: &ADb, id: i32) -> Option<vault::Model> {
        vault::Entity::find_by_id(id).one(&**db).await.unwrap()
    }

    /// Every field of a history row but its summary.
    fn assert_trace(
        row: &crate::auth::history::Model,
        action: &str,
        resource_key: &str,
        object_pk: &str,
        object_repr: &str,
        user: &CurrentUser,
    ) {
        assert_eq!(row.action, action);
        assert_eq!(row.resource_key, resource_key);
        assert_eq!(row.object_pk, object_pk);
        assert_eq!(row.object_repr, object_repr);
        assert_eq!(row.user_id, user.id);
        assert_eq!(row.username, user.username);
        assert_eq!(row.batch_id, None);
    }

    fn summary_of(row: &crate::auth::history::Model) -> serde_json::Value {
        serde_json::from_str(row.summary.as_deref().expect("a summary")).unwrap()
    }

    // --- the trace: create ---

    #[tokio::test]
    async fn a_create_leaves_one_history_row_with_every_field() {
        let db = setup().await;
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "fresh")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        let created = tickets(&db).await[2].id.to_string();
        assert_eq!(created, "3");
        assert_trace(&rows[0], "create", "tickets", &created, &created, &creator);
        assert_eq!(rows[0].summary, None, "a create has no diff");
    }

    #[tokio::test]
    async fn a_create_is_known_by_its_label_or_by_its_key_when_the_label_is_null() {
        let db = setup_vaults().await;
        let creator = member_on::<vault::Column>(pk(10), "vaults", CREATE);
        run_on::<vault::Entity>(
            &creator,
            None,
            &[("name", "gamma"), ("nick", "Gam")],
            SaveAction::Save,
            &db,
        )
        .await
        .unwrap();
        run_on::<vault::Entity>(
            &creator,
            None,
            &[("name", "delta")],
            SaveAction::Save,
            &db,
        )
        .await
        .unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 2);
        assert_trace(&rows[0], "create", "vaults", "3", "Gam", &creator);
        assert_trace(&rows[1], "create", "vaults", "4", "4", &creator);
    }

    #[tokio::test]
    async fn a_refused_create_leaves_no_history_and_no_row() {
        let db = setup().await;
        for rights in [NOTHING, UPDATE, UPDATE_OWN, DELETE] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, None, &[("title", "t")]);
            assert_refused(run(&mut form, SaveAction::Save, &db).await);
        }
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", ""), ("body", "b")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&creator, None, &[("body", "b")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&creator, None, &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = form_for::<ticket::Entity>(None, None, &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        // A key makes it an update, which `CREATE` alone doesn't grant.
        let mut form = gated(&creator, Some("1"), &[("title", "t")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert!(history(&db).await.is_empty());
        assert_eq!(tickets(&db).await.len(), 2);
    }

    #[tokio::test]
    async fn the_database_net_on_a_create_leaves_no_history() {
        // `body` is nullable for the schema but NOT NULL in the table: the
        // insert fails after `missing_required` let it through.
        let db = setup().await;
        db.execute_unprepared("DROP TABLE save_tickets")
            .await
            .unwrap();
        db.execute_unprepared(
            "CREATE TABLE save_tickets (id INTEGER PRIMARY KEY AUTOINCREMENT, owner_id INTEGER, \
             title TEXT NOT NULL, body TEXT NOT NULL, level INTEGER NOT NULL DEFAULT 0, \
             stamp TEXT, made TEXT, edited TEXT, hidden TEXT)",
        )
        .await
        .unwrap();
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "t")]);
        assert!(run(&mut form, SaveAction::Save, &db).await.is_err());
        assert!(history(&db).await.is_empty());
        assert!(tickets(&db).await.is_empty());
    }

    // --- the trace: update ---

    #[tokio::test]
    async fn an_update_traces_only_the_columns_that_really_changed() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let same_owner = pk(10).to_string();
        let mut form = gated(
            &editor,
            Some("1"),
            &[
                ("title", "renamed"),
                ("body", "b"),
                ("owner_id", &same_owner),
            ],
        );
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        assert_trace(&rows[0], "update", "tickets", "1", "1", &editor);
        assert_eq!(
            summary_of(&rows[0]),
            serde_json::json!({
                "title": { "old": "one", "new": "renamed" },
                "body": { "old": "null", "new": "b" },
            }),
            "owner_id sent back unchanged and the stamp are not in it"
        );
    }

    #[tokio::test]
    async fn an_update_traces_a_reassigned_owner_with_old_and_new() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let to = pk(30).to_string();
        let mut form = gated(&editor, Some("1"), &[("owner_id", &to)]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(
            summary_of(&rows[0]),
            serde_json::json!({
                "owner_id": { "old": pk(10).to_string(), "new": to },
            })
        );
    }

    #[tokio::test]
    async fn an_update_sending_the_current_values_traces_without_a_summary() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let mut form = gated(&editor, Some("1"), &[("title", "one")]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        assert_trace(&rows[0], "update", "tickets", "1", "1", &editor);
        assert_eq!(rows[0].summary, None);
        assert_eq!(
            ticket_of(&db, 1).await.unwrap().stamp.as_deref(),
            Some("stamped"),
            "the write itself still happened"
        );
    }

    #[tokio::test]
    async fn an_update_without_change_traces_nothing() {
        let db = setup().await;
        let editor = member(pk(10), UPDATE);
        let mut form = gated(&editor, Some("1"), &[]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        let owner = member(pk(10), UPDATE_OWN);
        let to = pk(20).to_string();
        let mut form = gated(&owner, Some("1"), &[("owner_id", &to)]);
        run(&mut form, SaveAction::Save, &db).await.unwrap();
        assert!(history(&db).await.is_empty());
    }

    #[tokio::test]
    async fn a_refused_update_leaves_no_history_and_no_change() {
        let db = setup().await;
        let before = tickets(&db).await;
        let owner = member(pk(10), UPDATE_OWN);
        let mut form = gated(&owner, Some("2"), &[("title", "taken")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        for rights in [NOTHING, CREATE, DELETE, DELETE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[("title", "x")]);
            assert_refused(run(&mut form, SaveAction::Save, &db).await);
        }
        let editor = member(pk(10), UPDATE);
        let mut form = gated(&editor, Some("99"), &[("title", "ghost")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&editor, Some("abc"), &[("title", "x")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = gated(&editor, Some("1"), &[("title", "t"), ("owner_id", "abc")]);
        assert_fields_refused(run(&mut form, SaveAction::Save, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[("title", "x")]);
        assert_refused(run(&mut form, SaveAction::Save, &db).await);
        assert!(history(&db).await.is_empty());
        assert_eq!(tickets(&db).await, before);
    }

    #[tokio::test]
    async fn a_sensitive_column_is_listed_masked_never_with_its_value() {
        let db = setup_vaults().await;
        let editor = member_on::<vault::Column>(pk(10), "vaults", UPDATE);
        run_on::<vault::Entity>(
            &editor,
            Some("1"),
            &[
                ("name", "alpha2"),
                ("api_token", "tok-new"),
                ("hash", "HACK"),
            ],
            SaveAction::Save,
            &db,
        )
        .await
        .unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        let masked = crate::auth::history::REDACTED;
        assert_eq!(
            summary_of(&rows[0]),
            serde_json::json!({
                "name": { "old": "alpha", "new": "alpha2" },
                "api_token": { "old": masked, "new": masked },
            }),
            "the SECRET hash is never written, so never traced"
        );
        let raw = rows[0].summary.as_deref().unwrap();
        for leaked in ["tok-old", "tok-new", "H1", "HACK"] {
            assert!(!raw.contains(leaked), "{leaked} leaked in {raw}");
        }
        let row = vault_of(&db, 1).await.unwrap();
        assert_eq!(row.api_token.as_deref(), Some("tok-new"));
        assert_eq!(row.hash.as_deref(), Some("H1"));
    }

    #[tokio::test]
    async fn an_update_is_known_by_its_old_label_and_the_new_one_is_in_the_summary() {
        let db = setup_vaults().await;
        let editor = member_on::<vault::Column>(pk(10), "vaults", UPDATE);
        run_on::<vault::Entity>(
            &editor,
            Some("1"),
            &[("nick", "Bo")],
            SaveAction::Save,
            &db,
        )
        .await
        .unwrap();
        run_on::<vault::Entity>(
            &editor,
            Some("2"),
            &[("name", "beta2")],
            SaveAction::Save,
            &db,
        )
        .await
        .unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 2);
        assert_trace(&rows[0], "update", "vaults", "1", "Al", &editor);
        assert_eq!(
            summary_of(&rows[0]),
            serde_json::json!({ "nick": { "old": "Al", "new": "Bo" } })
        );
        assert_trace(&rows[1], "update", "vaults", "2", "2", &editor);
    }

    // --- the trace: delete ---

    #[tokio::test]
    async fn a_delete_leaves_one_history_row_with_the_deleted_rows_label() {
        let db = setup_vaults().await;
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("2"), &[]);
        run(&mut form, SaveAction::Delete, &db).await.unwrap();
        let vaulter = member_on::<vault::Column>(pk(10), "vaults", DELETE);
        run_on::<vault::Entity>(&vaulter, Some("1"), &[], SaveAction::Delete, &db)
            .await
            .unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 2);
        assert_trace(&rows[0], "delete", "tickets", "2", "2", &user);
        assert_eq!(rows[0].summary, None);
        assert_trace(&rows[1], "delete", "vaults", "1", "Al", &vaulter);
        assert_eq!(rows[1].summary, None);
        assert!(vault_of(&db, 1).await.is_none());
    }

    #[tokio::test]
    async fn an_own_delete_is_traced_as_the_account_that_made_it() {
        let db = setup().await;
        let user = member(pk(10), DELETE_OWN);
        let mut form = gated(&user, Some("1"), &[]);
        run(&mut form, SaveAction::Delete, &db).await.unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        assert_trace(&rows[0], "delete", "tickets", "1", "1", &user);
    }

    #[tokio::test]
    async fn a_label_that_is_a_secret_column_is_replaced_by_the_key() {
        let db = setup_vaults().await;
        let user = member_on::<locked::Column>(pk(10), "locked", DELETE);
        run_on::<locked::Entity>(&user, Some("1"), &[], SaveAction::Delete, &db)
            .await
            .unwrap();
        let rows = history(&db).await;
        assert_eq!(rows.len(), 1);
        assert_trace(&rows[0], "delete", "locked", "1", "1", &user);
        assert!(!format!("{:?}", rows[0]).contains("S3CR3T"));
    }

    #[tokio::test]
    async fn a_refused_delete_leaves_no_history_and_keeps_the_rows() {
        let db = setup().await;
        let before = tickets(&db).await;
        for rights in [NOTHING, CREATE, UPDATE, UPDATE_OWN] {
            let user = member(pk(10), rights);
            let mut form = gated(&user, Some("1"), &[]);
            assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        }
        let own = member(pk(10), DELETE_OWN);
        let mut form = gated(&own, Some("2"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("99"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        let mut form = gated(&user, Some("1/2"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        let mut form = form_for::<ticket::Entity>(None, Some("1"), &[]);
        assert_refused(run(&mut form, SaveAction::Delete, &db).await);
        assert!(history(&db).await.is_empty());
        assert_eq!(tickets(&db).await, before);
    }

    // --- no write without its trace ---

    #[tokio::test]
    async fn a_create_whose_trace_fails_is_rolled_back() {
        let db = setup().await;
        db.execute_unprepared("DROP TABLE eihwaz_history")
            .await
            .unwrap();
        let creator = member(pk(10), CREATE);
        let mut form = gated(&creator, None, &[("title", "fresh")]);
        let result = run(&mut form, SaveAction::Save, &db).await;
        assert!(
            matches!(result, Err(ref e) if !matches!(e, DbErr::Custom(_))),
            "{result:?}"
        );
        assert_eq!(tickets(&db).await.len(), 2, "the row was not kept");
    }

    #[tokio::test]
    async fn an_update_whose_trace_fails_is_rolled_back() {
        let db = setup().await;
        db.execute_unprepared("DROP TABLE eihwaz_history")
            .await
            .unwrap();
        let before = tickets(&db).await;
        let editor = member(pk(10), UPDATE);
        let mut form = gated(&editor, Some("1"), &[("title", "renamed")]);
        let result = run(&mut form, SaveAction::Save, &db).await;
        assert!(
            matches!(result, Err(ref e) if !matches!(e, DbErr::Custom(_))),
            "{result:?}"
        );
        assert_eq!(
            tickets(&db).await,
            before,
            "neither the title nor the stamp"
        );
    }

    #[tokio::test]
    async fn a_delete_whose_trace_fails_is_rolled_back() {
        let db = setup().await;
        db.execute_unprepared("DROP TABLE eihwaz_history")
            .await
            .unwrap();
        let before = tickets(&db).await;
        let user = member(pk(10), DELETE);
        let mut form = gated(&user, Some("2"), &[]);
        let result = run(&mut form, SaveAction::Delete, &db).await;
        assert!(
            matches!(result, Err(ref e) if !matches!(e, DbErr::Custom(_))),
            "{result:?}"
        );
        assert_eq!(tickets(&db).await, before, "the row is still there");
    }

    #[test]
    fn shown_gives_a_text_without_quotes_and_the_json_of_anything_else() {
        assert_eq!(shown(&Value::String(Some("abc".into()))), "abc");
        assert_eq!(
            shown(&Value::String(Some("a\"b".into()))),
            "a\"b",
            "the text itself, not its JSON escape"
        );
        assert_eq!(shown(&Value::String(Some(String::new()))), "");
        assert_eq!(shown(&Value::Int(Some(42))), "42");
        assert_eq!(shown(&Value::BigInt(Some(-7))), "-7");
        assert_eq!(shown(&Value::Bool(Some(true))), "true");
        assert_eq!(shown(&Value::Int(None)), "null");
        assert_eq!(shown(&Value::String(None)), "null");
    }

    fn ticket_row(id: i32) -> ticket::Model {
        ticket::Model {
            id,
            owner_id: None,
            title: "t".into(),
            body: None,
            level: 0,
            stamp: None,
            made: None,
            edited: None,
            hidden: None,
        }
    }

    fn vault_row(id: i32, nick: Option<&str>) -> vault::Model {
        vault::Model {
            id,
            name: "n".into(),
            nick: nick.map(str::to_string),
            api_token: None,
            hash: None,
        }
    }

    #[test]
    fn key_of_is_the_key_as_text() {
        assert_eq!(key_of::<ticket::Entity>(&ticket_row(7)), "7");
        assert_eq!(key_of::<ticket::Entity>(&ticket_row(0)), "0");
        assert_eq!(key_of::<vault::Entity>(&vault_row(123, None)), "123");
    }

    #[test]
    fn label_of_is_the_label_column_and_falls_back_to_the_key() {
        assert_eq!(label_of::<vault::Entity>(&vault_row(5, Some("Al"))), "Al");
        assert_eq!(label_of::<vault::Entity>(&vault_row(5, None)), "5", "NULL");
        assert_eq!(
            label_of::<vault::Entity>(&vault_row(5, Some(""))),
            "5",
            "empty"
        );
        assert_eq!(
            label_of::<vault::Entity>(&vault_row(5, Some("null"))),
            "null",
            "the text \"null\" is a label, only a real NULL falls back"
        );
        assert_eq!(label_of::<vault::Entity>(&vault_row(5, Some(" "))), " ");
        assert_eq!(label_of::<ticket::Entity>(&ticket_row(9)), "9", "no LABEL");
    }

    #[test]
    fn label_of_never_shows_a_secret_column() {
        let row = locked::Model {
            id: 4,
            hash: "S3CR3T".into(),
        };
        assert_eq!(label_of::<locked::Entity>(&row), "4");
    }
}
