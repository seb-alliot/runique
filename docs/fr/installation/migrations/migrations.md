# Migrations (SeaORM)

## Workflow en deux étapes

### 1. Générer les fichiers de migration

`runique makemigrations` lit vos entités déclarées dans `src/entities` et génère les fichiers de migration correspondants :

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

**Ce que `makemigrations` fait en une seule commande :**

| Passe | Source | Résultat |
| --- | --- | --- |
| 1 — Tables app | Blocs `model!{}` dans `src/entities` | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Extensions framework | Blocs `extend!{}` dans `src/entities` | `ALTER TABLE ADD/DROP COLUMN` |
| 3 — Positionnement admin | `RUNIQUE_USER_TABLE` dans `.env` | Ordre garanti dans `lib.rs` |

### 2. Appliquer les migrations

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

---

## Changements destructifs — `--force`

Avant d'écrire quoi que ce soit, `makemigrations` détecte les changements destructifs et **refuse** de générer la migration, sauf si vous passez `--force` :

- `DROP COLUMN` (perte de données)
- changement de type d'une colonne
- passage `nullable` → `not null` (nécessite une valeur par défaut ou un backfill)
- suppression d'une clé étrangère (risque d'enregistrements orphelins)
- ajout d'une FK `ON DELETE CASCADE` sur des données existantes (suppressions en cascade possibles)

```bash
runique makemigrations --force
```

> Le contrôle couvre aussi bien les tables d'application (`model!{}`) que les extensions de tables framework (`extend!{}`).

---

## Génération atomique

La génération est **tout ou rien**. Les changements des modèles et des blocs `extend!{}` sont d'abord planifiés en mémoire, validés (contrôle destructif ci-dessus), puis écrits en un seul lot. En cas d'erreur d'écriture, tous les fichiers générés sont annulés et les snapshots ainsi que `lib.rs` sont restaurés dans leur état initial.

---

## Clés étrangères et moteur cible

`makemigrations` détecte le moteur (via `DB_URL`/`DATABASE_URL`/`DB_ENGINE`) et adapte la génération des FK :

- **PostgreSQL / MySQL / MariaDB** : toutes les contraintes FK sont regroupées dans une migration `…_create_relations` appliquée **après** la création des tables (`ALTER TABLE … ADD CONSTRAINT`).
- **SQLite** : ne supporte pas l'ajout de FK à une table existante. Les FK sont donc déclarées **inline dans le `CREATE TABLE`**, et la migration `create_relations` n'est pas générée.

Conséquence : les fichiers de migration sont **spécifiques au moteur** pour lequel ils ont été générés. Pour changer de moteur, régénérez les migrations (à partir de zéro) avec le `DB_ENGINE` cible.

---

## Tables framework — fournies automatiquement

Runique injecte automatiquement deux migrations dans votre `lib.rs` sans que vous ayez à les définir :

| Migration | Crée | Ordre |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_staff, is_superuser, created_at, updated_at) | 1er |
| `EihwazSessionsMigration` | `eihwaz_sessions` (persistance des sessions authentifiées) | 2e |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes`, `eihwaz_history` | 3e |

> Vous n'avez pas besoin de déclarer `eihwaz_users` dans vos entités.

---

## Étendre une table framework — `extend!{}`

Pour ajouter des colonnes à une table framework, utilisez `extend!{}` dans un fichier d'entité :

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        avatar: image [upload_to: "avatars/"],
        bio: textarea,
        website: url [required],
    }
}
```

`makemigrations` détecte ces blocs et génère les `ALTER TABLE ADD COLUMN` correspondants.
Le snapshot d'extension est stocké dans `migration/src/snapshots/runique/eihwaz_users.rs`.

**Tables extensibles :**

```text
eihwaz_users · eihwaz_groupes · eihwaz_droits
eihwaz_sessions · eihwaz_users_groupes · eihwaz_groupes_droits
```

> Étendre une table inconnue déclenche une erreur à la **compilation**.

---

## Utiliser une table user personnalisée

Si vous préférez gérer votre propre table utilisateur, déclarez-la dans `.env` :

```env
RUNIQUE_USER_TABLE=ma_table_users
```

`makemigrations` positionnera alors `AdminTableMigration` juste après la migration de `ma_table_users`.
La FK dans `eihwaz_users_groupes` ciblera automatiquement `ma_table_users`.

> Par défaut (`RUNIQUE_USER_TABLE` absent) : `eihwaz_users` est utilisée.

---

## Autres commandes de migration

```bash
sea-orm-cli migrate down --migration-dir migration/src   # Annuler la dernière migration
sea-orm-cli migrate status --migration-dir migration/src # Voir l'état des migrations
```

---

## Wrapper Runique — rollback atomique (avancé)

```bash
runique migration down --migrations migration/src <fichier>
runique migration down --migrations migration/src --batch <timestamp>
runique migration status --migrations migration/src
```

> Ces commandes utilisent le système de batch Runique avec rollback transactionnel.
> Préférer `sea-orm-cli` pour le workflow normal.

---

> `runique makemigrations` est le seul outil à utiliser pour **générer** les fichiers de migration.
> Ne pas utiliser `sea-orm-cli migrate generate` : Runique maintient ses propres snapshots et ordre chronologique.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Base de données](/docs/fr/installation/base-de-donnees) | SQLite, PostgreSQL, MariaDB |
| [CLI Runique](/docs/fr/installation/cli) | Commandes disponibles |
| [Modèles](/docs/fr/model) | DSL `model!{}` |

## Retour au sommaire

- [Installation](/docs/fr/installation)
