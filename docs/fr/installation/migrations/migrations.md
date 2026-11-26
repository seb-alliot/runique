# Migrations

## Le crate `migration`

Les migrations vivent dans un crate Rust à part, `migration/`, qui fait partie du workspace du projet :

```
migration/
├── Cargo.toml        runique + runique-cli + sea-orm-migration, même moteur que l'application
└── src/
    ├── main.rs       lance la CLI de SeaORM (up, down, status), avec le up de Runique
    ├── lib.rs        la liste des migrations, tenue à jour par makemigrations
    ├── m<date>_*.rs  les migrations générées
    └── snapshots/    l'état du schéma déjà migré, modèle par modèle
```

**Vous n'avez pas à le créer** : `runique new` l'écrit avec le projet, et `runique makemigrations` le crée s'il manque (projet non créé par `runique new`), avec le moteur de `DB_ENGINE`. Il n'y a rien à écrire à la main dans `lib.rs`.

### Le `Cargo.toml` attendu

Runique l'écrit une seule fois, à la création ; ensuite il est à vous et Runique n'y touche plus. Voici le fichier tel qu'il est généré (exemple Postgres) :

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "migration"
path = "src/lib.rs"

[dependencies]
runique = { version = "4.1.0", features = ["postgres"] }
runique-cli = { version = "4.1.0", default-features = false, features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
async-trait = "0.1"
```

Ce qui est obligatoire :

- **`runique`** avec la même feature de moteur que l'application (`sqlite`, `postgres`, `mysql`) : il fournit les tables du framework.
- **`runique-cli`** sans ses features par défaut, avec la même feature de moteur : il fournit les fonctions que les migrations générées appellent, et le `up` que `main.rs` donne à SeaORM.
- **`sea-orm-migration`** à la **même version exacte** que celle de Runique, avec `runtime-tokio-rustls` et le pilote du moteur : `sqlx-sqlite`, `sqlx-postgres`, ou `sqlx-mysql`.
- **Les features par défaut de `sea-orm-migration`** : elles apportent `cli::run_cli`, appelé par `src/main.rs`. Avec `default-features = false`, ajoutez `"cli"` à la liste, sinon le crate ne compile plus.
- **`tokio`** avec `macros` et `rt-multi-thread` (`#[tokio::main]` de `main.rs`), et **`async-trait`** (les migrations générées).
- **`[lib] name = "migration"`** : `main.rs` appelle `migration::Migrator`.

---

## Workflow en deux étapes

### 1. Générer les fichiers de migration

```bash
runique makemigrations
```

`makemigrations` lit vos modèles dans `src/entities/`, les compare aux snapshots, et écrit ce qui a changé :

| Passe | Source | Résultat |
| --- | --- | --- |
| 1 — Tables de l'application | Blocs `model!{}` | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Extensions du framework | Blocs `extend!{}` | `ALTER TABLE ADD / DROP COLUMN` sur une table `eihwaz_*` |
| 3 — Tables du framework | Automatique | Migrations `eihwaz_*` placées en tête de `lib.rs` |

La commande se termine par le nombre de fichiers écrits, ou « No changes detected » si rien n'a changé, puis rappelle la commande suivante.

Chaque migration générée commence par la ligne `// runique: waiting migrate up` : elle marque une migration que `runique migration up` n'a pas encore appliquée. Quand `runique migration up` a réussi, il retire cette ligne de chaque migration déclarée dans `lib.rs`, sans toucher au reste du fichier. Relancer `makemigrations` avant d'appliquer ne les empile pas : les migrations en attente sont recalculées et remplacées (voir [Fonctionnement de makemigrations](/docs/fr/architecture/makemigrations)). Runique l'écrit en première ligne et la reconnaît où qu'elle soit dans le fichier, tant qu'elle est seule sur sa ligne. Ne la retirez pas et ne l'ajoutez pas à la main.

### 2. Appliquer les migrations

```bash
runique migration up
```

Sous PostgreSQL et SQLite, le lancement est en **tout ou rien** : toutes les migrations en attente passent dans une seule transaction. Si l'une échoue, aucune n'est appliquée ni inscrite dans `seaql_migrations`, et la base est telle qu'avant la commande. Corrigez, puis relancez. Sous MySQL ce n'est pas le cas : voir « Limites actuelles » plus bas.

C'est le `main.rs` du crate `migration` qui le demande, en donnant le migrateur du projet à celui de Runique, qui n'apporte que son `up` :

```rust
cli::run_cli(runique_cli::migration::runtime::RuniqueMigrator(migration::Migrator)).await;
```

Un crate `migration` écrit avant la 4.1 contient `cli::run_cli(migration::Migrator).await;` : remplacez cette ligne par celle du dessus. Sans ce changement, `runique migration up` fonctionne comme avant, une migration à la fois, sans tout ou rien.

### Annuler ou consulter

```bash
runique migration down -n 1    # annule la dernière migration (son down() s'exécute)
runique migration status       # liste les migrations et leur état
```

Ces commandes lancent le crate `migration` (`cargo run`), qui lit `DATABASE_URL` dans `.env` : lancez-les depuis la racine du projet. `sea-orm-cli` n'est pas nécessaire, et la version de SeaORM utilisée est celle de `migration/Cargo.toml`, alignée sur Runique.

> `runique makemigrations` est le seul outil à utiliser pour **générer** les migrations. N'utilisez pas `sea-orm-cli migrate generate` : Runique tient ses propres snapshots et l'ordre de `lib.rs`.

---

## Changements destructifs — `--force`

Avant d'écrire quoi que ce soit, `makemigrations` détecte les changements destructifs et **refuse** de générer la migration, sauf avec `--force` :

- `DROP COLUMN` (perte de données)
- changement de type d'une colonne (voir les limites plus bas)
- passage `nullable` → obligatoire (nécessite une valeur par défaut ou un remplissage préalable)
- réduction de longueur (`max_length` plus petit) : des valeurs plus longues seraient tronquées ou refusées
- suppression d'une clé étrangère (risque d'enregistrements orphelins)
- ajout d'une FK `ON DELETE CASCADE` sur des données existantes

