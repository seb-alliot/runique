//! Tests — sécurité du flux reset-password admin (`admin_main/handle_password.rs`).
//!
//! `handle_password.rs` était à 0 % de couverture alors qu'il touche directement
//! l'auth (génération de token de reset, lié à un `user_id`, IDOR-safe selon son
//! propre commentaire). Ces tests vérifient le point qui compte vraiment : le
//! token émis par une action admin de reset est bien lié à l'utilisateur CIBLE,
//! jamais à l'acteur (l'admin qui déclenche l'action), et qu'un id inconnu ne
//! provoque ni crash ni création de token fantôme.
//!
//! Chaque test monte SA PROPRE instance de serveur admin (via
//! `admin_server::build_admin_app()`) sur SON PROPRE runtime tokio, au lieu de
//! partager le serveur `admin_server_addr()`/`admin_server_db()` utilisé par les
//! autres suites admin. Raison : ces tests interrogent la DB directement
//! (`consume()`, lookup par email) — partager cette connexion à travers les
//! runtimes de plusieurs `#[tokio::test]` casse le pool SQLite `:memory:`
//! (taille 1) dès qu'un runtime se termine, ce qui casse silencieusement TOUS
//! les logins suivants sur le serveur partagé (bug découvert en déboguant ce
//! fichier — reproduit avec des tests diagnostiques, non gardés).

use crate::helpers::admin_server::{self, ADMIN_PREFIX};
use regex::Regex;
use runique::sea_orm::{
    ConnectionTrait, DatabaseConnection,
    sea_query::{Alias, Asterisk, Expr, ExprTrait, Func, Query},
};
use runique::utils::Pk;
use runique::utils::reset_token::consume;
use serial_test::serial;

// `#[serial]` : `build_admin_app()` enregistre les routes nommées dans le
// registre global `PENDING_URLS` (partagé avec toutes les autres suites admin
// `#[serial]`) — deux builds concurrents s'y marchent dessus. Voir
// `register_url.rs` et le commentaire identique dans `test_admin_route_crawl.rs`.

const SUPERUSER_USERNAME: &str = "crawler";
const SUPERUSER_PASSWORD: &str = "crawler_password_123";

// ── Helpers locaux ────────────────────────────────────────────────────────────

/// Monte un serveur admin complet SUR LE RUNTIME DE L'APPELANT (pas un thread
/// séparé) — la connexion DB retournée reste utilisable en toute sécurité
/// puisqu'elle ne traverse jamais de frontière de runtime.
async fn spawn_local_admin_server() -> (String, DatabaseConnection) {
    let (router, dbc) = admin_server::build_admin_app().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind serveur admin local");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve admin app local");
    });
    (format!("http://{addr}"), dbc)
}

fn local_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("reqwest client")
}

/// Récupère le token CSRF exposé en header sur un GET (même flux que
/// `login_as_superuser` / `test_admin_escaping_contract.rs`).
async fn csrf_token(client: &reqwest::Client, url: &str) -> String {
    let resp = client.get(url).send().await.expect("GET pour csrf token");
    assert_eq!(resp.status(), 200, "GET {url} devrait rendre 200");
    resp.headers()
        .get("x-csrf-token")
        .expect("header x-csrf-token absent")
        .to_str()
        .expect("x-csrf-token non-UTF8")
        .to_string()
}

/// Se connecte en tant que superuser (flux CSRF réel, identique à
/// `admin_server::login_as_superuser` mais sur le serveur local de ce fichier).
async fn login_as_superuser(base: &str) -> reqwest::Client {
    let client = local_client();
    let login_url = format!("{base}{ADMIN_PREFIX}/login");
    let token = csrf_token(&client, &login_url).await;

    let resp = client
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
        resp.status().is_redirection(),
        "POST admin/login devrait rediriger vers le dashboard, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
    client
}

