# Runique — l'expérience développeur Django, en Rust type-safe

![Rust](https://img.shields.io/badge/rust-1.94%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-2375%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-2.2.0-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

**Déclarez un modèle une seule fois — obtenez la table en base, la migration, un formulaire type-safe *et* un panel admin complet.** Runique est un framework web batteries-included qui apporte la productivité de Django à Rust, sans sacrifier la sécurité et les performances de Rust. Construit sur Axum, SeaORM et Tera.

> **Statut — honnête :** développement actif. Le crate du framework (`runique`) est la source de vérité ; `demo-app` est une véritable application de validation testée contre lui. L'admin est en **bêta**. Rien ci-dessous n'est exagéré — voir [État du projet](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md).

🌍 **Langues** : [English](https://runique.io/readme/en) | Français

---

## Des macros déclaratives, pas du boilerplate

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

`model!` génère l'**entité SeaORM** (`article::Model`) et sa **migration SQL** (`runique makemigrations`). Un **formulaire type-safe** correspondant est déclaré avec `#[form]` (validé côté serveur, dérivable depuis le schéma). Enregistrez la ressource et vous obtenez un **CRUD admin complet** — affichage de liste, recherche, filtres, permissions :

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

Rust dispose de briques web rapides et bas niveau — mais d'aucun framework *batteries-included* avec la productivité de Django. Assembler à la main un ORM, un moteur de templates, une couche de formulaires et un admin est un projet en soi. Runique les intègre, de façon conventionnelle, pour que vous livriez des fonctionnalités plutôt que de la plomberie — tout en gardant la sécurité des types et la performance.

| Django (Python) | Runique (Rust) |
|---|---|
| `models.py` | `model!` → entité SeaORM + migration |
| `forms.py` | `#[form]` formulaires type-safe |
| `admin.py` | `admin!` panel admin généré |
| `urls.py` | `urlpatterns!` macro de routing |
| Templates Django | Tera (auto-échappé) |
| QuerySet | SeaORM + DSL de requêtes `search!` |
| middleware | slots de middleware ordonnés |

Comparatif complet : [Runique vs Django](https://runique.io/docs/fr/comparatif).

---

## Sécurité par défaut

La sécurité est intégrée par construction, pas ajoutée après coup :

- **CSRF** protégé avec comparaison de token en temps constant (`ct_eq`)
- **CSP** avec nonces par réponse, configurable via le builder
- **Auth** : login timing-safe (pas d'énumération d'utilisateurs), hashage de mot de passe Argon2
- **Sessions** persistées avec protection prioritaire des utilisateurs authentifiés
- **Réinitialisation de mot de passe** : tokens persistés en base, hashés SHA-256, à usage unique, durcis contre l'IDOR
- **Sanitisation des sorties** (ammonia) + auto-échappement Tera, validation des hôtes autorisés

[Politique de sécurité](https://runique.io/docs/fr/middleware)

---

## Démarrage rapide

```bash
runique new myapp
cd myapp
cargo run            # votre app est un binaire Rust normal
```

> `runique start` n'est **pas** la commande pour lancer l'app — c'est le
> générateur de code admin : il surveille vos déclarations `admin!` et
> régénère le code CRUD (voir [Admin (bêta)](#admin-beta)).

Un `main.rs` simplifié (version complète dans `demo-app/src/main.rs`) :

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

Les routes sont déclarées avec la macro `urlpatterns!` et retournent un `Router` Axum :

```rust
pub fn routes() -> Router {
    urlpatterns! {
        "/"          => view!{ index },        name = "index",
        "/blog/{id}" => view!{ blog_detail },  name = "blog_detail",
    }
    .rate_limit("/login", "login", view!(login_user), 10, 60, vec![Method::POST])
}
```

Guide détaillé : [Installation](https://runique.io/docs/fr/installation)

---

## Contenu de ce dépôt

- `runique/` → crate du framework (le produit, source de vérité)
- `demo-app/` → application de validation testée contre le framework
- `docs/` → documentation EN/FR

Version du workspace (source de vérité) : **2.2.0**.

---

## CLI

`runique` fournit :

- `runique new <name>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]` — générateur de code admin (one-shot) + lancement de l'app en une seule commande
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

> ⚠️ **Attention — rollback de migrations**
> `runique makemigrations` génère les migrations en préservant l'ordre
> chronologique du système de migrations. Quand vous devez **revenir en
> arrière** sur une migration, privilégiez le CLI SeaORM : il garde la table
> de suivi des migrations synchronisée avec l'état réel du schéma. Mélanger
> les outils de rollback peut désynchroniser le suivi des migrations.

---

## Admin (bêta)

`runique start` exécute, séquentiellement sur un seul thread :

1. parse vos déclarations `admin!` dans `src/admin.rs`
2. génère le code CRUD sous `src/admins/`
3. lance `cargo run --release` (bloquant)

Il vérifie d'abord si `.with_admin(...)` existe dans `src/main.rs` et ne génère/lance que si c'est activé, sinon il quitte avec un message explicite. Ce n'est pas un watcher : pour régénérer après une modification de `src/admin.rs`, relancez `runique start`.

Limites actuelles (bêta) : permissions principalement au niveau des ressources, le dossier généré `src/admins/` est écrasé, durcissement progressif en cours.

Documentation admin : [Admin](https://runique.io/docs/fr/admin)

---

## Features et bases de données

Features par défaut : `orm`, `all-databases`.

Backends disponibles : `sqlite`, `postgres`, `mysql`, `mariadb`.

---

## Sessions

`CleaningMemoryStore` remplace le `MemoryStore` par défaut avec un nettoyage automatique des sessions expirées, un système de double watermark (128 Mo / 256 Mo), et une protection prioritaire des sessions authentifiées (purgées en dernier, survivent aux redémarrages via un fallback DB).

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

- [État du projet](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md) — rapport d'état continuellement mis à jour
- [Changelog](https://runique.io/changelog)
- [Runique vs Django — comparatif des fonctionnalités](https://runique.io/docs/fr/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Politique de sécurité](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## Licence

MIT — voir [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
