// Tests pour ModelSchema et SchemaDiff

use runique::forms::Forms;
use runique::migration::{
    column::ColumnDef,
    foreign_key::ForeignKeyDef,
    hooks::HooksDef,
    index::IndexDef,
    primary_key::PrimaryKeyDef,
    relation::RelationDef,
    schema::{ModelSchema, SchemaDiff},
};
use runique::sea_orm::{
    ConnectionTrait,
    sea_query::{Alias, Query},
};

use crate::helpers::db;
use crate::helpers::db_mariadb as db_maria;
use crate::helpers::db_postgres as db_pg;
use serial_test::serial;

// ═══════════════════════════════════════════════════════════════
// ModelSchema::new() — conversion PascalCase → snake_case
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_new_pascal_case_simple() {
    let s = ModelSchema::new("User");
    assert_eq!(s.model_name, "User");
    assert_eq!(s.table_name, "user");
}

#[test]
fn test_schema_new_pascal_case_compose() {
    let s = ModelSchema::new("BlogPost");
    assert_eq!(s.model_name, "BlogPost");
    assert_eq!(s.table_name, "blog_post");
}

#[test]
fn test_schema_new_defauts() {
    let s = ModelSchema::new("Article");
    assert!(s.primary_key.is_none());
    assert!(s.columns.is_empty());
    assert!(s.foreign_keys.is_empty());
    assert!(s.relations.is_empty());
    assert!(s.indexes.is_empty());
    assert!(s.hooks.is_none());
    assert!(s.schema.is_none());
}

// ═══════════════════════════════════════════════════════════════
// Builders — table_name, schema
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_table_name_override() {
    let s = ModelSchema::new("User").table_name("custom_users");
    assert_eq!(s.table_name, "custom_users");
}

#[test]
fn test_schema_set_schema() {
    let s = ModelSchema::new("User").schema("public");
    assert_eq!(s.schema.as_deref(), Some("public"));
}

// ═══════════════════════════════════════════════════════════════
// Builders — primary_key, column, foreign_key, relation, index, hooks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_primary_key() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id"));
    assert!(s.primary_key.is_some());
    assert_eq!(s.primary_key.unwrap().name, "id");
}

#[test]
fn test_schema_column_ajout() {
    let s = ModelSchema::new("User").column(ColumnDef::new("username").string());
    assert_eq!(s.columns.len(), 1);
    assert_eq!(s.columns[0].name, "username");
}

#[test]
fn test_schema_multi_columns() {
    let s = ModelSchema::new("Post")
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("body").text());
    assert_eq!(s.columns.len(), 2);
}

#[test]
fn test_schema_foreign_key_ajout() {
    let s = ModelSchema::new("Post").foreign_key(ForeignKeyDef::new("user_id").references("users"));
    assert_eq!(s.foreign_keys.len(), 1);
    assert_eq!(s.foreign_keys[0].from_column, "user_id");
}

#[test]
fn test_schema_relation_ajout() {
    let s = ModelSchema::new("Post").relation(RelationDef::has_one("profile"));
    assert_eq!(s.relations.len(), 1);
}

#[test]
fn test_schema_index_ajout() {
    let s = ModelSchema::new("User").index(IndexDef::new(vec!["email"]).unique());
    assert_eq!(s.indexes.len(), 1);
    assert!(s.indexes[0].unique);
}

#[test]
fn test_schema_hooks_ajout() {
    let s = ModelSchema::new("User").hooks(HooksDef::new().before_save(0, "handler"));
    assert!(s.hooks.is_some());
    assert_eq!(s.hooks.unwrap().hooks.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// build()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_build_sans_pk_retourne_err() {
    let result = ModelSchema::new("User").build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing primary key"));
}

#[test]
fn test_schema_build_avec_pk_retourne_ok() {
    let result = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .build();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().model_name, "User");
}

// ═══════════════════════════════════════════════════════════════
// diff()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_identiques_est_vide() {
    let s1 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let s2 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = s1.diff(&s2);
    assert!(diff.is_empty());
}

#[test]
fn test_schema_diff_colonne_ajoutee() {
    let old = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let new = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.added_columns.len(), 1);
    assert_eq!(diff.added_columns[0].name, "email");
    assert!(diff.dropped_columns.is_empty());
}

