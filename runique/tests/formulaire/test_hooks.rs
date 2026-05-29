//! Tests — hooks `before_save` / `after_save` / `save_as` sur `RuniqueForm`.
//!
//! Couvre :
//!   - ordre d'appel : before_save → on_save → after_save → commit
//!   - rollback automatique si before_save échoue
//!   - rollback automatique si on_save échoue
//!   - rollback automatique si after_save échoue
//!   - SaveContext transmis correctement

use crate::helpers::db;
use async_trait::async_trait;
use runique::{
    forms::{field::RuniqueForm, form::Forms},
    prelude::SaveContext,
    sea_orm::{ConnectionTrait, DatabaseTransaction, DbErr},
};

// ═══════════════════════════════════════════════════════════════
// Forme minimale avec suivi des appels et injections d'erreurs
// ═══════════════════════════════════════════════════════════════

struct TrackerForm {
    form: Forms,
    fail_before: bool,
    fail_save: bool,
    fail_after: bool,
    /// Appels enregistrés dans l'ordre
    calls: Vec<String>,
    /// SaveContext reçus (before + after)
    contexts: Vec<SaveContext>,
    /// SQL exécuté dans before_save (si Some)
    before_sql: Option<String>,
    /// SQL exécuté dans on_save (si Some)
    sql: Option<String>,
}

impl TrackerForm {
    fn new() -> Self {
        let mut form = Forms::new("csrf_test");
        form.mark_validated();
        Self {
            form,
            fail_before: false,
            fail_save: false,
            fail_after: false,
            calls: Vec::new(),
            contexts: Vec::new(),
            before_sql: None,
            sql: None,
        }
    }

    fn with_fail_before(mut self) -> Self {
        self.fail_before = true;
        self
    }

    fn with_fail_save(mut self) -> Self {
        self.fail_save = true;
        self
    }

    fn with_fail_after(mut self) -> Self {
        self.fail_after = true;
        self
    }

    fn with_before_sql(mut self, sql: impl Into<String>) -> Self {
        self.before_sql = Some(sql.into());
        self
    }

    fn with_sql(mut self, sql: impl Into<String>) -> Self {
        self.sql = Some(sql.into());
        self
    }
}

#[async_trait]
impl RuniqueForm for TrackerForm {
    fn register_fields(_form: &mut Forms) {}

    fn from_form(form: Forms) -> Self {
        Self {
            form,
            fail_before: false,
            fail_save: false,
            fail_after: false,
            calls: Vec::new(),
            contexts: Vec::new(),
            before_sql: None,
            sql: None,
        }
    }

    fn get_form(&self) -> &Forms {
        &self.form
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }

    async fn before_save(
        &mut self,
        ctx: SaveContext,
        txn: &DatabaseTransaction,
    ) -> Result<(), DbErr> {
        self.calls.push("before".to_string());
        self.contexts.push(ctx);
        if let Some(sql) = &self.before_sql {
            txn.execute_unprepared(sql.as_str())
                .await
                .map_err(|e: DbErr| DbErr::Custom(e.to_string()))?;
        }
        if self.fail_before {
            return Err(DbErr::Custom("before_save forced failure".to_string()));
        }
        Ok(())
    }

    async fn on_save(&mut self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        self.calls.push("save".to_string());
        if let Some(sql) = &self.sql {
            txn.execute_unprepared(sql.as_str())
                .await
                .map_err(|e: DbErr| DbErr::Custom(e.to_string()))?;
        }
        if self.fail_save {
            return Err(DbErr::Custom("on_save forced failure".to_string()));
        }
        Ok(())
    }

