# Fonctionnement interne de Makemigrations

La commande `runique makemigrations` fait le pont entre vos entités Rust (`model!{}`) et le schéma de base de données. Contrairement aux générateurs ORM classiques, elle est conçue pour préserver l'intention architecturale et les extensions propres au framework.

---

## Le pipeline de génération

Le processus de génération suit une architecture en trois phases :

### Phase 1 : Extraction de l'AST (`parse_schema_from_source`)

Runique utilise un parseur léger maison (basé sur `syn` et des expressions régulières pour la performance) pour lire vos fichiers `src/entities/*.rs`.

- **Analyse statique** : il ne compile pas votre code. Il lit directement les fichiers source pour extraire la structure des blocs `model!{}`.
- **Normalisation** : il convertit les types DSL de haut niveau (ex. `datetime`, `uuid`) en structures internes `FieldDef`.
- **Intelligence** : c'est ici qu'a lieu le **mapping automatique des champs** (association de noms de champs comme `email` à des comportements de formulaire spécialisés).

### Phase 2 : Diff et snapshots

Runique maintient un état caché dans `migration/src/snapshots/`.

- **État courant** : le parseur construit un schéma virtuel de votre code actuel.
- **État précédent** : il charge le dernier snapshot depuis le système de fichiers.
- **Moteur de diff** : il compare les deux états pour détecter :
    - Nouvelles tables / Tables supprimées.
    - Colonnes ajoutées / Colonnes supprimées.
    - **Renommage de colonne** : via l'indice explicite `[renamed_from: "ancien"]`, le diff émet un `RENAME COLUMN` au lieu d'un `DROP` + `ADD` (zéro perte de données). Sans cet indice, l'outil non interactif ne peut pas deviner l'intention.
    - Contraintes modifiées (ex. passage de `nullable` à `required`).
    - **Valeurs d'enum** : ajout, suppression et renommage (par position). Un renommage est traité comme **une seule** opération, exclu des listes ajout/suppression.

### Phase 3 : Génération SeaQuery

Le diff est converti en une séquence de requêtes `SeaQuery` (`TableCreate`, `TableAlter`).

1. **Ordonnancement** : il garantit que les dépendances (clés étrangères) sont traitées dans le bon ordre (tri topologique des nouvelles tables).
2. **Tables framework** : il injecte automatiquement les migrations `eihwaz_users` et `eihwaz_groupes` si elles sont absentes ou doivent être étendues via `extend!{}`.
3. **Sortie en code Rust** : il écrit un nouveau fichier `.rs` dans `migration/src/` et met à jour le trait `Migrator`.

### Génération spécifique au moteur

Le moteur cible est détecté (`DB_URL`/`DATABASE_URL`/`DB_ENGINE`) et la sortie est adaptée :

- **Clés étrangères** : regroupées dans une migration `create_relations` (`ALTER … ADD CONSTRAINT`) sur PostgreSQL/MySQL/MariaDB ; déclarées **inline dans le `CREATE TABLE`** sur SQLite (qui ne sait pas ajouter de FK à une table existante).
- **Enums** : `CREATE TYPE … AS ENUM` sur PostgreSQL ; `VARCHAR`/`ENUM` natif ailleurs. Un renommage de valeur d'enum devient `ALTER TYPE … RENAME VALUE` sur PostgreSQL (atomique) et un simple `UPDATE` des données sur les autres moteurs.
- **`updated_at`** : trigger PostgreSQL ; `ON UPDATE CURRENT_TIMESTAMP` sur MySQL/MariaDB.

Les fichiers générés sont donc **spécifiques au moteur** : pour en changer, régénérez à partir de zéro avec le bon `DB_ENGINE`.

---

## Commit atomique & garde destructif

Les phases ci-dessus ne font que *calculer* un plan en mémoire — rien n'est écrit tant que le plan complet (changements `model!{}` plus changements `extend!{}`) n'est pas assemblé et validé :

1. **Garde destructif** : `DROP COLUMN`, changements de type de colonne, `nullable → not null`, suppression de clés étrangères et ajout de contraintes `ON DELETE CASCADE` sont bloqués sauf si `makemigrations --force` est passé. Le contrôle couvre aussi bien les changements `model!{}` que `extend!{}`.
2. **Commit unique** : la création des dossiers, l'écriture des fichiers, l'enregistrement dans `lib.rs` et le positionnement de `AdminTableMigration` s'exécutent sous un rollback unique. En cas d'erreur d'écriture, les fichiers générés sont supprimés et les snapshots ainsi que `lib.rs` préexistants sont restaurés dans leur état précédent.

---

## Pourquoi des snapshots maison ?

Runique ne s'appuie pas uniquement sur l'état de la base de données (qui peut être désynchronisé). En conservant des snapshots de l'**état DSL**, le framework garantit que vos formulaires Admin correspondent toujours à vos déclarations de modèles, même si vous n'avez pas encore appliqué les migrations.

### Logique `extend!{}`

Quand vous utilisez `extend! { table: "eihwaz_users", ... }`, le parseur :
1. Identifie la table framework ciblée.
2. Stocke l'extension dans un dossier de snapshot dédié.
3. Génère un `ALTER TABLE` au lieu d'un `CREATE TABLE` lors du prochain `makemigrations`.

---

← [**Architecture**](/docs/fr/architecture) | [**Modèles**](/docs/fr/model) →