#[test]
fn test_schema_diff_colonne_supprimee() {
    let old = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let new = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.dropped_columns.len(), 1);
    assert_eq!(diff.dropped_columns[0], "email");
    assert!(diff.added_columns.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// SchemaDiff
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_new_est_vide() {
    let diff = SchemaDiff::new("users");
    assert_eq!(diff.table_name, "users");
    assert!(diff.is_empty());
    assert!(diff.added_columns.is_empty());
    assert!(diff.dropped_columns.is_empty());
    assert!(diff.modified_columns.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// to_migration() — exécution réelle contre une vraie DB (pas juste "ne panique pas")
//
// `to_migration()` retourne un `TableCreateStatement` typé, pas du texte à
// parser : le vrai test consiste à l'exécuter pour de vrai (comme le ferait un
// utilisateur du framework), puis à insérer/relire une ligne réelle — ça prouve
// que les colonnes existent avec des types compatibles, pas seulement que le
// builder n'a pas paniqué en mémoire. SQLite en mémoire ci-dessous, Postgres et
// MariaDB via Docker plus bas.
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_schema_to_migration_creates_real_table_sqlite() {
    let conn = db::fresh_db().await;

    let schema = ModelSchema::new("Widget")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("view_count").integer())
        .column(ColumnDef::new("active").boolean());

    conn.execute(&schema.to_migration())
        .await
        .expect("to_migration() doit produire un CREATE TABLE réellement accepté par SQLite");

    // Insère comme le ferait un vrai appelant, puis relit — prouve que les
    // colonnes existent avec les bons types, pas seulement en mémoire.
    db::exec(
        &conn,
        "INSERT INTO widget (title, view_count, active) VALUES ('Widget A', 42, 1)",
    )
    .await;

    let row = conn
        .query_one(
            &Query::select()
                .columns([
                    Alias::new("title"),
                    Alias::new("view_count"),
                    Alias::new("active"),
                ])
                .from(Alias::new("widget"))
                .to_owned(),
        )
        .await
        .expect("select échoué")
        .expect("la ligne insérée doit être relisible depuis la vraie table");

    assert_eq!(row.try_get::<String>("", "title").unwrap(), "Widget A");
    assert_eq!(row.try_get::<i32>("", "view_count").unwrap(), 42);
    assert!(row.try_get::<bool>("", "active").unwrap());
}

#[tokio::test]
#[serial]
async fn test_schema_to_migration_creates_real_table_postgres() {
    let Some(conn) = db_pg::connect().await else {
        return;
    };
    db_pg::exec(&conn, "DROP TABLE IF EXISTS widget").await;

    let schema = ModelSchema::new("Widget")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("view_count").integer())
        .column(ColumnDef::new("active").boolean());

    conn.execute(&schema.to_migration())
        .await
        .expect("to_migration() doit produire un CREATE TABLE réellement accepté par Postgres");

    db_pg::exec(
        &conn,
        "INSERT INTO widget (title, view_count, active) VALUES ('Widget A', 42, true)",
    )
    .await;

    let row = conn
        .query_one(
            &Query::select()
                .columns([
                    Alias::new("title"),
                    Alias::new("view_count"),
                    Alias::new("active"),
                ])
                .from(Alias::new("widget"))
                .to_owned(),
        )
        .await
        .expect("select échoué")
        .expect("la ligne insérée doit être relisible depuis la vraie table");

    assert_eq!(row.try_get::<String>("", "title").unwrap(), "Widget A");
    assert_eq!(row.try_get::<i32>("", "view_count").unwrap(), 42);
    assert!(row.try_get::<bool>("", "active").unwrap());

    db_pg::exec(&conn, "DROP TABLE IF EXISTS widget").await;
}

#[tokio::test]
#[serial]
async fn test_schema_to_migration_creates_real_table_mariadb() {
    let Some(conn) = db_maria::connect().await else {
        return;
    };
    db_maria::exec(&conn, "DROP TABLE IF EXISTS widget").await;

    let schema = ModelSchema::new("Widget")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("view_count").integer())
        .column(ColumnDef::new("active").boolean());

    conn.execute(&schema.to_migration())
        .await
        .expect("to_migration() doit produire un CREATE TABLE réellement accepté par MariaDB");

    db_maria::exec(
        &conn,
        "INSERT INTO widget (title, view_count, active) VALUES ('Widget A', 42, true)",
    )
    .await;

    let row = conn
        .query_one(
            &Query::select()
                .columns([
                    Alias::new("title"),
                    Alias::new("view_count"),
                    Alias::new("active"),
                ])
                .from(Alias::new("widget"))
                .to_owned(),
        )
        .await
        .expect("select échoué")
        .expect("la ligne insérée doit être relisible depuis la vraie table");

    assert_eq!(row.try_get::<String>("", "title").unwrap(), "Widget A");
    assert_eq!(row.try_get::<i32>("", "view_count").unwrap(), 42);
    assert!(row.try_get::<bool>("", "active").unwrap());

    db_maria::exec(&conn, "DROP TABLE IF EXISTS widget").await;
}

