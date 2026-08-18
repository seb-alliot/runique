//! Tests — parcours des routes admin ("route crawl").
//!
//! Construit un serveur admin complet (ressources builtin : `users`, `droits`,
//! `groupes` — cf. `helpers::admin_server`) et vérifie qu'aucune route ne rend
//! une 500. C'est le test le plus rentable identifié pour détecter les bugs
//! muets de rendu (cf. mémoire projet `project_tests_detection_bugs_muets` —
//! 9 des 10 bugs de la migration Tera 2 auraient été attrapés par ce type de
//! parcours).
//!
//! Couvre : login réel (flux CSRF inclus), dashboard, listes, detail/create/
//! edit/delete/bulk pour les 3 ressources builtin, les 4 vues d'historique,
//! et le cas login en erreur (mauvais mot de passe).

use crate::helpers::admin_server::{
    ADMIN_PREFIX, SEED_GROUPE_ID, SEED_HISTORY_BATCH_ID, SUPERUSER_USERNAME, admin_server_addr,
    admin_test_client, assert_get_ok, droit_id, login_as_superuser,
};
use serial_test::serial;

// `#[serial]` : `admin_server_addr()` ne construit le serveur qu'une fois (OnceLock),
// mais ce build unique passe par le meme registre global `PENDING_URLS` que
// `test_admin_prefix.rs`/`test_robots_txt.rs` (egalement `#[serial]`). Comme on ne
// sait pas lequel de ces tests declenchera le build en premier, ils doivent tous
// tenir le meme verrou pour garantir qu'aucun autre build ne s'execute en meme temps.
// Voir `register_url.rs`.

#[tokio::test]
#[serial]
async fn test_admin_crawl_dashboard_and_lists() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");

    let client = login_as_superuser(&base).await;

    let dashboard_resp = client
        .get(format!("{base}{ADMIN_PREFIX}/"))
        .send()
        .await
        .expect("GET dashboard");
    assert_eq!(
        dashboard_resp.status(),
        200,
        "dashboard: body = {}",
        dashboard_resp.text().await.unwrap_or_default()
    );

    for resource in ["users", "droits", "groupes"] {
        let url = format!("{base}{ADMIN_PREFIX}/{resource}/list");
        let resp = client.get(&url).send().await.expect("GET liste");
        let status = resp.status();
        assert_eq!(
            status,
            200,
            "liste '{resource}' ({url}) — body: {}",
            resp.text().await.unwrap_or_default()
        );
    }
}

#[tokio::test]
#[serial]
async fn test_admin_crawl_detail_create_edit_delete_bulk() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    // Chaque ressource a un id connu (seedé dans `helpers::admin_server`) :
    // users/groupes ont un id entier simple, droits un id composite
    // "{groupe_id}:{resource_key}".
    let ids: [(&str, String); 3] = [
        ("users", "1".to_string()),
        ("groupes", SEED_GROUPE_ID.to_string()),
        ("droits", droit_id()),
    ];

    for (resource, id) in &ids {
        assert_get_ok(&client, &format!("{base}{ADMIN_PREFIX}/{resource}/create")).await;
        assert_get_ok(
            &client,
            &format!("{base}{ADMIN_PREFIX}/{resource}/{id}/detail"),
        )
        .await;
        assert_get_ok(
            &client,
            &format!("{base}{ADMIN_PREFIX}/{resource}/{id}/edit"),
        )
        .await;
        assert_get_ok(
            &client,
            &format!("{base}{ADMIN_PREFIX}/{resource}/{id}/delete"),
        )
        .await;
        assert_get_ok(
            &client,
            &format!("{base}{ADMIN_PREFIX}/{resource}/bulk?ids={id}"),
        )
        .await;
    }
}

#[tokio::test]
#[serial]
async fn test_admin_crawl_history_views() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    assert_get_ok(&client, &format!("{base}{ADMIN_PREFIX}/history")).await;
    assert_get_ok(&client, &format!("{base}{ADMIN_PREFIX}/history/timeline")).await;
    assert_get_ok(
        &client,
        &format!("{base}{ADMIN_PREFIX}/history/batch/{SEED_HISTORY_BATCH_ID}"),
    )
    .await;
    // Première ligne insérée par `seed_fixtures` → id auto-incrémenté = 1.
    assert_get_ok(&client, &format!("{base}{ADMIN_PREFIX}/history/1")).await;
}

/// Un login raté (mauvais mot de passe) doit re-rendre `login.html` avec un
/// message d'erreur — pas planter sur une variable de contexte manquante.
/// C'est exactement le genre de branche que la migration Tera 2 a cassée en
/// silence (cf. mémoire projet `project_tests_detection_bugs_muets`).
#[tokio::test]
#[serial]
async fn test_admin_login_wrong_password_rerenders_form() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = admin_test_client();

    let login_url = format!("{base}{ADMIN_PREFIX}/login");
    let get_resp = client.get(&login_url).send().await.expect("GET login");
    let token = get_resp
        .headers()
        .get("x-csrf-token")
        .expect("x-csrf-token absent")
        .to_str()
        .expect("x-csrf-token non-UTF8")
        .to_string();

    let post_resp = client
        .post(&login_url)
        .form(&[
            ("username", SUPERUSER_USERNAME),
            ("password", "mot_de_passe_incorrect"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST login mauvais mot de passe");

    assert_eq!(
        post_resp.status(),
        200,
        "un login raté doit re-rendre le formulaire (200), pas rediriger ni planter"
    );
    let body = post_resp.text().await.unwrap_or_default();
    assert!(
        !body.is_empty(),
        "le corps de la page de login en erreur ne devrait pas être vide"
    );
}