    async fn after_save(
        &mut self,
        ctx: SaveContext,
        _txn: &DatabaseTransaction,
    ) -> Result<(), DbErr> {
        self.calls.push("after".to_string());
        self.contexts.push(ctx);
        if self.fail_after {
            return Err(DbErr::Custom("after_save forced failure".to_string()));
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// Ordre d'appel
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_save_as_ordre_appel() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new();
    form.save_as(SaveContext::Create, &db).await.unwrap();
    assert_eq!(form.calls, ["before", "save", "after"]);
}

#[tokio::test]
async fn test_save_as_contexte_transmis_create() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new();
    form.save_as(SaveContext::Create, &db).await.unwrap();
    assert_eq!(form.contexts, [SaveContext::Create, SaveContext::Create]);
}

#[tokio::test]
async fn test_save_as_contexte_transmis_update() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new();
    form.save_as(SaveContext::Update, &db).await.unwrap();
    assert_eq!(form.contexts, [SaveContext::Update, SaveContext::Update]);
}

#[tokio::test]
async fn test_save_as_contexte_transmis_delete() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new();
    form.save_as(SaveContext::Delete, &db).await.unwrap();
    assert_eq!(form.contexts, [SaveContext::Delete, SaveContext::Delete]);
}

// ═══════════════════════════════════════════════════════════════
// Rollback — before_save échoue
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_before_save_echoue_retourne_err() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new().with_fail_before();
    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_before_save_echoue_save_non_appele() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new().with_fail_before();
    let _ = form.save_as(SaveContext::Create, &db).await;
    assert!(!form.calls.contains(&"save".to_string()));
    assert!(!form.calls.contains(&"after".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// Rollback — on_save échoue
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_on_save_echoue_retourne_err() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new().with_fail_save();
    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_on_save_echoue_after_non_appele() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new().with_fail_save();
    let _ = form.save_as(SaveContext::Create, &db).await;
    assert!(!form.calls.contains(&"after".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// Rollback — after_save échoue
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_after_save_echoue_retourne_err() {
    let db = db::fresh_db().await;
    let mut form = TrackerForm::new().with_fail_after();
    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_after_save_echoue_rollback_db() {
    // on_save insère une ligne — after_save échoue → rollback → table vide
    let db =
        db::fresh_db_with_schema("CREATE TABLE hook_test (id INTEGER PRIMARY KEY, val TEXT)").await;

    let mut form = TrackerForm::new()
        .with_sql("INSERT INTO hook_test (val) VALUES ('test')")
        .with_fail_after();

    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
    db::assert_count(&db, "hook_test", 0).await;
}

#[tokio::test]
async fn test_commit_si_tous_hooks_ok() {
    // on_save insère une ligne — tout réussit → commit → ligne présente
    let db =
        db::fresh_db_with_schema("CREATE TABLE hook_test_ok (id INTEGER PRIMARY KEY, val TEXT)")
            .await;

    let mut form = TrackerForm::new().with_sql("INSERT INTO hook_test_ok (val) VALUES ('test')");

    form.save_as(SaveContext::Create, &db).await.unwrap();
    db::assert_count(&db, "hook_test_ok", 1).await;
}

#[tokio::test]
async fn test_on_save_echoue_rollback_db() {
    // on_save insère une ligne puis échoue → rollback → table vide
    let db = db::fresh_db_with_schema(
        "CREATE TABLE hook_test_save_fail (id INTEGER PRIMARY KEY, val TEXT)",
    )
    .await;

    let mut form = TrackerForm::new()
        .with_sql("INSERT INTO hook_test_save_fail (val) VALUES ('test')")
        .with_fail_save();

    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
    db::assert_count(&db, "hook_test_save_fail", 0).await;
}

#[tokio::test]
async fn test_before_save_echoue_rollback_db() {
    // before_save insère une ligne puis échoue → rollback → table vide
    let db = db::fresh_db_with_schema(
        "CREATE TABLE hook_test_before_fail (id INTEGER PRIMARY KEY, val TEXT)",
    )
    .await;

    let mut form = TrackerForm::new()
        .with_before_sql("INSERT INTO hook_test_before_fail (val) VALUES ('test')")
        .with_fail_before();

    let result = form.save_as(SaveContext::Create, &db).await;
    assert!(result.is_err());
    db::assert_count(&db, "hook_test_before_fail", 0).await;
}
