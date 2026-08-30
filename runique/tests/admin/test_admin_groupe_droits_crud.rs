//! Tests — CRUD complet sur les groupes de permissions (`admin/permissions/groupe.rs`
//! + `groupes_droits.rs`, exposés via `builtin/groupe.rs` + `builtin/droit.rs`).
//!
//! Ces deux entités étaient à 0 % de couverture : le seul test existant qui les
//! touche (`test_admin_route_crawl.rs`) ne fait que des GET (aucune 500), sans
//! jamais créer/modifier/supprimer un groupe ou un droit. C'est aussi le seul
//! mécanisme de contrôle d'accès de tout le panel admin — un aller-retour CRUD
//! complet ici est une vraie validation fonctionnelle, pas juste un chiffre de
//! couverture.
//!
//! Vérifie entièrement via HTTP (GET/POST + assertions sur le HTML rendu) —
//! pas d'accès direct à la DB, donc pas de risque de partage de connexion
//! cross-runtime (cf. `test_admin_password_security.rs`).

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
/// `templates/admin/composant/list_partial.html`: `<span class="admin-badge--id">#{{ entry.id }}</span>`
/// suivi des colonnes visibles, dont le texte recherché fait partie).
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

/// Vérifie si la checkbox `name="{field}"` est rendue `checked` dans le HTML
/// (cf. `templates/field_html/base_boolean.html`).
fn checkbox_is_checked(body: &str, field: &str) -> bool {
    let marker = format!("name=\"{field}\"");
    let Some(start) = body.find(&marker) else {
        panic!("champ '{field}' introuvable dans le formulaire");
    };
    let end = body[start..]
        .find('>')
        .map(|i| start + i)
        .unwrap_or(body.len());
    body[start..end].contains("checked")
}

#[tokio::test]
#[serial]
async fn test_groupe_and_droit_full_crud_roundtrip() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    // ─── 1. Créer un nouveau groupe ─────────────────────────────────────────
    let groupe_nom = "crud_test_groupe";
    let create_groupe_url = format!("{base}{ADMIN_PREFIX}/groupes/create");
    let token = csrf_token(&client, &create_groupe_url).await;
    let resp = client
        .post(&create_groupe_url)
        .form(&[("nom", groupe_nom), ("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST groupes/create");
    assert!(
        resp.status().is_redirection(),
        "création groupe devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let list_body = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/list"))
        .send()
        .await
        .expect("GET groupes/list")
        .text()
        .await
        .unwrap_or_default();
    let groupe_id = find_id_by_visible_text(&list_body, groupe_nom);

    // ─── 2. Créer un droit pour ce groupe : can_create + can_read seulement ──
    let create_droit_url = format!("{base}{ADMIN_PREFIX}/droits/create");
    let token = csrf_token(&client, &create_droit_url).await;
    let resp = client
        .post(&create_droit_url)
        .form(&[
            ("groupe_id", groupe_id.as_str()),
            ("resource_key", "users"),
            ("can_create", "true"),
            ("can_read", "true"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST droits/create");
    assert!(
        resp.status().is_redirection(),
        "création droit devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let droit_id = format!("{groupe_id}:users");
    let edit_url = format!("{base}{ADMIN_PREFIX}/droits/{droit_id}/edit");
    let edit_body = client
        .get(&edit_url)
        .send()
        .await
        .expect("GET droit edit")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        checkbox_is_checked(&edit_body, "can_create"),
        "can_create devrait être coché juste après création"
    );
    assert!(
        checkbox_is_checked(&edit_body, "can_read"),
        "can_read devrait être coché juste après création"
    );
    assert!(
        !checkbox_is_checked(&edit_body, "can_update"),
        "can_update ne devrait PAS être coché (jamais envoyé)"
    );
    assert!(
        !checkbox_is_checked(&edit_body, "can_delete"),
        "can_delete ne devrait PAS être coché (jamais envoyé)"
    );

    // ─── 3. Modifier le droit : n'envoyer que can_update cette fois ─────────
    let token = csrf_token(&client, &edit_url).await;
    let resp = client
        .post(&edit_url)
        .form(&[
            ("groupe_id", groupe_id.as_str()),
            ("resource_key", "users"),
            ("can_update", "true"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST droit edit");
    assert!(
        resp.status().is_redirection(),
        "modification droit devrait rediriger, reçu {} — body: {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let edit_body_after = client
        .get(&edit_url)
        .send()
        .await
        .expect("GET droit edit après modif")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        checkbox_is_checked(&edit_body_after, "can_update"),
        "can_update devrait être coché après la modification"
    );
    assert!(
        !checkbox_is_checked(&edit_body_after, "can_create"),
        "can_create ne devrait plus être coché — l'update remplace, il ne fusionne pas"
    );

    // ─── 4. Supprimer le droit ───────────────────────────────────────────────
    let delete_url = format!("{base}{ADMIN_PREFIX}/droits/{droit_id}/delete");
    let token = csrf_token(&client, &delete_url).await;
    let resp = client
        .post(&delete_url)
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST droit delete");
    assert!(
        resp.status().is_redirection(),
        "suppression droit devrait rediriger, reçu {}",
        resp.status()
    );

    let list_body_after_delete = client
        .get(format!("{base}{ADMIN_PREFIX}/droits/list"))
        .send()
        .await
        .expect("GET droits/list après suppression")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        !list_body_after_delete.contains(&droit_id),
        "le droit supprimé ne devrait plus apparaître dans la liste"
    );

    // ─── 5. Renommer le groupe ────────────────────────────────────────────────
    let groupe_edit_url = format!("{base}{ADMIN_PREFIX}/groupes/{groupe_id}/edit");
    // Nom sans relation de sous-chaîne avec `groupe_nom` (sinon la vérif
    // "l'ancien nom n'apparaît plus" serait toujours vraie par construction).
    let renamed = "renommage_distinct_xyz";
    let token = csrf_token(&client, &groupe_edit_url).await;
    let resp = client
        .post(&groupe_edit_url)
        .form(&[("nom", renamed), ("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST groupe edit");
    assert!(
        resp.status().is_redirection(),
        "renommage groupe devrait rediriger, reçu {}",
        resp.status()
    );

    let list_body_renamed = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/list"))
        .send()
        .await
        .expect("GET groupes/list après renommage")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        list_body_renamed.contains(renamed),
        "le nouveau nom devrait apparaître dans la liste"
    );
    assert!(
        !list_body_renamed.contains(groupe_nom),
        "l'ancien nom ne devrait plus apparaître dans la liste"
    );

    // ─── 6. Supprimer le groupe ───────────────────────────────────────────────
    let groupe_delete_url = format!("{base}{ADMIN_PREFIX}/groupes/{groupe_id}/delete");
    let token = csrf_token(&client, &groupe_delete_url).await;
    let resp = client
        .post(&groupe_delete_url)
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST groupe delete");
    assert!(
        resp.status().is_redirection(),
        "suppression groupe devrait rediriger, reçu {}",
        resp.status()
    );

    let final_list = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/list"))
        .send()
        .await
        .expect("GET groupes/list après suppression groupe")
        .text()
        .await
        .unwrap_or_default();
    assert!(
        !final_list.contains(renamed),
        "le groupe supprimé ne devrait plus apparaître dans la liste"
    );
}
