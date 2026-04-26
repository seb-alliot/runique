# Tester une application Runique

Runique fournit un ensemble de helpers spécialisés pour écrire des tests d'intégration expressifs pour votre application Axum.

---

## 1. Configuration des tests

Pour éviter la duplication de code, Runique encourage l'utilisation d'un builder de serveur partagé qui lance une instance réelle de votre application sur un port libre aléatoire.

```rust
use mon_projet::helpers::server::test_server_addr;

#[tokio::test]
async fn test_homepage_is_ok() {
    // 1. Récupère l'adresse du serveur partagé (le lance au premier appel)
    let addr = test_server_addr();
    
    // 2. Utilise reqwest (ou n'importe quel client) pour l'appeler
    let client = reqwest::Client::new();
    let resp = client.get(format!("http://{}/", addr)).await.unwrap();
    
    assert_eq!(resp.status(), 200);
}
```

---

## 2. Utilisation des requêtes Oneshot

Pour des tests plus rapides ne nécessitant pas de serveur TCP réel, vous pouvez utiliser les builders `oneshot`. Ceux-ci appellent le routeur Axum directement en mémoire.

```rust
use crate::helpers::{request, server::build_engine, server::build_default_router};

#[tokio::test]
async fn test_ping() {
    // Construction du moteur et du routeur
    let engine = build_engine().await;
    let app = build_default_router(engine);
    
    // GET standard
    let resp = request::get(app.clone(), "/").await;
    assert_eq!(resp.status(), 200);
    
    // POST avec header CSRF
    let resp = request::post_with_header(app, "/submit", "x-csrf-token", "mon-token").await;
    assert_eq!(resp.status(), 200);
}
```

---

## 3. Assertions spécialisées

Le module helper `assert` fournit des macros lisibles pour inspecter les réponses HTTP :

| Helper | Usage |
| --- | --- |
| `assert_status(&resp, code)` | Vérifie le code de statut HTTP |
| `assert_is_redirect(&resp)` | Vérifie s'il s'agit d'une redirection (3xx) |
| `assert_redirect(&resp, "/dest")` | Vérifie la redirection ET la destination exacte |
| `assert_has_header(&resp, "key")` | Vérifie si un header est présent |
| `assert_body_str(resp, "text").await` | Vérifie si le body contient exactement "text" |

```rust
use crate::helpers::assert::{assert_status, assert_redirect, assert_body_str};

#[tokio::test]
async fn test_login_flow() {
    let app = my_app_router();
    let resp = request::get(app, "/protected").await;
    
    // Vérifie la redirection vers le login
    assert_redirect(&resp, "/login");
}
```

---

## 4. Tests de base de données (SQLite isolé)

Runique fournit `fresh_db()` qui retourne une nouvelle base de données SQLite en mémoire, totalement isolée pour chaque test.

```rust
use crate::helpers::db;

#[tokio::test]
async fn test_database_persistence() {
    // Récupère une DB fraîche et isolée
    let db = db::fresh_db().await;
    
    // Applique le schéma et lance des requêtes
    db::exec(&db, "CREATE TABLE demo (id INTEGER PRIMARY KEY, val TEXT)").await;
    db::exec(&db, "INSERT INTO demo (val) VALUES ('runique')").await;
    
    // Comptage facilité
    db::assert_count(&db, "demo", 1).await;
}
```

---

## 5. Health Checks au démarrage (build)

Lorsque vous construisez votre application via `RuniqueApp::builder(config).build().await`, Runique effectue une suite de vérifications de santé :

- **Connectivité DB** : la base est-elle joignable ?
- **Templates** : tous les fichiers `.html` sont-ils syntaxiquement valides (Tera) ?
- **Sécurité** : la `SECRET_KEY` est-elle sûre (pas celle par défaut en prod) ?
- **Intégrité middleware** : les dépendances entre middlewares sont-elles satisfaites ?

Si une vérification échoue, la méthode `build()` retourne une erreur `BuildError::CheckFailed(CheckReport)` qui affiche un diagnostic clair dans le terminal avec des suggestions de correction.

---

← [**Exemples**](/docs/fr/exemple) | [**Dépannage**](/docs/fr/installation/troubleshooting) →
