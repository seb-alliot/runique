//! Tests — contrat d'échappement sur les vues qui affichent des lignes DB.
//!
//! Quatrième test du chantier "détection des bugs muets" (cf. mémoire projet
//! `project_tests_detection_bugs_muets`). `tests/context/test_autoescape.rs`
//! couvre déjà les filtres Tera en isolation ; ce fichier étend le contrat aux
//! vraies vues admin (liste/detail) qui affichent des données venues de la DB :
//! aucune colonne sensible ne doit sortir en clair, et aucune valeur utilisateur
//! ne doit casser l'échappement HTML.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, admin_server_addr, login_as_superuser, seed_superuser_id_str,
};
use serial_test::serial;

// `#[serial]` : meme raison que `test_admin_route_crawl.rs` — le build unique
// derriere `admin_server_addr()` (OnceLock) partage `PENDING_URLS` avec les
// autres suites `#[serial]`. Voir `register_url.rs`.

/// Signature d'un hash Argon2 (`$argon2id$...`) — si elle apparaît dans une page
/// rendue, un secret est sorti en clair au lieu d'être masqué par le template.
const ARGON2_SIGNATURE: &str = "$argon2";

/// Le mot de passe (hashé) d'un utilisateur ne doit jamais apparaître en clair
/// dans la liste ni le détail admin — `admin/detail.html` est censé le
/// remplacer par `••••••••` pour toute clé `password`/`password_hash`.
#[tokio::test]
#[serial]
async fn test_admin_password_never_leaks_in_list_or_detail() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    let list_resp = client
        .get(format!("{base}{ADMIN_PREFIX}/users/list"))
        .send()
        .await
        .expect("GET users/list");
    assert_eq!(list_resp.status(), 200);
    let list_body = list_resp.text().await.unwrap_or_default();
    assert!(
        !list_body.contains(ARGON2_SIGNATURE),
        "hash de mot de passe visible dans la liste users : {list_body}"
    );

    let detail_resp = client
        .get(format!(
            "{base}{ADMIN_PREFIX}/users/{}/detail",
            seed_superuser_id_str()
        ))
        .send()
        .await
        .expect("GET users/{id}/detail");
    assert_eq!(detail_resp.status(), 200);
    let detail_body = detail_resp.text().await.unwrap_or_default();
    assert!(
        !detail_body.contains(ARGON2_SIGNATURE),
        "hash de mot de passe visible dans le détail users : {detail_body}"
    );
}

/// Une valeur utilisateur contenant des caractères HTML spéciaux ne doit
/// jamais ressortir non échappée dans la liste — sinon c'est une injection
/// HTML stockée.
///
/// Le payload n'est PAS une balise complète (`<script>...`) : `TextField::set_value`
/// (`forms/fields/text.rs`) passe par `sanitize_strict` (ammonia, liste de
/// balises vide) qui supprime toute balise reconnue — `<script>` disparaîtrait
/// intégralement (avec son contenu) *avant* même d'atteindre ce test, ce qui
/// masquerait la défense qu'on veut vérifier ici : l'échappement en sortie
/// (Tera autoescape). Le payload utilise donc des `<`/`>`/`&` qui ne forment
/// aucune balise valide — ils survivent à la sanitization d'entrée intacts et
/// doivent ressortir échappés au rendu.
#[tokio::test]
#[serial]
async fn test_admin_special_chars_are_escaped_in_list() {
    let addr = admin_server_addr();
    let base = format!("http://{addr}");
    let client = login_as_superuser(&base).await;

    let payload = "5 < 10 & 10 > 5";

    // CSRF réel : GET le formulaire de création pour obtenir le token.
    let create_get = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/create"))
        .send()
        .await
        .expect("GET groupes/create");
    assert_eq!(create_get.status(), 200);
    let token = create_get
        .headers()
        .get("x-csrf-token")
        .expect("x-csrf-token absent")
        .to_str()
        .expect("x-csrf-token non-UTF8")
        .to_string();

    let create_post = client
        .post(format!("{base}{ADMIN_PREFIX}/groupes/create"))
        .form(&[("nom", payload), ("csrf_token", token.as_str())])
        .send()
        .await
        .expect("POST groupes/create");
    assert!(
        create_post.status().is_redirection(),
        "création du groupe devrait rediriger, reçu {} — body: {}",
        create_post.status(),
        create_post.text().await.unwrap_or_default()
    );

    let list_resp = client
        .get(format!("{base}{ADMIN_PREFIX}/groupes/list"))
        .send()
        .await
        .expect("GET groupes/list");
    assert_eq!(list_resp.status(), 200);
    let list_body = list_resp.text().await.unwrap_or_default();
    assert!(
        !list_body.contains(payload),
        "caractères spéciaux non échappés dans la liste groupes : {list_body}"
    );
    assert!(
        list_body.contains("5 &lt; 10 &amp; 10 &gt; 5"),
        "le payload devrait apparaître échappé dans la liste groupes : {list_body}"
    );
}
