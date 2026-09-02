# Runique — l'expérience développeur Django, en Rust type-safe

![Rust](https://img.shields.io/badge/rust-1.94%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-2375%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-2.2.0-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

Déclarez un modèle une fois, et vous récupérez la table en base, la migration, un formulaire type-safe et un panel admin complet — sans câblage supplémentaire. Runique apporte la productivité de Django à Rust sans vous demander de renoncer à sa sécurité ni à ses performances. Construit sur Axum, SeaORM et Tera, il s'efface une fois le code répétitif écarté.

> **Statut, sans détour :** développement actif. Le crate du framework (`runique`) fait foi ; `demo-app` est une vraie application testée contre lui, pas une démo jetable. Le panel admin est en **bêta**. Rien ci-dessous n'est enjolivé — voir l'[état du projet](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) pour la version sans filtre.

🌍 **Langues** : [English](https://runique.io/readme/en) | Français

---

## Des macros déclaratives, pas du code répétitif

```rust
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    enums: { Status: [Draft="Draft", Published="Published"], },
    {
        title:  text [required],
        slug:   text [unique],
        body:   richtext [required],
        status: choice [enum(Status), default: "Draft"],
        views:  int [default: 0],
    }
}
```

`model!` génère l'entité SeaORM (`article::Model`) et sa migration SQL (`runique makemigrations`) à partir de la même déclaration. Ajoutez `#[form]` et vous obtenez un formulaire type-safe correspondant, validé côté serveur et dérivable directement du schéma. Enregistrez la ressource dans `admin!` et le CRUD est déjà là — liste, recherche, filtres, permissions, tout y est :

```rust
admin! {
    article: article::Model => ArticleForm {
        title: "Articles",
        list_display: [["title", "Title"], ["status", "Status"], ["views", "Views"]],
        search_fields: ["title", "body"],
        list_filter:   [["status", "Status", 5]],
    }
}
```

<!-- Ajouter une vraie capture d'écran de l'admin généré ici — ça vend le framework mieux qu'un paragraphe : -->
<!-- ![Runique admin panel](docs/assets/admin.png) -->

---

## Pourquoi Runique

Rust a déjà de bonnes briques web, rapides et bas niveau — ce qui lui manque, c'est un framework offrant d'entrée la productivité quotidienne de Django. Assembler soi-même un ORM, un moteur de templates, une couche de formulaires et un admin est un projet à part entière avant même d'avoir écrit une seule fonctionnalité. Runique fait ce câblage à votre place, selon un jeu de conventions cohérent, pour que le temps aille dans votre application plutôt que dans sa plomberie — sans rien perdre de la sécurité des types ni des performances de Rust.

| Django (Python) | Runique (Rust) |
|---|---|
| `models.py` | `model!` → entité SeaORM + migration |
| `forms.py` | `#[form]` formulaires type-safe |
| `admin.py` | `admin!` panel admin généré |
| `urls.py` | `urlpatterns!` macro de routing |
| Templates Django | Tera (auto-échappé) |
| QuerySet | SeaORM + DSL de requêtes `search!` |
| middleware | slots de middleware ordonnés |

Pour le tableau complet : [Runique vs Django](https://runique.io/docs/fr/comparatif).

---

## Sécurité par défaut

Rien de tout ça n'est ajouté après coup — c'est la base dont vous partez :

- le CSRF compare les tokens en temps constant (`ct_eq`), pour qu'aucun timing ne trahisse une correspondance
- le CSP embarque un nonce par réponse, configurable via le builder
- le login est timing-safe (pas d'énumération d'utilisateurs par le temps de réponse), et les mots de passe sont hashés en Argon2
- les sessions sont persistées en base, avec priorité donnée aux sessions authentifiées quand la mémoire se tend
- les tokens de réinitialisation de mot de passe vivent en base, hashés SHA-256, à usage unique, durcis contre l'IDOR
- les sorties sont assainies (ammonia) en plus de l'auto-échappement de Tera, et la validation des hôtes autorisés est appliquée

[Politique de sécurité](https://runique.io/docs/fr/middleware)

---

## Démarrage rapide

```bash
runique new myapp
cd myapp
cargo run            # votre app est un binaire Rust normal
```

> `runique start` régénère le code CRUD admin depuis vos déclarations
> `admin!`, puis lance `cargo run` lui-même — une étape de génération
> one-shot enchaînée au lancement, pas un watcher en tâche de fond (voir
> [Admin (bêta)](#admin-beta)). Un simple `cargo run` saute la régénération.

Un `main.rs` allégé (version complète dans `demo-app/src/main.rs`) :

```rust,no_run
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env();
    let db = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;
    Ok(())
}
```

Les routes passent par la macro `urlpatterns!` et ressortent sous forme d'un `Router` Axum classique :

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/"          => view!{ index },        name = "index",
        "/blog/{id}" => view!{ blog_detail },  name = "blog_detail",
    }
    .rate_limit("/login", "login", view!(login_user), 10, 60, vec![Method::POST])
}
```

Pour le guide complet : [Installation](https://runique.io/docs/fr/installation)

---

## Contenu de ce dépôt

- `runique/` — le crate du framework lui-même, le produit et la source de vérité
- `demo-app/` — une vraie application construite contre le framework, utilisée pour le valider
- `docs/` — documentation en anglais et en français

Version du workspace (source de vérité) : **2.2.0**.

---

## CLI

`runique` fournit :

- `runique new <name>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]` — régénère le code admin puis lance l'app (one-shot, pas un watcher)
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

> ⚠️ **À propos du rollback de migrations**
> `runique makemigrations` écrit les migrations en préservant l'ordre
> chronologique du système de migrations. Si vous devez un jour en revenir
> en arrière, passez plutôt par le CLI SeaORM : il garde la table de suivi
> des migrations synchronisée avec l'état réel du schéma. Mélanger les deux
> chemins de rollback peut désynchroniser ce suivi.

---

## Admin (bêta)

`runique start` fait trois choses, dans l'ordre, sur un seul thread :

1. il parse vos déclarations `admin!` dans `src/admin.rs`
2. il génère le code CRUD sous `src/admins/`
3. il lance `cargo run --release`, bloquant

Il vérifie d'abord si `.with_admin(...)` existe dans `src/main.rs`, et ne génère/lance que si c'est le cas — sinon il quitte avec un message expliquant pourquoi. Pas de surveillance continue : relancez `runique start` pour régénérer après une modification de `src/admin.rs`.

C'est encore de la bêta : les permissions fonctionnent surtout au niveau des ressources pour l'instant, le dossier généré `src/admins/` est écrasé à chaque régénération, et le durcissement est en cours plutôt que terminé.

Documentation admin : [Admin](https://runique.io/docs/fr/admin)

---

## Features et bases de données

Activées par défaut : `orm`, `all-databases`.

Backends sélectionnables individuellement : `sqlite`, `postgres`, `mysql`, `mariadb`.

---

## Sessions

`CleaningMemoryStore` remplace le `MemoryStore` par défaut : il ajoute un nettoyage automatique des sessions expirées, un système de double watermark (128 Mo / 256 Mo) pour borner la mémoire, et une priorité pour les sessions authentifiées — elles sont purgées en dernier sous pression, et survivent aux redémarrages grâce à un fallback en base.

Référence complète : [Sessions](https://runique.io/docs/fr/session)

---

## Tests et couverture

- Tests rapportés : **2375 réussis** (2 ignorés)
- Snapshot de couverture (`2026-09-02`, package `runique`, module admin inclus) : fonctions **75.54%**, lignes **73.69%**, régions **72.30%**

```bash
cargo llvm-cov --package runique --summary-only
```

Détail complet par fichier : [docs/couverture_test.md](docs/couverture_test.md)

---

## Documentation

- [Installation](https://runique.io/docs/fr/installation)
- [Architecture](https://runique.io/docs/fr/architecture)
- [Configuration](https://runique.io/docs/fr/configuration)
- [Routing](https://runique.io/docs/fr/routing)
- [Formulaires](https://runique.io/docs/fr/formulaire)
- [Modèle/Schéma](https://runique.io/docs/fr/model)
- [Templates](https://runique.io/docs/fr/template)
- [ORM](https://runique.io/docs/fr/orm)
- [Middlewares](https://runique.io/docs/fr/middleware)
- [Messages flash](https://runique.io/docs/fr/flash)
- [Exemples](https://runique.io/docs/fr/exemple)
- [Admin bêta](https://runique.io/docs/fr/admin)
- [Sessions](https://runique.io/docs/fr/session)
- [Variables d'environnement](https://runique.io/docs/fr/env)

---

## État du projet & ressources

- [État du projet](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) — tenu à jour au fil de l'avancement
- [Changelog](https://runique.io/changelog)
- [Runique vs Django — comparatif des fonctionnalités](https://runique.io/docs/fr/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Politique de sécurité](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## Licence

MIT — voir [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
