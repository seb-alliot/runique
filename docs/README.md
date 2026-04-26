  Pas de souci ! Vous avez partagé le même fichier deux fois — il est bien **en anglais**, pas en français. 😄

Voici la traduction complète en français :

---

# Runique — Framework Rust inspiré de Django

![Rust](https://img.shields.io/badge/rust-1.88%2B-orange)
![Tests passing](https://img.shields.io/badge/tests-1930%2B%20passing-green)
![License](https://img.shields.io/badge/license-MIT-green)
![Version](https://img.shields.io/badge/version-2.1.0-blue)
[![Crates.io](https://img.shields.io/crates/v/runique)](https://crates.io/crates/runique)
[![Runique](https://img.shields.io/badge/Runique-brightgreen)](https://runique.io)

Runique est un framework web construit sur Axum, axé sur les formulaires fortement typés, les middlewares de sécurité, le rendu de templates, l'intégration ORM et un workflow d'administration généré par code.

> État actuel : développement actif. La source de vérité du framework est la crate `runique`.
> `demo-app` est utilisée comme application de validation/test pour le comportement du framework.

🌍 **Langues** : English | [Français](https://runique.io/readme/fr)

---

## Contenu de ce dépôt

- `runique/` → crate du framework (produit principal)
- `demo-app/` → application de test/validation pour le développement du framework
- `docs/` → documentation EN/FR

Version du workspace (source de vérité) : **2.1.0**.

---

## Capacités principales

- Système de formulaires fortement typés (`forms`, extracteurs, validateurs, renderers)
- Macros de routage et aides d'URL
- Intégration de templates Tera et aides de contexte
- Middlewares de sécurité (CSRF, CSP, hôtes autorisés, assainissement, auth/session)
- Intégration SeaORM + outillage de migrations
- Système de messages flash
- Admin bêta (macro `admin!` + code CRUD généré par démon)

Les principaux modules publics sont exposés depuis `runique/src/lib.rs`.

---

## Installation

```bash
git clone https://github.com/seb-alliot/runique
cd runique
cargo build --workspace
cargo test --workspace
```

Guide détaillé : [Installation](https://runique.io/docs/en/installation)

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

- `runique new <nom>`
- `runique start [--main src/main.rs] [--admin src/admin.rs]`
- `runique create-superuser`
- `runique makemigrations --entities src/entities --migrations migration/src [--force false]`
- `runique migration up|down|status --migrations migration/src`

> ⚠️ **Avertissement**
> La commande `makemigrations` génère des tables SeaORM tout en préservant l'ordre chronologique du système de migrations.
> Pour garantir la cohérence du suivi des migrations, utilisez uniquement le CLI SeaORM pour appliquer ou gérer les migrations.
> L'utilisation d'autres commandes peut entraîner une désynchronisation des migrations.

## État de l'admin bêta

Comportement du démon admin dans `start` :

- vérifie si `.with_admin(...)` existe dans `src/main.rs`
- démarre le watcher admin quand activé
- sinon quitte avec un message explicite

Les ressources admin sont déclarées dans `src/admin.rs` via `admin!`.

Le workflow :

1. analyser les déclarations `admin!`
2. générer le code admin sous `src/admins/`
3. rafraîchir en cas de changement avec le mode watcher

Limites actuelles de la bêta :

- permissions principalement au niveau des ressources
- écrasement du dossier généré (`src/admins/`)
- durcissement itératif toujours en cours

Documentation admin : [Admin](https://runique.io/docs/en/admin)

---

## Fonctionnalités et backends de base de données

Fonctionnalités par défaut :

- `orm`
- `all-databases`

Backends sélectionnables :

- `sqlite`
- `postgres`
- `mysql`
- `mariadb`

---

## Aperçu des tests et de la couverture

- Tests rapportés : **1930+ passants**
- Aperçu de la couverture (`2026-04-26`, package `runique`) :
  - Fonctions : **81,95 %**
  - Lignes : **78,45 %**
  - Régions : **76,25 %**

```bash
cargo llvm-cov --tests --package runique --ignore-filename-regex "admin" --summary-only
```

Répartition complète par fichier : [docs/couverture_test.md](docs/couverture_test.md)

---

## Sessions

`CleaningMemoryStore` remplace le `MemoryStore` par défaut avec un nettoyage automatique des sessions expirées, un système de seuils à deux niveaux (128 Mo / 256 Mo) et une protection prioritaire des sessions authentifiées.

- Seuil bas : purge en arrière-plan des sessions anonymes expirées
- Seuil haut : purge d'urgence synchrone + refus 503 si toujours dépassé
- `protect_session(&session, duration_secs)` — marque une session anonyme comme intouchable jusqu'à un timestamp donné
- Clé `user_id` — protège automatiquement les sessions authentifiées

Référence complète : [Sessions](https://runique.io/docs/en/session)

---

## Variables d'environnement

Tout le comportement est configurable via `.env`. Variables clés :

```env
RUNIQUE_SESSION_CLEANUP_SECS=60
RUNIQUE_SESSION_LOW_WATERMARK=134217728
RUNIQUE_SESSION_HIGH_WATERMARK=268435456
SECRET_KEY=your-secret-key
DATABASE_URL=sqlite://db.sqlite3
```

Référence complète : [Variables d'environnement](https://runique.io/docs/en/env)

---

## Documentation

- [Installation](https://runique.io/docs/en/installation)
- [Architecture](https://runique.io/docs/en/architecture)
- [Configuration](https://runique.io/docs/en/configuration)
- [Routage](https://runique.io/docs/en/routing)
- [Formulaires](https://runique.io/docs/en/formulaire)
- [Modèle/Schéma](https://runique.io/docs/en/model)
- [Templates](https://runique.io/docs/en/template)
- [ORM](https://runique.io/docs/en/orm)
- [Middlewares](https://runique.io/docs/en/middleware)
- [Messages Flash](https://runique.io/docs/en/flash)
- [Exemples](https://runique.io/docs/en/exemple)
- [Admin bêta](https://runique.io/docs/en/admin)
- [Sessions](https://runique.io/docs/en/session)
- [Variables d'environnement](https://runique.io/docs/en/env)

---

## État du projet

Pour le rapport d'état détaillé et continuellement mis à jour, voir [PROJECT_STATUS.md](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md).

## Ressources

- [Changelog](https://runique.io/changelog)
- [Runique vs Django — Comparaison des fonctionnalités](https://runique.io/docs/en/comparatif)
- [Crates.io](https://crates.io/crates/runique)
- [Politique de sécurité](https://github.com/seb-alliot/runique/blob/main/SECURITY.md)

---

## Licence

MIT — voir [LICENSE](https://github.com/seb-alliot/runique/blob/main/LICENSE)

---
