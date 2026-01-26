---

# 🦀 Runique Framework - Exemple Complet

Un exemple minimaliste mais complet d’application web Rust avec **Runique**, prêt à être exécuté immédiatement.

Ce projet inclut :

* Configuration `.env`
* Builder `RuniqueAppBuilder` avec toutes les méthodes configurables
* Routes, handlers et templates Tera
* Sessions, CSRF et sécurité (CSP, Allowed Hosts)
* ORM optionnel (SeaORM)
* Fichiers statiques et médias

---

## 1. Arborescence

```
my_runique_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── routes.rs
│   ├── handlers.rs
│   └── models.rs
├── templates/
│   ├── base.html
│   └── index.html
├── static/
│   └── style.css
```

---

## 2. Configuration `.env`

```env
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=dev-secret-key
DEBUG=true

ALLOWED_HOSTS=localhost,127.0.0.1

```

---

## 3. Cargo.toml (extrait)

```toml
[dependencies]
runique = { path = "../runique" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tower-sessions = "0.8"
tera = "1.21"
```

Si tu utilises l’ORM :

```toml
sea-orm = { version = "0.13", features = ["sqlx-postgres"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-native-tls"] }
```

---

## 4. Exemple `src/main.rs`

```rust
#[macro_use]
extern crate runique;

mod forms;
mod models;
mod prelude;
mod url;
mod views;

use prelude::*;
use runique::tower_sessions::MemoryStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger la configuration
    let config = RuniqueConfig::from_env();

    // Connexion à la base de données (optionnel)
    #[cfg(feature = "orm")]
    let db = {
        let db_config = DatabaseConfig::from_env()?.build();
        db_config.connect().await?
    };

    // Créer et lancer l'application
    let app = RuniqueApp::builder(config)
        .routes(url::routes())                      // Routes applicatives
        .with_database(db)                          // Connexion ORM (optionnel)
        .with_error_handler(true)                   // Middleware capture d'erreurs
        .with_allowed_hosts(true)                   // Protection Host Header
        .with_csp(true)                             // CSP & headers sécurité
        .with_cache(true)                           // No-cache en dev
        .with_sanitizer(true)                       // Nettoyage inputs
        .with_session_store(MemoryStore::default()) // Session store
        .with_session_duration(time::Duration::hours(2)) // Durée session
        .build()
        .await?;

    app.run().await?;

    Ok(())
}
```

---

## 5. Stack Middleware (ordre d'exécution)

Runique applique automatiquement le middleware **dans l’ordre optimal** :

1. `RequestExtensions` - Injection Tera, Config, Engine
2. Fichiers statiques (css/js/images)
3. Gestion des erreurs (si activé)
4. Sessions (`MemoryStore` ou autre store)
5. CSRF protection (automatique pour POST/PUT/PATCH/DELETE)
6. Cache (dev no-cache si activé)
7. Sanitize inputs (si activé)
8. Routes utilisateur

Toutes ces étapes sont configurables via `RuniqueAppBuilder`.

---

## 6. Méthodes configurables du builder

| Méthode                           | Description                                                              |
| --------------------------------- | ------------------------------------------------------------------------ |
| `routes(router)`                  | Définit les routes applicatives (obligatoire)                            |
| `with_database(db)`               | Fournit la connexion ORM (`DatabaseConnection`) si feature `orm` activée |
| `with_error_handler(bool)`        | Middleware pour capture et gestion des erreurs                           |
| `with_allowed_hosts(bool)`        | Protection contre Host Header Injection                                  |
| `with_csp(bool)`                  | Active CSP et headers de sécurité                                        |
| `with_sanitizer(bool)`            | Nettoyage automatique des inputs                                         |
| `with_cache(bool)`                | No-cache en développement                                                |
| `with_session_store(store)`       | Définit le store de session (ex : `MemoryStore`)                         |
| `with_session_duration(duration)` | Durée d’inactivité avant expiration de session                           |

---

## 7. Sessions

```rust
use tower_sessions::{Session, SessionManagerLayer, MemoryStore, Expiry};

async fn login(session: Session) {
    session.insert("user_id", 1).await.ok();
    session.insert("username", "john").await.ok();
}

async fn dashboard(session: Session) {
    let user_id: Option<i32> = session.get("user_id").await.ok().flatten();
}
```

**Notes :**

* CSRF est généré automatiquement pour chaque session
* Double Submit Cookie pattern
* Pas besoin d’ajouter `{% csrf %}` dans les formulaires, c’est fait automatiquement

---

## 8. CSP - Content Security Policy

* Nonce généré automatiquement et injecté dans `TemplateContext`
* Support `<script nonce="{{ csp_nonce }}">` et `<style nonce="{{ csp_nonce }}">`

```html
<script nonce="{{ csp_nonce }}">
    console.log("Code JS sécurisé avec nonce");
</script>
```

* Configuration via `CspConfig::strict()`, `CspConfig::permissive()`, `CspConfig::default()`

---

## 9. Authentification

* Middleware custom pour vérifier si l’utilisateur est connecté
* Extractor `CurrentUser` pour récupérer `user_id` et `username` depuis la session

```rust
async fn dashboard(user: CurrentUser) -> impl IntoResponse {
    format!("Bienvenue, {}!", user.username)
}
```

---

## 10. Lancer le projet

```bash
cargo run
```

Ouvrir ensuite :

```
http://127.0.0.1:3000
```

Tu obtiens :

* Page HTML rendue avec Tera
* Routes fonctionnelles
* Sessions actives
* CSRF automatique
* CSP et headers sécurité
* Static files servis

---