// ── FK — vraiment contrainte en DB, pas juste déclarée ─────────────────────────
//
// `to_migration()` ajoute la FK inline (`table.foreign_key(...)` dans le même
// CREATE TABLE) — le test ne se contente pas de vérifier qu'elle est créée sans
// erreur : il insère une ligne référençant un parent inexistant et s'attend à
// un vrai rejet de la DB, la seule preuve que la contrainte est réellement active.

#[tokio::test]
#[serial]
async fn test_schema_to_migration_fk_is_enforced_postgres() {
    let Some(conn) = db_pg::connect().await else {
        return;
    };
    db_pg::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_post").await;
    db_pg::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_user").await;

    let parent = ModelSchema::new("RqTestFkUser")
        .table_name("rq_test_fk_user")
        .primary_key(PrimaryKeyDef::new("id"));
    conn.execute(&parent.to_migration())
        .await
        .expect("création de la table parent");

    let child = ModelSchema::new("RqTestFkPost")
        .table_name("rq_test_fk_post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("user_id").integer())
        .foreign_key(ForeignKeyDef::new("user_id").references("rq_test_fk_user"));
    conn.execute(&child.to_migration())
        .await
        .expect("création de la table enfant avec FK inline");

    db_pg::exec(&conn, "INSERT INTO rq_test_fk_user (id) VALUES (1)").await;
    db_pg::exec(&conn, "INSERT INTO rq_test_fk_post (user_id) VALUES (1)").await;

    let result = conn
        .execute_unprepared("INSERT INTO rq_test_fk_post (user_id) VALUES (999)")
        .await;
    assert!(
        result.is_err(),
        "la contrainte FK doit rejeter une référence vers un parent inexistant"
    );

    db_pg::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_post").await;
    db_pg::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_user").await;
}

#[tokio::test]
#[serial]
async fn test_schema_to_migration_fk_is_enforced_mariadb() {
    let Some(conn) = db_maria::connect().await else {
        return;
    };
    db_maria::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_post").await;
    db_maria::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_user").await;

    let parent = ModelSchema::new("RqTestFkUser")
        .table_name("rq_test_fk_user")
        .primary_key(PrimaryKeyDef::new("id"));
    conn.execute(&parent.to_migration())
        .await
        .expect("création de la table parent");

    let child = ModelSchema::new("RqTestFkPost")
        .table_name("rq_test_fk_post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("user_id").integer())
        .foreign_key(ForeignKeyDef::new("user_id").references("rq_test_fk_user"));
    conn.execute(&child.to_migration())
        .await
        .expect("création de la table enfant avec FK inline");

    db_maria::exec(&conn, "INSERT INTO rq_test_fk_user (id) VALUES (1)").await;
    db_maria::exec(&conn, "INSERT INTO rq_test_fk_post (user_id) VALUES (1)").await;

    let result = conn
        .execute_unprepared("INSERT INTO rq_test_fk_post (user_id) VALUES (999)")
        .await;
    assert!(
        result.is_err(),
        "la contrainte FK doit rejeter une référence vers un parent inexistant"
    );

    db_maria::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_post").await;
    db_maria::exec(&conn, "DROP TABLE IF EXISTS rq_test_fk_user").await;
}

#[tokio::test]
async fn test_schema_to_migration_fk_is_enforced_sqlite() {
    let conn = db::fresh_db().await;
    // SQLite n'impose les FK que si le pragma est activé sur la connexion —
    // sinon la contrainte est acceptée en DDL mais jamais vérifiée à l'écriture.
    conn.execute_unprepared("PRAGMA foreign_keys = ON;")
        .await
        .expect("activation du pragma FK");

    let parent = ModelSchema::new("RqTestFkUser")
        .table_name("rq_test_fk_user")
        .primary_key(PrimaryKeyDef::new("id"));
    conn.execute(&parent.to_migration())
        .await
        .expect("création de la table parent");

    let child = ModelSchema::new("RqTestFkPost")
        .table_name("rq_test_fk_post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("user_id").integer())
        .foreign_key(ForeignKeyDef::new("user_id").references("rq_test_fk_user"));
    conn.execute(&child.to_migration())
        .await
        .expect("création de la table enfant avec FK inline");

    db::exec(&conn, "INSERT INTO rq_test_fk_user (id) VALUES (1)").await;
    db::exec(&conn, "INSERT INTO rq_test_fk_post (user_id) VALUES (1)").await;

    let result = conn
        .execute_unprepared("INSERT INTO rq_test_fk_post (user_id) VALUES (999)")
        .await;
    assert!(
        result.is_err(),
        "la contrainte FK doit rejeter une référence vers un parent inexistant"
    );
}

// ── Colonne ignorée — vraiment absente de la table, pas juste hors du builder ──

#[tokio::test]
async fn test_schema_to_migration_ignored_column_is_really_absent() {
    let conn = db::fresh_db().await;

    let schema = ModelSchema::new("Secretive")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("internal_cache").string().ignore());

    conn.execute(&schema.to_migration())
        .await
        .expect("création de la table");

    // Un vrai appelant qui sélectionne la colonne ignorée doit obtenir une
    // vraie erreur SQL — preuve qu'elle n'existe pas dans la table réelle,
    // pas seulement que le builder ne l'a pas émise en mémoire.
    let result = conn
        .execute_unprepared("SELECT internal_cache FROM secretive")
        .await;
    assert!(
        result.is_err(),
        "la colonne ignorée ne doit pas exister dans la vraie table"
    );
}

// ═══════════════════════════════════════════════════════════════
// to_model() — contenu de la chaîne générée
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_contient_struct_model() {
    let s = ModelSchema::new("Article").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(code.contains("pub struct Model"));
}

#[test]
fn test_schema_to_model_contient_table_name() {
    let s = ModelSchema::new("BlogPost").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(
        code.contains("blog_post"),
        "doit contenir le nom de table snake_case"
    );
}

#[test]
fn test_schema_to_model_pk_i32() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id").i32());
    let code = s.to_model();
    assert!(code.contains("i32"));
}

#[test]
fn test_schema_to_model_pk_i64() {
    let s = ModelSchema::new("BigTable").primary_key(PrimaryKeyDef::new("id").i64());
    let code = s.to_model();
    assert!(code.contains("i64"));
}

#[test]
fn test_schema_to_model_colonne_nullable() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("bio").text().nullable());
    let code = s.to_model();
    assert!(
        code.contains("Option<"),
        "colonne nullable doit générer Option<T>"
    );
}

