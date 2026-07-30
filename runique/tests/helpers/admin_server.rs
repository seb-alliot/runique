//! Serveur admin complet partagé pour les tests d'intégration admin.
//!
//! Monte un vrai `RuniqueAppBuilder` (comme `demo-app`) avec les 3 ressources
//! builtin (`users`, `droits`, `groupes`) sur une base SQLite en mémoire, et
//! expose un client authentifié (flux CSRF réel). Utilisé par
//! `tests/admin/test_admin_route_crawl.rs` et
//! `tests/admin/test_admin_escaping_contract.rs` — factorisé ici pour éviter
//! que les deux dérivent (schéma, registre, login) au fil des évolutions.

use std::{net::SocketAddr, sync::Arc, sync::OnceLock};

use axum::{Router, routing::get};
use runique::admin::AdminRoutes;
use runique::prelude::*;

use crate::helpers::db;

// ── Constantes ───────────────────────────────────────────────────────────────

pub const TEST_SECRET: &str = "runique_admin_crawl_test_secret_key_1234567890_ok";
pub const ADMIN_PREFIX: &str = "/admin";
pub const SUPERUSER_USERNAME: &str = "crawler";
pub const SUPERUSER_PASSWORD: &str = "crawler_password_123";

// Identifiants connus des lignes seedées ci-dessous — utilisés par les tests
// de detail/edit/delete pour construire des URLs valides.
pub const SEED_GROUPE_ID: i64 = 1;
// Doit être une clé de ressource RÉELLEMENT enregistrée dans le registre de
// test (users/droits/groupes) : `prune_orphan_droits` (RuniqueAppBuilder::build)
// supprime au boot tout droit dont `resource_key` ne correspond à aucune
// ressource enregistrée (fermeture de faille — cf. mémoire projet 2.1.21).
pub const SEED_DROIT_RESOURCE_KEY: &str = "groupes";
pub const SEED_HISTORY_BATCH_ID: &str = "seed-batch-1";

// ── Schéma SQLite minimal (users + sessions + permissions + historique) ───────

const USERS_DDL: &str = "
    CREATE TABLE eihwaz_users (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        username    TEXT NOT NULL UNIQUE,
        email       TEXT NOT NULL UNIQUE,
        password    TEXT NOT NULL,
        is_active   INTEGER NOT NULL DEFAULT 1,
        is_staff    INTEGER NOT NULL DEFAULT 0,
        is_superuser INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT,
        updated_at  TEXT
    )
";

const SESSIONS_DDL: &str = "
    CREATE TABLE eihwaz_sessions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        cookie_id   TEXT NOT NULL UNIQUE,
        user_id     INTEGER NOT NULL,
        session_id  TEXT NOT NULL,
        session_data TEXT,
        expires_at  TEXT NOT NULL
    )
";

const GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_groupes (
        id  INTEGER PRIMARY KEY AUTOINCREMENT,
        nom TEXT NOT NULL
    )
";

const GROUPES_DROITS_DDL: &str = "
    CREATE TABLE eihwaz_groupes_droits (
        groupe_id      INTEGER NOT NULL,
        resource_key   TEXT NOT NULL,
        can_create     INTEGER NOT NULL DEFAULT 0,
        can_read       INTEGER NOT NULL DEFAULT 0,
        can_update     INTEGER NOT NULL DEFAULT 0,
        can_delete     INTEGER NOT NULL DEFAULT 0,
        can_update_own INTEGER NOT NULL DEFAULT 0,
        can_delete_own INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (groupe_id, resource_key)
    )
";

const USERS_GROUPES_DDL: &str = "
    CREATE TABLE eihwaz_users_groupes (
        user_id   INTEGER NOT NULL,
        groupe_id INTEGER NOT NULL,
        PRIMARY KEY (user_id, groupe_id)
    )
";