```bash
runique makemigrations --force
```

Le contrôle couvre les tables de l'application (`model!{}`) comme les extensions des tables du framework (`extend!{}`).

---

## Clés étrangères et moteur cible

Les fichiers générés sont **communs à tous les moteurs** : ce qui ne concerne qu'un moteur est choisi à l'exécution, dans le fichier.

- Les FK sont déclarées **dans le `CREATE TABLE`**, et les tables référencées sont créées avant celles qui les référencent.
- **Cycle** (deux nouvelles tables qui se référencent l'une l'autre) : sous SQLite, la clé reste dans le `CREATE TABLE` ; sous PostgreSQL / MySQL, la clé qui ferme le cycle est ajoutée par un `ALTER TABLE` une fois la table visée créée.
- Chaque colonne de `belongs_to` reçoit un index `idx_<table>_<colonne>`.
- Les enums deviennent un `CREATE TYPE … AS ENUM` sous PostgreSQL, un `VARCHAR` ailleurs.

Changer de moteur ne demande donc pas de régénérer les migrations.

---

## Tables du framework — fournies automatiquement

Cinq migrations du framework sont placées en tête de `lib.rs`, sans rien à déclarer :

| Migration | Crée | Ordre |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_superuser, is_dirigeant, is_admin, session_version, activated_at, created_at, updated_at) | 1er |
| `EihwazSessionsMigration` | `eihwaz_sessions` (sessions authentifiées persistées) | 2e |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes` | 3e |
| `EihwazHistoryMigration` | `eihwaz_history` (historique des modifications de l'admin) | 4e |
| `EihwazResetTokensMigration` | `eihwaz_reset_tokens` (jetons de réinitialisation hashés, un seul actif par compte) | 5e |

> Ne déclarez pas `eihwaz_users` dans vos entités : pour lui ajouter des colonnes, utilisez `extend!{}`.

---

## Étendre une table du framework — `extend!{}`

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        avatar: image [nullable, upload_to: "avatars/"],
        bio: textarea [nullable],
        website: url [nullable],
    }
}
```

