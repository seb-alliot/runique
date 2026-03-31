# Runique — Framework Rust inspiré de Django

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1731%2F1731%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.54-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

Runique est un framework web basé sur Axum, axé sur les formulaires typés, les middlewares de sécurité, le rendu de templates, l’intégration ORM, et un workflow d’administration généré automatiquement.

> État actuel : en développement actif. La source de vérité du framework est le crate `runique`.
> `demo-app` est utilisée comme application de validation/test du comportement du framework.

🌍 **Langues** : Anglais | [Français](https://runique.io/readme/fr)

---

## Contenu de ce dépôt

* `runique/` → crate du framework (produit principal)
* `demo-app/` → application de test/validation pour le développement du framework
* `docs/` → documentation EN/FR

Version du workspace (source de vérité) : **1.1.54**.

---

## Capacités principales

* Système de formulaires typés (`forms`, extracteurs, validateurs, renderers)
* Macros de routing et helpers d’URL
* Intégration du moteur de templates Tera et helpers de contexte
* Middlewares de sécurité (CSRF, CSP, hôtes autorisés, sanitisation, auth/session)
* Intégration SeaORM + outils de migration
* Système de messages flash
* Admin bêta (macro `admin!` + génération automatique de code CRUD via daemon)

Les modules publics principaux sont exposés depuis `runique/src/lib.rs`.

---

## Installation

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build --workspace
cargo test --workspace
```

Guide détaillé : Installation
[https://runique.io/docs/en/installation](https://runique.io/docs/en/installation)

---

## Utilisation rapide

```rust,no_run
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();
    let app = RuniqueApp::builder(config).build().await.unwrap();
    app.run().await.unwrap();
}
```

---

## CLI (commandes actuelles)

`runique` fournit :

* `runique new <name>`
* `runique start [--main src/main.rs] [--admin src/admin.rs]`
* `runique create-superuser`
* `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
* `runique migration up|down|status --migrations migration/src`

> ⚠️ **Attention**
> La commande `makemigrations` génère les tables SeaORM tout en respectant l’ordre chronologique du système de migrations.
> Pour garantir la cohérence du suivi des migrations, utilisez uniquement le CLI SeaORM pour les appliquer ou les gérer.
> L’utilisation d’autres commandes peut entraîner une désynchronisation des migrations.

---

## Statut de l’admin (bêta)

Comportement du daemon admin dans `start` :

* vérifie si `.with_admin(...)` est présent dans `src/main.rs`
* démarre le watcher admin si activé
* sinon quitte avec un message explicite

Les ressources admin sont déclarées dans `src/admin.rs` via `admin!`.

Workflow :

1. parse des déclarations `admin!`
2. génération du code admin dans `src/admins/`
3. rafraîchissement automatique via watcher

Limites actuelles (bêta) :

* permissions principalement au niveau des ressources
* écrasement du dossier généré (`src/admins/`)
* durcissement progressif en cours

Documentation admin :
[https://runique.io/docs/en/admin](https://runique.io/docs/en/admin)

---

## Features et bases de données

Features par défaut :

* `orm`
* `all-databases`

Backends disponibles :

* `sqlite`
* `postgres`
* `mysql`
* `mariadb`

---

## Tests et couverture

* Tests : **1731/1731 réussis**
* Snapshot de couverture (`2026-03-01`, package `runique`) :

  * Fonctions : **76.66%**
  * Lignes : **71.04%**
  * Régions : **67.22%**

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

---

## Sessions

`CleaningMemoryStore` remplace le `MemoryStore` par défaut avec :

* nettoyage automatique des sessions expirées
* système de double watermark (128 MB / 256 MB)
* protection prioritaire des sessions authentifiées

Fonctionnement :

* Low watermark : purge en arrière-plan des sessions anonymes expirées
* High watermark : purge d’urgence synchronisée + refus 503 si dépassement
* `protect_session(&session, duration_secs)` → protège une session anonyme jusqu’à un timestamp
* clé `user_id` → protège automatiquement les sessions authentifiées

Référence complète :
[https://runique.io/docs/en/session](https://runique.io/docs/en/session)

---

## Variables d’environnement

Tout est configurable via `.env`. Variables principales :

```env
RUNIQUE_SESSION_CLEANUP_SECS=60
RUNIQUE_SESSION_LOW_WATERMARK=134217728
RUNIQUE_SESSION_HIGH_WATERMARK=268435456
SECRET_KEY=your-secret-key
DATABASE_URL=sqlite://db.sqlite3
```

Référence complète :
[https://runique.io/docs/en/env](https://runique.io/docs/en/env)

---

## Documentation

* Installation
* Architecture
* Configuration
* Routing
* Formulaires
* Modèle/Schéma
* Templates
* ORM
* Middlewares
* Messages flash
* Exemples
* Admin (bêta)
* Sessions
* Variables d’environnement

---

## État du projet

Pour un rapport détaillé et continuellement mis à jour :
PROJECT_STATUS.md
[https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md)

---

## Ressources

* Changelog
  [https://runique.io/changelog](https://runique.io/changelog)

* Runique vs Django — Comparaison des fonctionnalités
  [https://runique.io/docs/en/comparatif](https://runique.io/docs/en/comparatif)

* Crates.io
  [https://crates.io/crates/runique](https://crates.io/crates/runique)

* Politique de sécurité
  [https://github.com/seb-alliot/runique/blob/main/SECURITY.md](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## Licence

MIT — voir LICENSE
[https://github.com/seb-alliot/runique/blob/main/LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)

---