#[test]
fn test_schema_to_model_colonne_ignoree_absente() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("internal_cache").string().ignore());
    let code = s.to_model();
    assert!(
        !code.contains("internal_cache"),
        "champ ignoré ne doit pas apparaître"
    );
}

#[test]
fn test_schema_to_model_contient_active_model_behavior() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(code.contains("ActiveModelBehavior"));
}

// ═══════════════════════════════════════════════════════════════
// Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_clone() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("name").string());
    let cloned = s.clone();
    assert_eq!(cloned.model_name, "User");
    assert_eq!(cloned.table_name, "user");
    assert_eq!(cloned.columns.len(), 1);
    assert!(cloned.primary_key.is_some());
}

// ═══════════════════════════════════════════════════════════════
// to_model() — relations
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_belongs_to() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::belongs_to("User", "user_id", "id"));
    let code = s.to_model();
    assert!(
        code.contains("belongs_to"),
        "BelongsTo doit générer belongs_to"
    );
    assert!(code.contains("user_id") || code.contains("UserId"));
}

#[test]
fn test_schema_to_model_has_many() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::has_many("post"));
    let code = s.to_model();
    assert!(code.contains("has_many"), "HasMany doit générer has_many");
}

#[test]
fn test_schema_to_model_has_one() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::has_one("profile"));
    let code = s.to_model();
    assert!(code.contains("has_many") || code.contains("has_one") || code.contains("Profile"));
}

#[test]
fn test_schema_to_model_many_to_many() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::many_to_many("tag", "post_tag"));
    let code = s.to_model();
    assert!(
        code.contains("many_to_many") || code.contains("via"),
        "ManyToMany doit générer via"
    );
}

