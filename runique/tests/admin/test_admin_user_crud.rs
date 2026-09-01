//! Tests — CRUD complet + bulk delete sur la ressource builtin `users`.
//!
//! `test_admin_route_crawl.rs` ne fait que des GET (aucune 500), sans jamais
//! créer/modifier/supprimer un user pour de vrai — `test_admin_password_security.rs`
//! ne couvre que le flux reset-password (création incluse, mais pas edit/delete/bulk).
//! `handle_bulk.rs` (bulk_action=delete/group_set/update-submit) n'avait aucun test
//! dédié. Ce fichier couvre create/edit/delete individuel + bulk delete via de
//! vraies requêtes HTTP, vérifiées sur le HTML rendu — même pattern que
//! `test_admin_groupe_droits_crud.rs`.
//!
//! Reste non couvert : bulk `update-submit`/`group_set` (pas de `group_action`
//! configuré pour `users` dans ce harness de test) — à faire dans un futur passage.

use crate::helpers::admin_server::{ADMIN_PREFIX, admin_server_addr, login_as_superuser};
use regex::Regex;
use serial_test::serial;

// `#[serial]` : même raison que les autres suites admin partageant
// `admin_server_addr()` (registre global `PENDING_URLS`). Voir `register_url.rs`.

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

/// Trouve l'id d'une ligne de liste via le badge `#<id>` qui précède
/// immédiatement le texte donné dans le HTML rendu (cf.
/// `test_admin_groupe_droits_crud.rs::find_id_by_visible_text`).
fn find_id_by_visible_text(list_body: &str, needle: &str) -> String {
    let re = Regex::new(r"#(\d+)").expect("regex id valide");
    let needle_pos = list_body
        .find(needle)
        .unwrap_or_else(|| panic!("texte '{needle}' introuvable dans la liste : {list_body}"));
    let mut last_id = None;
    for cap in re.captures_iter(&list_body[..needle_pos]) {
        last_id = Some(cap[1].to_string());
    }
    last_id.unwrap_or_else(|| panic!("aucun badge #id avant '{needle}' dans la liste"))
}

#[tokio::test]
#[serial]
async fn test_user_create_edit_delete_roundtrip() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    // ─── 1. Créer un user ────────────────────────────────────────────────────
    let username = "crud_test_user";
    let email = "crud_test_user@example.com";
    let create_url = format!("{base}{ADMIN_PREFIX}/users/create");
    let token = csrf_token(&client, &create_url).await;
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

    let list_body = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        list_body.contains(username),
        "le user créé devrait apparaître dans la liste"
    );
    let user_id = find_id_by_visible_text(&list_body, username);

    // ─── 2. Modifier le user (username + email changés) ─────────────────────
    let edit_url = format!("{base}{ADMIN_PREFIX}/users/{user_id}/edit");
    // Nom ET email sans relation de sous-chaîne avec les valeurs d'origine
    // (sinon la vérif "l'ancien nom a disparu" est toujours vraie par
    // construction — l'email affiché dans la liste contiendrait encore
    // l'ancien username si on ne le changeait pas aussi. Même piège documenté
    // dans test_admin_groupe_droits_crud.rs, version email en plus).
    let renamed = "renommage_distinct_xyz";
    let renamed_email = "renommage_distinct_xyz@example.com";
    let token = csrf_token(&client, &edit_url).await;
    let resp = client
        .post(&edit_url)
        .form(&[
            ("username", renamed),
            ("email", renamed_email),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST users edit");
    assert!(
        resp.status().is_redirection(),
        "modification user devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let list_body_renamed = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list après renommage")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        list_body_renamed.contains(renamed),
        "le nouveau username devrait apparaître dans la liste"
    );
    assert!(
        !list_body_renamed.contains(username),
        "l'ancien username ne devrait plus apparaître dans la liste"
    );

    // ─── 3. Supprimer le user ─────────────────────────────────────────────────
    let delete_url = format!("{base}{ADMIN_PREFIX}/users/{user_id}/delete");
    let token = csrf_token(&client, &delete_url).await;
    let resp = client
        .post(&delete_url)
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST users delete");
    assert!(
        resp.status().is_redirection(),
        "suppression user devrait rediriger, reçu {}",
        resp.status()
    );

    let final_list = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list après suppression")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        !final_list.contains(renamed),
        "le user supprimé ne devrait plus apparaître dans la liste"
    );
}

// ── handle_bulk.rs — bulk_action=delete (aucun test dédié avant celui-ci) ──────

#[tokio::test]
#[serial]
async fn test_user_bulk_delete_roundtrip() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    // Crée deux users réels à supprimer en bloc.
    let mut ids = Vec::new();
    for username in ["bulk_del_user_a", "bulk_del_user_b"] {
        let create_url = format!("{base}{ADMIN_PREFIX}/users/create");
        let token = csrf_token(&client, &create_url).await;
        let resp = client
            .post(&create_url)
            .form(&[
                ("username", username),
                ("email", &format!("{username}@example.com")),
                ("csrf_token", token.as_str()),
            ])
            .send()
            .await
            .expect("POST users/create");
        assert!(
            resp.status().is_redirection(),
            "création user devrait rediriger, reçu {}",
            resp.status()
        );

        let list_body = client
            .get(format!("{base}{ADMIN_PREFIX}/users/list"))
            .send()
            .await
            .expect("GET users/list")
            .text()
            .await
            .unwrap_or_default();
        ids.push(find_id_by_visible_text(&list_body, username));
    }

    let list_before = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list avant bulk delete")
        .text()
        .await
        .unwrap_or_default();
    assert!(list_before.contains("bulk_del_user_a"));
    assert!(list_before.contains("bulk_del_user_b"));

    // ─── bulk_action=delete sur les deux ids d'un coup ───────────────────────
    let bulk_url = format!("{base}{ADMIN_PREFIX}/users/bulk");
    let token = csrf_token(&client, &format!("{base}{ADMIN_PREFIX}/users/list")).await;
    let resp = client
        .post(&bulk_url)
        .form(&[
            ("ids", ids.join(",").as_str()),
            ("bulk_action", "delete"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST users/bulk delete");
    assert!(
        resp.status().is_redirection(),
        "bulk delete devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let list_after = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list après bulk delete")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        !list_after.contains("bulk_del_user_a"),
        "le premier user supprimé en bloc ne devrait plus apparaître"
    );
    assert!(
        !list_after.contains("bulk_del_user_b"),
        "le second user supprimé en bloc ne devrait plus apparaître"
    );
}