/// Crée un user via le formulaire admin (`username`+`email` suffisent —
/// le mot de passe vide est auto-généré par `handle_create_post`).
async fn create_admin_user(client: &reqwest::Client, base: &str, username: &str, email: &str) {
    let create_url = format!("{base}{ADMIN_PREFIX}/users/create");
    let token = csrf_token(client, &create_url).await;

    let resp = client
        .post(&create_url)
        .form(&[
            ("username", username),
            ("email", email),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST users/create");
    assert!(
        resp.status().is_redirection(),
        "création user devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );
}

/// Résout l'id d'un user par email — même idiome que `helpers::db::count`
/// (query builder + `try_get`), généralisé à une colonne autre que `COUNT(*)`.
async fn find_user_id_by_email(db: &DatabaseConnection, email: &str) -> Pk {
    let stmt = Query::select()
        .column(Alias::new("id"))
        .from(Alias::new("eihwaz_users"))
        .and_where(Expr::col(Alias::new("email")).eq(email))
        .to_owned();
    let row = db
        .query_one(&stmt)
        .await
        .unwrap_or_else(|e| panic!("query user id by email échoué : {e}"))
        .unwrap_or_else(|| panic!("aucun user avec l'email {email}"));
    row.try_get::<Pk>("", "id")
        .unwrap_or_else(|e| panic!("lecture id échouée : {e}"))
}

/// Résout l'id du superuser seedé (toujours id `1` — cf. `admin_server::seed_superuser`).
async fn find_superuser_id(db: &DatabaseConnection) -> Pk {
    find_user_id_by_email(db, "crawler@example.com").await
}

async fn count_reset_tokens(db: &DatabaseConnection) -> i64 {
    let stmt = Query::select()
        .expr_as(Func::count(Expr::col(Asterisk)), Alias::new("n"))
        .from(Alias::new("eihwaz_reset_tokens"))
        .to_owned();
    let row = db
        .query_one(&stmt)
        .await
        .expect("count reset_tokens échoué")
        .expect("count reset_tokens : aucune ligne");
    row.try_get::<i64>("", "n")
        .expect("count reset_tokens : lecture échouée")
}

/// Extrait le token de reset depuis le corps HTML rendu (notice
/// `admin.reset_password.success_link`, cf. `trad/en.json`: "Reset link
/// (mailer not configured): {}") — le mailer n'est pas configuré dans ce
/// harness de test, donc l'URL de reset est toujours affichée en clair.
fn extract_reset_token(body: &str) -> String {
    let re = Regex::new(r#"/reset-password/([^/\s"<]+)/[^/\s"<]+"#).expect("regex valide");
    re.captures(body)
        .unwrap_or_else(|| panic!("aucune URL de reset trouvée dans le corps : {body}"))
        .get(1)
        .expect("groupe de capture token")
        .as_str()
        .to_string()
}

// ═══════════════════════════════════════════════════════════════
// handle_reset_password — IDOR : le token doit être lié à la CIBLE,
// jamais à l'acteur qui déclenche l'action.
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_reset_password_binds_token_to_target_not_actor() {
    let (base, dbc) = spawn_local_admin_server().await;
    let client = login_as_superuser(&base).await;
    let actor_id = find_superuser_id(&dbc).await;

    let target_email = "reset-target-idor@example.com";
    create_admin_user(&client, &base, "reset_target_idor", target_email).await;
    let target_id = find_user_id_by_email(&dbc, target_email).await;
    assert_ne!(
        target_id, actor_id,
        "précondition : la cible doit être différente de l'acteur"
    );

    let reset_url = format!("{base}{ADMIN_PREFIX}/users/{target_id}/reset-password");
    let token = csrf_token(
        &client,
        &format!("{base}{ADMIN_PREFIX}/users/{target_id}/detail"),
    )
    .await;

    let reset_resp = client
        .post(&reset_url)
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST reset-password");
    assert!(
        reset_resp.status().is_redirection(),
        "reset-password devrait rediriger, reçu {} — body: {}",
        reset_resp.status(),
        reset_resp.text().await.unwrap_or_default()
    );

    // La notice (lien de reset en clair, mailer non configuré) apparaît sur la
    // page de détail vers laquelle l'action redirige.
    let detail_resp = client
        .get(format!("{base}{ADMIN_PREFIX}/users/{target_id}/detail"))
        .send()
        .await
        .expect("GET detail après reset");
    assert_eq!(detail_resp.status(), 200);
    let body = detail_resp.text().await.unwrap_or_default();
    let raw_token = extract_reset_token(&body);

    let bound_user_id = consume(&dbc, &raw_token)
        .await
        .expect("le token émis doit résoudre un user_id valide");

    assert_eq!(
        bound_user_id, target_id,
        "le token doit être lié à la cible du reset ({target_id}), pas à un autre id"
    );
    assert_ne!(
        bound_user_id, actor_id,
        "IDOR: le token ne doit jamais se lier à l'acteur qui a déclenché l'action"
    );
}

// ═══════════════════════════════════════════════════════════════
// handle_reset_password — id inconnu : pas de crash, pas de token fantôme.
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_reset_password_unknown_id_creates_no_token() {
    let (base, dbc) = spawn_local_admin_server().await;
    let client = login_as_superuser(&base).await;

    let bogus_id = "999999999";
    let before = count_reset_tokens(&dbc).await;

    // Pas de flux CSRF dédié pour un id inexistant (pas de page détail à
    // charger) — on réutilise un token valide obtenu sur la page users/list.
    let token = csrf_token(&client, &format!("{base}{ADMIN_PREFIX}/users/list")).await;

    let resp = client
        .post(format!(
            "{base}{ADMIN_PREFIX}/users/{bogus_id}/reset-password"
        ))
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST reset-password id inconnu");

    assert!(
        !resp.status().is_server_error(),
        "un id inconnu ne doit jamais provoquer une 5xx, reçu {}",
        resp.status()
    );

    let after = count_reset_tokens(&dbc).await;
    assert_eq!(
        before, after,
        "aucun token ne doit être créé pour un id qui n'existe pas"
    );
}

// ═══════════════════════════════════════════════════════════════
// send_user_created_email (création admin) — même garantie IDOR : le token
// émis à la création est lié au nouvel user, pas au superuser créateur.
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_create_user_token_binds_to_new_user_not_creator() {
    let (base, dbc) = spawn_local_admin_server().await;
    let client = login_as_superuser(&base).await;
    let actor_id = find_superuser_id(&dbc).await;

    let new_email = "create-flow-idor@example.com";
    create_admin_user(&client, &base, "create_flow_idor", new_email).await;
    let new_user_id = find_user_id_by_email(&dbc, new_email).await;

    // Le create redirige vers la liste — la notice (lien de reset, mailer non
    // configuré) s'affiche là.
    let list_resp = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list après create");
    assert_eq!(list_resp.status(), 200);
    let body = list_resp.text().await.unwrap_or_default();
    let raw_token = extract_reset_token(&body);

    let bound_user_id = consume(&dbc, &raw_token)
        .await
        .expect("le token émis à la création doit résoudre un user_id valide");

    assert_eq!(
        bound_user_id, new_user_id,
        "le token émis à la création doit être lié au nouvel user, pas à un autre id"
    );
    assert_ne!(
        bound_user_id, actor_id,
        "IDOR: le token de création ne doit jamais se lier au superuser créateur"
    );
}