`makemigrations` génère les `ALTER TABLE ADD COLUMN` correspondants. Le snapshot de l'extension est stocké dans `migration/src/snapshots/runique/<table>.rs`.

**Tables extensibles :** `eihwaz_users` et `eihwaz_groupes`, et elles seules. Étendre une autre table est une erreur de **compilation**.

---

## Génération atomique

Les changements des modèles et des blocs `extend!{}` sont d'abord planifiés en mémoire, validés (contrôle destructif, longueur des identifiants), puis écrits en un seul lot : migrations et `lib.rs`, tous ou aucun. Les snapshots, eux, sont écrits par `runique migration up` une fois les migrations appliquées, dans un lot à part.

- **Une erreur d'écriture** annule tout : le dossier `migration/src/` revient à son état d'avant.
- **Une coupure** (courant, `kill -9`) est réparée au lancement suivant de `runique makemigrations`. D'ici là, `runique migration up` refuse de partir.

Fichiers laissés par une coupure, refus de réparer, marche à suivre : [Écriture atomique des fichiers](/docs/fr/architecture/atomic).

---

## Limites actuelles

- **Le changement de type d'une colonne n'est pas encore migré.** `makemigrations` le bloque ; avec `--force`, la migration écrite ne contient qu'un commentaire d'avertissement, aucun `ALTER`, alors que le snapshot enregistre déjà le nouveau type. La console l'affiche en jaune. Avant `runique migration up`, écrivez à la main le `up()` et le `down()` de cette migration (`alter_table` / `modify_column` de SeaORM), ou ajoutez une nouvelle colonne, recopiez les données, puis supprimez l'ancienne. Le passage vers ou depuis un enum, lui, est migré.
- **Retirer un modèle supprime sa table**, bloqué sans `--force` : voir [Supprimer une table](/docs/fr/architecture/makemigrations). Un modèle que le lecteur ne voit plus (commenté, déplacé dans un sous-dossier ou dans `mod.rs`) compte comme retiré. Sous MySQL, une suppression appliquée ne s'annule pas si une migration suivante du même lancement échoue.
- **Ne copiez pas les snapshots d'un autre projet sans ses migrations** : un snapshot signifie « déjà migré ». `makemigrations` ne génère alors rien pour ces modèles, alors qu'aucune migration ne crée leurs tables.
- **Les noms d'index et de clés étrangères sont limités à 63 caractères, sur tous les moteurs** (ils sont générés à partir des noms de tables et de colonnes ; 63 est la limite de PostgreSQL, MySQL acceptent 64) : `makemigrations` refuse un nom trop long avant d'écrire le moindre fichier, plutôt que de le tronquer ; raccourcissez les noms en cause.
- **Sous MySQL, `runique migration up` n'est pas en tout ou rien.** MySQL valide chaque instruction de schéma d'elle-même, une transaction n'y change rien : si une migration échoue, celles d'avant restent appliquées, et celle qui a échoué peut rester à moitié faite. Corrigez la base à la main avant de relancer. Sous PostgreSQL et SQLite, le lancement entier est annulé (voir « Appliquer les migrations »), à condition que le `main.rs` du crate `migration` soit celui de la 4.1.
- **Sous SQLite, une clé étrangère ne s'ajoute ni ne se retire sur une table existante.** SQLite demande de reconstruire la table, ce que Runique ne fait pas encore. Une migration qui ajoute ou retire un `belongs_to` sur une table déjà créée est donc refusée à `runique migration up`, avant d'avoir modifié quoi que ce soit, avec un message qui nomme la table. Elle s'applique normalement sous PostgreSQL et MySQL. Une clé déclarée à la création de la table n'est pas concernée.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL, MySQL |
| [CLI Runique](/docs/fr/installation/cli) | Commandes disponibles |
| [Modèles](/docs/fr/model) | DSL `model!{}` |
| [Fonctionnement de makemigrations](/docs/fr/architecture/makemigrations) | Snapshots, diff, génération |

## Retour au sommaire

- [Installation](/docs/fr/installation)
