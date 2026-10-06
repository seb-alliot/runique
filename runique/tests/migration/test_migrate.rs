//! Tests — cli/migrate.rs : `up()` délègue à `sea-orm-cli migrate up`.

use crate::helpers::db_mariadb as db_maria;
use crate::helpers::db_postgres as db_pg;
use runique::cli::migrate::up;
use serial_test::serial;

// ═══════════════════════════════════════════════════════════════
// up() — teste les deux DB (Postgres + MariaDB)
// ═══════════════════════════════════════════════════════════════

// Ignoré sur Windows avec code page non-UTF-8 (bug SQLx sur Windows français)
// Validé sur le fixe (Ryzen 7 5800X). Relancer manuellement avec : cargo test test_up -- --ignored
#[tokio::test]
#[ignore]
#[serial]
async fn test_up_retourne_ok() {
    dotenvy::from_filename(".env.test").ok();
    let Some(pg_url) = crate::helpers::db_isolation::isolated_url("DATABASE_URL_PG").await else {
        return; // skip si pas de Docker
    };
    unsafe { std::env::set_var("DATABASE_URL", &pg_url) };
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../demo-app/migration");
    let result = up(migration_dir).await;
    assert!(result.is_ok(), "up() Postgres doit Ok: {:?}", result);

    // `Ok` seul ne prouve pas que le schéma existe réellement — `count()` panique
    // si la table n'existe pas (requête SQL qui échoue), donc son succès prouve
    // que la table framework ET une table métier demo-app ont bien été créées.
    let Some(db) = db_pg::connect().await else {
        return;
    };
    db_pg::count(&db, "eihwaz_users").await;
    db_pg::count(&db, "blog").await;
}

// Ignoré sur Windows avec code page non-UTF-8 (bug SQLx sur Windows français)
// Validé sur le fixe (Ryzen 7 5800X). Relancer manuellement avec : cargo test test_up -- --ignored
#[tokio::test]
#[ignore]
#[serial]
async fn test_up_mariadb_retourne_ok() {
    dotenvy::from_filename(".env.test").ok();
    let Some(maria_url) = crate::helpers::db_isolation::isolated_url("DATABASE_URL_MARIADB").await
    else {
        return; // skip si pas de Docker
    };
    unsafe { std::env::set_var("DATABASE_URL", &maria_url) };
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../demo-app/migration");
    let result = up(migration_dir).await;
    assert!(result.is_ok(), "up() MariaDB doit Ok: {:?}", result);

    // Idem Postgres : `count()` panique si la table n'existe pas, donc son succès
    // prouve que le schéma a réellement été créé, pas juste que `up()` a retourné Ok.
    let Some(db) = db_maria::connect().await else {
        return;
    };
    db_maria::count(&db, "eihwaz_users").await;
    db_maria::count(&db, "blog").await;
}

#[tokio::test]
async fn test_up_chemin_inexistant_retourne_err() {
    // up() avec chemin inexistant → sea-orm-cli échoue → Err attendu
    let result = up("/chemin/inexistant/abc").await;
    assert!(result.is_err(), "up() chemin inexistant doit Err");
}
