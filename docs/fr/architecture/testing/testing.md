# Tester une application Runique

Runique teste votre **logique métier contre une vraie base de données** : chaque test s'exécute dans une transaction toujours annulée, et affiche le SQL qu'il a exécuté à côté de son résultat. C'est le builder `runique_test` (feature `test-utils`) et la commande `runique test`.

---

## 1. Mise en place

`test-utils` se déclare **uniquement** dans `[dev-dependencies]` : elle compile du code qui ne doit jamais arriver dans un build de production.

```toml
# Cargo.toml
[dependencies]
runique = { version = "3.0.0", features = ["postgres"] }

[dev-dependencies]
runique = { version = "3.0.0", features = ["test-utils"] }
```

Les tests vivent dans `src/runique_test/`, un fichier par domaine, déclarés derrière `cfg(test)` :

```rust
// src/main.rs
#[cfg(test)]
mod runique_test;
```

```rust
// src/runique_test/mod.rs
/// Le fichier d'environnement où tous les tests de ce dossier lisent leur base.
pub const ENV: &str = ".env";

mod blog;
mod user;
```

Le fichier d'environnement doit nommer sa base (`DATABASE_URL` ou `DB_ENGINE`) : sinon le test est refusé, plutôt que de créer en silence un fichier SQLite local. Rien n'est lu dans l'environnement du shell, où `DATABASE_URL` pourrait pointer n'importe où.

---

## 2. Écrire un test

Un test est un `#[tokio::test]` qui renvoie `Result<(), TestFailure>`, dont la dernière expression est `runique_test` :

```rust
// src/runique_test/blog.rs
use crate::backend::blog::get_article;
use crate::entities::blog;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;

#[tokio::test]
async fn un_article_cree_est_trouve_par_son_id() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let article = blog::ActiveModel {
            title: Set("Annulé à la fin du test".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        get_article(db, article.id)
            .await
            .map(|_| ())
            .ok_or_else(|| DbErr::Custom("l'article doit être trouvé par son id".into()))
    })
    .await
}
```

`db` est un `&ADb` : le même type que `request.engine.db`, vos fonctions métier s'exécutent donc telles quelles. Quoi qu'il arrive, la transaction est annulée.

### Ce qui fait échouer un test

- le handler renvoie `Err`, panique, ou dépasse le délai (`RUNIQUE_TEST_TIMEOUT` dans le fichier d'environnement, 30 s par défaut) — `runique_test` ne panique jamais, il renvoie `Err(TestFailure)` avec son message ;
- **une requête a échoué alors que le handler a renvoyé `Ok`** : une erreur a été avalée quelque part (`.ok()`, `unwrap_or_default()`…) ;
- la transaction a été terminée depuis le test (un `COMMIT` brut, ou du DDL sous MariaDB/MySQL, qui valide implicitement).

### Refus attendus

Un refus que vous *voulez* (contrainte unique, clé étrangère, `NOT NULL`) passe par `expect_db_error` : il s'exécute dans un savepoint — sous Postgres, une instruction en échec hors savepoint gâche toute la transaction — et il est marqué comme attendu dans la trace.

```rust
use runique::runique_test::expect_db_error;

let refus = expect_db_error(db, async |sp| {
    contribution(id_utilisateur_supprime).insert(sp).await
})
.await?; // Err si la base l'a accepté
```

---

## 3. Lancer les tests

```bash
runique test                    # tous les tests de src/runique_test/
runique test blog               # les tests de src/runique_test/blog.rs
runique test blog un_article_cree_est_trouve_par_son_id   # un seul test
```

`runique test` vérifie d'abord la mise en place — `test-utils` uniquement en `[dev-dependencies]` (refusé s'il atteint un build de production via `[dependencies]`, `[workspace.dependencies]` ou une entrée de `[features]`), le module `runique_test` derrière `cfg(test)`, chaque fichier déclaré dans `mod.rs` — puis lance `cargo test` un test à la fois, sortie affichée. `cargo test` seul fonctionne aussi : les tests sont sérialisés dans le processus.

Un autre moteur (MongoDB…) peut se brancher en implémentant le trait `TestTransaction` ; `ADb` (SeaORM) est l'implémentation fournie.

---

## 4. Contrôles au démarrage

`RuniqueApp::builder(config).build().await` vérifie la configuration avant de démarrer.

**Dans tous les modes :**

- **Database** : ni `.with_database(...)` ni `.with_database_config(...)` ;
- **AdminPanel** : un préfixe d'admin vide, ou une entrée d'`extra_routes` qui nomme une ressource non enregistrée ;
- **static_cache** / **media_cache** : une valeur de cache qui n'est pas un en-tête HTTP valide ;
- **MediaRoot** : un dossier media impossible à créer ;
- **CORS** : `any_origin()` combiné à `allow_credentials(true)`.

**Hors mode debug uniquement** (`DEBUG=false`), ce qui serait dangereux en production :

- **Security** : une `SECRET_KEY` faible ;
- **PublicUrl** : le reset de mot de passe ou l'admin activés sans `.with_public_url(...)` ;
- **ACME** : `ACME_ENABLED` sans `ACME_DOMAIN` ou `ACME_EMAIL` (feature `acme`).

`build()` renvoie alors une `BuildError` — `BuildErrorKind::CheckFailed(CheckReport)` pour les contrôles ci-dessus, CORS à part — affichée dans le terminal avec une suggestion pour chaque problème.

---

← [**Exemples**](/docs/fr/exemple) | [**Dépannage**](/docs/fr/installation/troubleshooting) →