// `to_model()` produit une String — les tests ci-dessus vérifient que les bons
// mots-clés/types apparaissent, mais aucun ne vérifie que le résultat est
// syntaxiquement du Rust valide (un `format!` mal placé — virgule oubliée,
// guillemet en trop — passerait `.contains(...)` sans problème). Les branches
// relation (le plus de `format!` imbriqués) sont les plus fragiles ; ce test
// exerce chaque variante de colonne ET chaque type de relation dans un seul
// schéma et vérifie que `syn::parse_file` accepte le résultat.
#[test]
fn test_schema_to_model_generates_valid_rust_syntax() {
    let s = ModelSchema::new("Comprehensive")
        .primary_key(PrimaryKeyDef::new("id").i64())
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("bio").text().nullable())
        .column(ColumnDef::new("views").integer())
        .column(ColumnDef::new("score").float())
        .column(ColumnDef::new("lat").double())
        .column(ColumnDef::new("active").boolean())
        .column(ColumnDef::new("event_date").date())
        .column(ColumnDef::new("uuid_val").uuid())
        .column(ColumnDef::new("data").json())
        .column(ColumnDef::new("internal").string().ignore())
        .relation(RelationDef::belongs_to("User", "user_id", "id"))
        .relation(RelationDef::has_many("comment"))
        .relation(RelationDef::has_one("profile"))
        .relation(RelationDef::many_to_many("tag", "comprehensive_tag"));

    let code = s.to_model();
    if let Err(e) = syn::parse_file(&code) {
        panic!("to_model() a produit du Rust syntaxiquement invalide : {e}\n---\n{code}");
    }
}

// ═══════════════════════════════════════════════════════════════
// to_model() — col_to_rust_type() variants
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_float_col() {
    let s = ModelSchema::new("Metrics")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("score").float());
    let code = s.to_model();
    assert!(code.contains("f32"), "float doit générer f32");
}

#[test]
fn test_schema_to_model_double_col() {
    let s = ModelSchema::new("Metrics")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("lat").double());
    let code = s.to_model();
    assert!(code.contains("f64"), "double doit générer f64");
}

#[test]
fn test_schema_to_model_boolean_col() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("active").boolean());
    let code = s.to_model();
    assert!(code.contains("bool"), "boolean doit générer bool");
}

#[test]
fn test_schema_to_model_date_col() {
    let s = ModelSchema::new("Event")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("event_date").date());
    let code = s.to_model();
    assert!(code.contains("NaiveDate"), "date doit générer NaiveDate");
}

#[test]
fn test_schema_to_model_uuid_col() {
    let s = ModelSchema::new("Token")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("uuid_val").uuid());
    let code = s.to_model();
    assert!(code.contains("Uuid"), "uuid doit générer Uuid");
}

#[test]
fn test_schema_to_model_json_col() {
    let s = ModelSchema::new("Config")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("data").json());
    let code = s.to_model();
    assert!(
        code.contains("serde_json::Value"),
        "json doit générer serde_json::Value"
    );
}

#[test]
fn test_schema_to_model_pk_uuid() {
    let s = ModelSchema::new("Token").primary_key(PrimaryKeyDef::new("id").uuid());
    let code = s.to_model();
    assert!(code.contains("Uuid"), "PK uuid doit générer Uuid");
}

// ═══════════════════════════════════════════════════════════════
// auto_now_columns / auto_now_update_columns / has_auto_timestamps
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_auto_now_columns() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now())
        .column(ColumnDef::new("updated_at").datetime().auto_now_update())
        .column(ColumnDef::new("title").string());
    let auto = s.auto_now_columns();
    assert_eq!(auto.len(), 1);
    assert_eq!(auto[0].name, "created_at");
}

#[test]
fn test_schema_auto_now_update_columns() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now())
        .column(ColumnDef::new("updated_at").datetime().auto_now_update());
    let auto_update = s.auto_now_update_columns();
    assert_eq!(auto_update.len(), 1);
    assert_eq!(auto_update[0].name, "updated_at");
}

#[test]
fn test_schema_has_auto_timestamps_true() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now());
    assert!(s.has_auto_timestamps());
}

#[test]
fn test_schema_has_auto_timestamps_false() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string());
    assert!(!s.has_auto_timestamps());
}

// ═══════════════════════════════════════════════════════════════
// fill_form()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_fill_form_all_fields() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, None);
    // 2 colonnes ajoutées (PK exclue automatiquement)
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_exclude() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("password").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, Some(&["password"]));
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_whitelist() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("bio").text());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, Some(&["username", "email"]), None);
    assert_eq!(form.fields.len() - before, 2);
}

// `test_schema_to_migration_ignored_col_skipped` remplacé par
// `test_schema_to_migration_ignored_column_is_really_absent` plus haut, qui
// vérifie l'absence réelle en DB plutôt que juste "ne panique pas".
