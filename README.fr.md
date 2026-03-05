# 🚀 Runique — Framework Rust inspiré de Django

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1523%2F1523%20passing-orange)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-1.1.36-blue)
![Crates.io](https://img.shields.io/crates/v/runique)

**Runique** est un framework web construit sur Axum, axé sur les formulaires **type-safe**, les middlewares de sécurité, le rendu de templates, l’intégration ORM et un workflow d’administration généré par code.

> État actuel : développement actif. La source de vérité du framework est la crate `runique`.
> `demo-app` est utilisée comme application de validation/test pour le comportement du framework.

🌍 **Langues** : [English](README.md) | [Français](README.fr.md)

---

# Ce que contient ce dépôt

* `runique/` → crate du framework (produit principal)
* `demo-app/` → application de test/validation pour le développement du framework
* `docs/` → documentation EN/FR

Version du workspace (source de vérité) : **1.1.36**.

---

# Capacités principales

* Système de formulaires **type-safe** (`forms`, extracteurs, validateurs, renderers)
* Macros de routage et helpers d’URL
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

Guide détaillé :
`docs/fr/01-installation.md`

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
* `runique makemigrations --entities src/frtities --migrations migration/src [--force false]`
* `runique migration up|down|status --migrations migration/src`

```

```

⚠️ Attention
Seule la commande makemigration permet de préserver correctement la chronologie des migrations de SeaORM.
L’utilisation des autres commandes peut désynchroniser le suivi des migrations.

```

Comportement du daemon admin dans `start` :

* vérifie si `.with_admin(...)` existe dans `src/main.rs`
* démarre le watcher admin si activé
* sinon quitte avec un message explicite

---

# Statut de l’admin (bêta)

Les ressources admin sont déclarées dans `src/admin.rs` via `admin!`.

Workflow :

1. parser les déclarations `admin!`
2. générer le code admin dans `src/admins/`
3. rafraîchir automatiquement lors des modifications grâce au mode watcher

Limites actuelles de la bêta :

* permissions principalement au niveau des ressources
* écrasement du dossier généré (`src/admins/`)
* renforcement progressif encore en cours

Documentation admin :
`docs/fr/11-Admin.md`

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

* Tests rapportés : **1523/1523 réussis**
* Snapshot de couverture (`2026-03-01`, package `runique`) :

  * Fonctions : **76.66%**
  * Lignes : **71.04%**
  * Régions : **67.22%**

Commande utilisée pour la couverture :

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

Voir :
[couverture_test.md](couverture_test.md)

---

# Carte de la documentation

## Anglais

* Installation — `docs/fr/01-installation.md`
* Architecture — `docs/fr/02-architecture.md`
* Configuration — `docs/fr/03-configuration.md`
* Routing — `docs/fr/04-routing.md`
* Forms — `docs/fr/05-forms.md`
* Model/Schema — `docs/fr/12-model.md`
* Templates — `docs/fr/06-templates.md`
* ORM — `docs/fr/07-orm.md`
* Middleware — `docs/fr/08-middleware.md`
* Flash Messages — `docs/fr/09-flash-messages.md`
* Examples — `docs/fr/10-examples.md`
* Admin bêta — `docs/fr/11-Admin.md`

## Français

* Installation — `docs/fr/01-installation.md`
* Architecture — `docs/fr/02-architecture.md`
* Configuration — `docs/fr/03-configuration.md`
* Routage — `docs/fr/04-routing.md`
* Formulaires — `docs/fr/05-forms.md`
* Model/Schema —
  [https://github.com/seb-alliot/runique/blob/main/docs/fr/12-model.md](https://github.com/seb-alliot/runique/blob/main/docs/fr/12-model.md)
* Templates — `docs/fr/06-templates.md`
* ORM — `docs/fr/07-orm.md`
* Middlewares — `docs/fr/08-middleware.md`
* Flash Messages — `docs/fr/09-flash-messages.md`
* Exemples — `docs/fr/10-examples.md`
* Admin bêta — `docs/fr/11-Admin.md`

---

# Statut du projet

Pour un rapport détaillé et continuellement mis à jour de l’état du projet, voir :

`PROJECT_STATUS.md`

---

# Ressources

* Structure du projet — `INDEX.md`
* Changelog — `CHANGELOG.md`
* Hub de documentation — `docs/README.md`
* Politique de sécurité — `SECURITY.md`

---

# Licence

MIT — voir `LICENCE`