const HISTORY_DDL: &str = "
    CREATE TABLE eihwaz_history (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        resource_key  TEXT NOT NULL,
        object_pk     TEXT NOT NULL,
        action        TEXT NOT NULL,
        user_id       INTEGER NOT NULL,
        username      TEXT NOT NULL,
        created_at    TEXT NOT NULL,
        summary       TEXT,
        batch_id      TEXT
    )
";

// ── Registre + routes admin (équivalent minimal du code généré par le daemon,
//    réduit aux ressources builtin — cf. `demo-app/src/admins/admin.rs::routes`
//    et `::admin_register`, qui sont génériques et ne dépendent d'aucune
//    entité déclarée par le projet) ─────────────────────────────────────────

pub fn build_registry() -> AdminRegistry {
    let mut registry = AdminRegistry::new();
    for entry in builtin_resources() {
        registry.register(entry);
    }
    registry
}

fn build_admin_routes(prefix: &str) -> AdminRoutes {
    let p = prefix.trim_end_matches('/');
    let router = Router::new()
        .route(
            &format!("{p}/{{resource}}/{{action}}"),
            get(admin_get).post(admin_post),
        )
        .route(
            &format!("{p}/{{resource}}/{{id}}/{{action}}"),
            get(admin_get_id).post(admin_post_id),
        )
        .route(
            &format!("{p}/{{parent}}/{{parent_id}}/{{resource}}/{{action}}"),
            get(admin_nested_get).post(admin_nested_post),
        )
        .route(
            &format!("{p}/{{parent}}/{{parent_id}}/{{resource}}/{{id}}/{{action}}"),
            get(admin_nested_get_id).post(admin_nested_post_id),
        );
    AdminRoutes::new(p, router)
}

// ── Construction du serveur admin complet ──────────────────────────────────────

async fn seed_superuser(dbc: &DatabaseConnection) {
    let hash = hash(SUPERUSER_PASSWORD).expect("hash mot de passe superuser");
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_users (username, email, password, is_active, is_staff, is_superuser) \
             VALUES ('{SUPERUSER_USERNAME}', 'crawler@example.com', '{hash}', 1, 1, 1)"
        ),
    )
    .await;
}

/// Une ligne de chaque table annexe (groupe, droit, historique) pour que les
/// routes detail/edit/delete/historique aient un objet réel à rendre — pas
/// seulement des listes vides.
async fn seed_fixtures(dbc: &DatabaseConnection) {
    db::exec(
        dbc,
        &format!("INSERT INTO eihwaz_groupes (id, nom) VALUES ({SEED_GROUPE_ID}, 'Éditeurs')"),
    )
    .await;
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_groupes_droits \
             (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) \
             VALUES ({SEED_GROUPE_ID}, '{SEED_DROIT_RESOURCE_KEY}', 0, 1, 0, 0, 0, 0)"
        ),
    )
    .await;
    // Deux entrées du même batch (comme une bulk action) pour couvrir la vue batch,
    // plus une entrée seule pour la vue diff.
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_history (resource_key, object_pk, action, user_id, username, created_at, summary, batch_id) \
             VALUES ('groupes', '{SEED_GROUPE_ID}', 'update', 1, '{SUPERUSER_USERNAME}', '2026-07-30T00:00:00', \
             '{{\"nom\":{{\"old\":\"Ancien\",\"new\":\"Éditeurs\"}}}}', '{SEED_HISTORY_BATCH_ID}')"
        ),
    )
    .await;
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_history (resource_key, object_pk, action, user_id, username, created_at, summary, batch_id) \
             VALUES ('groupes', '{SEED_GROUPE_ID}', 'update', 1, '{SUPERUSER_USERNAME}', '2026-07-30T00:01:00', NULL, '{SEED_HISTORY_BATCH_ID}')"
        ),
    )
    .await;
    db::exec(
        dbc,
        &format!(
            "INSERT INTO eihwaz_history (resource_key, object_pk, action, user_id, username, created_at, summary, batch_id) \
             VALUES ('users', '1', 'create', 1, '{SUPERUSER_USERNAME}', '2026-07-30T00:02:00', NULL, NULL)"
        ),
    )
    .await;
}

