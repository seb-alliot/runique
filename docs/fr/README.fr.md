# 🚀 Runique — Framework Rust inspiré de Django

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1731%2F1731%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.52-blue)
![Crates.io](https://img.shields.io/crates/v/runique)
[![Runique](https://img.shields.io/badge/Live-Demo-brightgreen)](https://runique-production.up.railway.app)

**Runique** est un framework web construit sur Axum, axé sur les formulaires **type-safe**, les middlewares de sécurité, le rendu de templates, l'intégration ORM et un workflow d'administration généré par code.

> État actuel : développement actif. La source de vérité du framework est la crate `runique`.
> `demo-app` est utilisée comme application de validation/test pour le comportement du framework.

🌍 **Langues** : [English](/readme/en) | [Français](/readme/fr)

---

# Ce que contient ce dépôt

* `runique/` → crate du framework (produit principal)
* `demo-app/` → application de test/validation pour le développement du framework
* `docs/` → documentation EN/FR

Version du workspace (source de vérité) : **1.1.52**.

---

# Capacités principales

* Système de formulaires **type-safe** (`forms`, extracteurs, validateurs, renderers)
* Macros de routage et helpers d'URL
* Intégration des templates Tera et helpers de contexte
* Middlewares de sécurité (CSRF, CSP, allowed hosts, sanitisation, auth/session)
* Intégration de SeaORM + outils de migration
* Système de messages flash
* Admin bêta (macro `admin!` + génération de CRUD par daemon)

Les principaux modules publics sont exposés depuis `runique/src/lib.rs`.

---

# Installation

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build --workspace
cargo test --workspace
```

Guide détaillé : [Installation](/docs/fr/installation)

---

# Utilisation rapide

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();
    let app = RuniqueApp::builder(config).build().await.unwrap();
    app.run().await.unwrap();
}
```

---

# CLI (commandes actuelles)

`runique` fournit :

* `runique new <name>`
* `runique start [--main src/main.rs] [--admin src/admin.rs]`
* `runique create-superuser`
* `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
* `runique migration up|down|status --migrations migration/src`

> ⚠️ **Avertissement**
> La commande `makemigrations` permet de générer les tables SeaORM tout en
> respectant la chronologie du système de migrations.
> Pour garantir la cohérence du suivi des migrations, utilisez uniquement
> la CLI de SeaORM pour appliquer ou gérer les migrations.
> L'utilisation des commandes peut entraîner une désynchronisation.

## Statut de l'admin (bêta)

Comportement du daemon admin dans `start` :

* vérifie si `.with_admin(...)` existe dans `src/main.rs`
* démarre le watcher admin si activé
* sinon quitte avec un message explicite

---

Les ressources admin sont déclarées dans `src/admin.rs` via `admin!`.

Workflow :

1. parser les déclarations `admin!`
2. générer le code admin dans `src/admins/`
3. rafraîchir automatiquement lors des modifications grâce au mode watcher

Limites actuelles de la bêta :

* permissions principalement au niveau des ressources
* écrasement du dossier généré (`src/admins/`)
* renforcement progressif encore en cours

Documentation admin : [Admin](/docs/fr/admin)

---

# Features et bases de données supportées

Features par défaut :

* `orm`
* `all-databases`

Backends sélectionnables :

* `sqlite`
* `postgres`
* `mysql`
* `mariadb`

---

# Snapshot tests et couverture

* Tests rapportés : **1731/1731 réussis**
* Snapshot de couverture (`2026-03-01`, package `runique`) :

  * Fonctions : **76.66%**
  * Lignes : **71.04%**
  * Régions : **67.22%**

Commande utilisée pour la couverture :

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

---

## Sessions

`CleaningMemoryStore` remplace le `MemoryStore` par défaut avec un nettoyage automatique des sessions expirées, un système de watermarks à deux niveaux (128 Mo / 256 Mo) et une protection prioritaire des sessions à valeur.

* Low watermark : purge asynchrone des sessions anonymes expirées
* High watermark : purge d'urgence synchrone + refus 503 si toujours dépassé
* `protect_session(&session, duration_secs)` — protège une session anonyme jusqu'à un timestamp futur
* Clé `user_id` — protège automatiquement les sessions authentifiées

Documentation complète : [Sessions](/docs/fr/session)

---

## Variables d'environnement

Tout le comportement est configurable via `.env`. Variables principales :

```env
RUNIQUE_SESSION_CLEANUP_SECS=60
RUNIQUE_SESSION_LOW_WATERMARK=134217728
RUNIQUE_SESSION_HIGH_WATERMARK=268435456
SECRET_KEY=votre-cle-secrete
DATABASE_URL=sqlite://db.sqlite3
```

Référence complète : [Variables d'environnement](/docs/fr/env)

---

# Carte de la documentation

- [Installation](/docs/fr/installation)
- [Architecture](/docs/fr/architecture)
- [Configuration](/docs/fr/configuration)
- [Routage](/docs/fr/routing)
- [Formulaires](/docs/fr/formulaire)
- [Model/Schema](/docs/fr/model)
- [Templates](/docs/fr/template)
- [ORM](/docs/fr/orm)
- [Middlewares](/docs/fr/middleware)
- [Flash Messages](/docs/fr/flash)
- [Exemples](/docs/fr/exemple)
- [Admin bêta](/docs/fr/admin)
- [Sessions](/docs/fr/session)
- [Variables d'environnement](/docs/fr/env)

---

# Ressources

* [Changelog](/changelog)
* [Runique vs Django — Comparatif](/docs/fr/comparatif)
* [GitHub](https://github.com/seb-alliot/runique)
* [Crates.io](https://crates.io/crates/runique)

---

# Licence

MIT — see [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)