pub fn droit_id() -> String {
    format!("{SEED_GROUPE_ID}:{SEED_DROIT_RESOURCE_KEY}")
}

async fn build_admin_app() -> Router {
    let dbc = db::fresh_db().await;
    db::exec(&dbc, USERS_DDL).await;
    db::exec(&dbc, SESSIONS_DDL).await;
    db::exec(&dbc, GROUPES_DDL).await;
    db::exec(&dbc, GROUPES_DROITS_DDL).await;
    db::exec(&dbc, USERS_GROUPES_DDL).await;
    db::exec(&dbc, HISTORY_DDL).await;
    seed_superuser(&dbc).await;
    seed_fixtures(&dbc).await;

    let mut config = RuniqueConfig {
        debug: true,
        ..Default::default()
    };
    config.server.secret_key = TEST_SECRET.to_string();

    let state = Arc::new(PrototypeAdminState {
        registry: Arc::new(build_registry()),
        config: Arc::new(AdminConfig::new()),
    });

    let app = RuniqueAppBuilder::new(config)
        .with_database(dbc)
        .no_statics()
        .with_admin(|a| {
            a.site_title("Test Admin")
                .auth(RuniqueAdminAuth::new())
                .routes(build_admin_routes(ADMIN_PREFIX))
                .with_state(state)
        })
        .build()
        .await
        .expect("construction de l'app admin de test");

    app.router
}

// ── Serveur partagé (une seule instance pour tous les tests qui l'utilisent) ──

static SERVER_ADDR: OnceLock<SocketAddr> = OnceLock::new();

pub fn admin_server_addr() -> SocketAddr {
    *SERVER_ADDR.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("test runtime");
            rt.block_on(async {
                let app = build_admin_app().await;
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind admin test server");
                let addr = listener.local_addr().expect("local addr");
                tx.send(addr).expect("send addr");
                axum::serve(listener, app).await.expect("serve admin app");
            });
        });
        rx.recv().expect("recv addr")
    })
}

pub fn admin_test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("reqwest client")
}

/// Se connecte en tant que superuser (flux CSRF réel : GET le formulaire pour
/// obtenir le token, puis POST le login avec ce même token dans le corps).
/// Retourne le client authentifié (cookie de session conservé).
pub async fn login_as_superuser(base: &str) -> reqwest::Client {
    let client = admin_test_client();

    let login_url = format!("{base}{ADMIN_PREFIX}/login");
    let get_resp = client
        .get(&login_url)
        .send()
        .await
        .expect("GET admin/login");
    assert_eq!(get_resp.status(), 200, "GET admin/login devrait rendre 200");
    let token = get_resp
        .headers()
        .get("x-csrf-token")
        .expect("header x-csrf-token absent sur GET admin/login")
        .to_str()
        .expect("x-csrf-token non-UTF8")
        .to_string();

    let post_resp = client
        .post(&login_url)
        .form(&[
            ("username", SUPERUSER_USERNAME),
            ("password", SUPERUSER_PASSWORD),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST admin/login");
    assert!(
        post_resp.status().is_redirection(),
        "POST admin/login devrait rediriger vers le dashboard, reçu {} — body: {}",
        post_resp.status(),
        post_resp.text().await.unwrap_or_default()
    );

    client
}

/// GET `url` avec `client` et vérifie un statut 200 (corps affiché en cas d'échec).
pub async fn assert_get_ok(client: &reqwest::Client, url: &str) {
    let resp = client
        .get(url)
        .send()
        .await
        .unwrap_or_else(|e| panic!("GET {url} a échoué : {e}"));
    let status = resp.status();
    assert_eq!(
        status,
        200,
        "GET {url} — body: {}",
        resp.text().await.unwrap_or_default()
    );
}
