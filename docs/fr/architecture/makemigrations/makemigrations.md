# Fonctionnement interne de Makemigrations

La commande `runique makemigrations` fait le pont entre vos entités Rust (`model!{}`, `extend!{}`) et le schéma de base de données. Elle compare vos déclarations au dernier état enregistré et écrit les migrations SeaORM qui font passer la base de l'un à l'autre.

---

## Le pipeline de génération

### Phase 1 : lecture du DSL

`makemigrations` lit les fichiers `src/entities/*.rs` sans les compiler, avec **le même parseur que la macro `model!{}`** : la crate `runique_dsl`. Un modèle refusé par la macro est refusé par la CLI, au même endroit et avec le même message :

- une erreur (type ou attribut inconnu, action de clé étrangère inconnue, règle de nullabilité non respectée…) arrête la commande avec le **fichier, la ligne et la colonne** ;
- un fichier qui n'est pas du Rust valide, ou qui déclare deux `model!{}`, est une erreur — jamais un fichier ignoré en silence ;
- **seuls les attributs décident** : le nom d'une colonne (`created_at`, `cache_key`…) n'a aucun effet.

Les cibles de `belongs_to` sont ensuite résolues comme le fait la macro : par le nom du module (le fichier `cible.rs`) ou d'une table du framework. La clé étrangère vise la vraie table et la vraie clé primaire de la cible ; une cible introuvable est une erreur. Chaque colonne de `belongs_to` reçoit un index `idx_<table>_<colonne>` (sauf si elle est déjà `unique` ou en tête d'un index de `meta`), et chaque champ liste (`multichoice` / `checkbox`) sa table `{table}_{champ}`.

### Phase 2 : diff et snapshots

Runique conserve dans `migration/src/snapshots/` l'état de chaque table **telle qu'elle a été appliquée**, sous forme d'un fichier de migration relu à chaque passage. Ce fichier est écrit par `runique migration up`, une fois la migration appliquée ; `makemigrations` ne l'écrit jamais. Le diff détecte :

- les tables ajoutées, et les colonnes ajoutées ou supprimées ;
- les **tables à supprimer** : une table que Runique a migrée (elle a son snapshot) et qu'aucun modèle ne produit plus (voir « Supprimer une table » plus bas) ;
- les **renommages de colonne** via `[renamed_from: "ancien"]` : un `RENAME COLUMN` au lieu d'un `DROP` + `ADD` (sans cet indice, l'outil non interactif ne peut pas deviner l'intention) ;
- les changements de type, de nullabilité, d'unicité, de valeur par défaut et de **longueur** (`max_length`) ;
- les valeurs d'enum ajoutées, supprimées ou renommées (un renommage est **une seule** opération) ;
- les clés étrangères et index ajoutés ou supprimés.

Un snapshot écrit avant la 3.0 ne contient pas les longueurs de colonne : il reprend une seule fois celles du modèle, sans produire de migration, puis il est réécrit avec elles.

### Phase 3 : génération

Le diff devient des instructions SeaQuery (`Table::create()`, `Table::alter()`, `Index::create()`…) écrites dans de nouveaux fichiers `migration/src/m<horodatage>_*.rs`, enregistrés dans le `Migrator` de `lib.rs`.

1. **Ordre** : les nouvelles tables sont triées pour qu'une table référencée soit créée avant celles qui la référencent ; l'ordre est déterministe.
2. **Clés étrangères** : déclarées dans le `CREATE TABLE`. Pour un **cycle** de tables nouvelles qui se référencent l'une l'autre, la clé qui ferme le cycle reste dans le `CREATE TABLE` sous SQLite (qui ne vérifie une clé qu'à l'écriture des lignes), et est ajoutée par un `ALTER TABLE` sous Postgres et MySQL, une fois la table visée créée.
3. **Tables du framework** : `eihwaz_users`, les sessions, les tables de l'admin et les jetons de réinitialisation ont leurs migrations dans Runique ; `makemigrations` les place en tête de `lib.rs`. Une table `eihwaz_*` déclarée dans `src/entities/` n'est jamais recréée — on l'étend avec `extend!{}`.

### Un fichier, tous les moteurs

Un fichier de migration est écrit une fois et doit pouvoir s'appliquer sur n'importe quel moteur. Ce qui ne concerne qu'un moteur est donc **choisi à l'exécution**, dans le fichier, plutôt qu'à la génération :

- **Enums** : `CREATE TYPE … AS ENUM` sous Postgres uniquement ; `VARCHAR` ailleurs. Un renommage de valeur devient `ALTER TYPE … RENAME VALUE` sous Postgres et un `UPDATE` des données ailleurs.
- **Clés d'un cycle** : voir plus haut.
- **`auto_now` / `auto_now_update`** : aucun trigger. L'entité remplit ces colonnes elle-même (`ActiveModelBehavior::before_save`), de la même façon sur tous les moteurs ; la migration ajoute seulement `DEFAULT CURRENT_TIMESTAMP`.
- **Modification de colonne sous SQLite** : SQLite ne sait pas modifier une colonne existante ; le fichier le signale par un commentaire et applique la modification sur les autres moteurs.

---

## Commit atomique & garde destructif

Les phases ci-dessus ne font que *calculer* un plan en mémoire — rien n'est écrit tant que le plan complet (changements `model!{}` plus changements `extend!{}`) n'est pas assemblé et validé :

1. **Garde destructif** : sont bloqués sauf si `makemigrations --force` est passé :
    - `DROP COLUMN` ;
    - changement de type de colonne ;
    - passage `nullable → not null` ;
    - **réduction de longueur** (`max_length` plus petit, ou longueur ajoutée à une colonne qui n'en avait pas) ;
    - suppression de clé étrangère ;
    - ajout d'une clé `ON DELETE CASCADE` sur une table existante.

   Le contrôle couvre les changements `model!{}` comme `extend!{}`.

   > **Exception — changement de type de colonne** : `--force` débloque l'exécution de la commande, mais ne génère **jamais** l'`ALTER` réel pour un changement de type (`String → Decimal`, etc.). Le fichier généré contient uniquement un commentaire `// Manual migration required.` — à écrire vous-même. La console le signale en jaune après la génération, colonne par colonne. Les autres catégories destructives génèrent le vrai SQL dès que `--force` est passé. Cette exception est volontaire : une conversion de type générique n'a pas de règle de cast fiable inter-moteurs (Postgres exige un `USING` explicite, MySQL convertit selon son mode SQL, SQLite ne supporte pas `ALTER COLUMN TYPE` du tout).
2. **Écriture protégée** : les migrations, `lib.rs` et les dossiers créés pour eux sont écrits en un seul lot, avec la suppression des migrations en attente remplacées. En cas d'erreur d'écriture, tout est annulé : les fichiers générés sont retirés, les migrations remplacées et `lib.rs` reviennent à leur état précédent. Un arrêt brutal (coupure, `kill -9`) est réparé au lancement suivant, à partir du journal laissé dans `migration/src/`. Voir [Écriture atomique des fichiers](/docs/fr/architecture/atomic).

---

## Pourquoi des snapshots maison ?

`makemigrations` compare vos modèles au **dernier état qu'il a lui-même généré**, enregistré dans les snapshots. Le résultat est donc le même sur toutes les machines de l'équipe, et ne dépend pas d'une base qui aurait pu être modifiée à la main.

Depuis la 4.1, la commande **se connecte d'abord à la base** (celle du `.env`, comme l'application) et s'arrête avec une erreur si elle ne répond pas, avant d'avoir lu ou écrit quoi que ce soit. Elle n'y écrit rien : seul `runique migration up` applique une migration.

Elle y lit la liste des migrations appliquées (`seaql_migrations`) : si l'une d'elles est inconnue de `lib.rs`, la commande s'arrête avant tout calcul. Une base qui n'a encore rien appliqué (table absente) est un cas normal.

## Supprimer une table

Retirer un modèle supprime sa table. La règle : une table a un snapshot dans `migration/src/snapshots/` (Runique l'a donc migrée) et plus aucun modèle ne la produit.

```
$ runique makemigrations
Changements destructifs détectés :
  menus: DROP TABLE (data loss)
Utilisez --force pour appliquer ces changements destructifs.
```

Sans `--force`, rien n'est écrit, pas même les autres changements du lancement. Avec `--force`, `makemigrations` écrit une migration `m…_drop_menus_table.rs` :

- `up` fait un `DROP TABLE IF EXISTS` : une table déjà absente ne fait pas échouer `runique migration up` ;
- `down` recrée la table d'après son snapshot. Elle revient **vide** : les lignes sont perdues dès que la suppression est appliquée.

Le snapshot reste tant que la suppression n'est pas appliquée ; `runique migration up` le retire ensuite. Entre-temps, relancer `makemigrations` ne réécrit rien, et si vous remettez le modèle, la migration de suppression en attente disparaît d'elle-même.

**Ce qui compte comme un modèle retiré.** Le lecteur ne voit que les fichiers `.rs` du premier niveau de `src/entities/`, hors `mod.rs`. Un modèle mis en commentaire, déplacé dans un sous-dossier ou dans `mod.rs`, ou dont le fichier change d'extension, est absent pour lui : sa table est proposée à la suppression. Le blocage sans `--force` est le seul filet : lisez la liste avant de forcer.

**Ce qui est supprimé aussi.** La table d'un champ liste (`checkbox`, `multichoice`) part avec son modèle, ou seule si le champ est retiré. Un type enum PostgreSQL de la table n'est supprimé que si plus aucune colonne ne s'en sert : un enum partagé par deux tables reste jusqu'à la dernière.

**Ce qui n'est jamais supprimé.** Les tables du framework (`eihwaz_*`), et toute table de la base que Runique n'a pas migrée lui-même : sans snapshot, elle est ignorée.

**L'ordre.** Les suppressions ferment le lancement. Entre elles, la table qui en vise une autre part la première : la table d'un champ liste avant celle de son modèle, par exemple. PostgreSQL et MySQL refusent de supprimer une table tant qu'une clé étrangère pointe dessus, même si les deux tables sont vides.

**Quand une table qui reste pointait sur la table supprimée.**

- Vous avez gardé son `belongs_to` : la commande s'arrête en erreur, avec ou sans `--force`. Mettre vos modèles en accord est votre travail, et le projet ne compilerait pas de toute façon.
- Vous avez retiré son `belongs_to` : sa migration retire la clé, puis la table visée est supprimée. Sous SQLite, ce retrait de clé n'est pas encore pris en charge : la migration est refusée avant d'avoir rien modifié (voir [les limites](/docs/fr/installation/migrations)).

> Renommer la table d'un modèle (`table: "…"`) sans autre précaution revient à supprimer l'ancienne et créer la nouvelle : les lignes sont perdues. Les deux sont bloqués sans `--force`.

## Relancer avant d'appliquer

Tant que `runique migration up` n'est pas passé, les migrations écrites sont **en attente** (elles portent la ligne `// runique: waiting migrate up`) et leurs tables n'ont pas de snapshot. Relancer `makemigrations` repart donc des mêmes snapshots et recalcule tout l'écart jusqu'aux modèles :

- **rien n'a changé** : le calcul donne les mêmes migrations, aucun fichier n'est réécrit ;
- **un modèle a changé** : les migrations en attente sont supprimées et remplacées par le nouveau calcul. Elles ne s'empilent pas : une table créée puis modifiée avant application donne une seule création, complète ;
- **le changement a été repris** (les modèles correspondent de nouveau aux snapshots) : les migrations en attente sont supprimées, avec leur ligne dans `lib.rs` ;
- **le lancement est refusé** (changement destructif sans `--force`) : rien n'est écrit, ce qui attendait reste tel quel.

Une migration déjà appliquée n'est jamais remplacée.

> Une migration en attente retouchée à la main est écrasée au lancement suivant. Si une migration doit être écrite à la main, appliquez-la avant de relancer `makemigrations`.

## Deux migrations écrites en parallèle

Sous le tag, chaque migration générée porte une seconde ligne qui dit d'où elle part pour sa table :

```rust
// runique: waiting migrate up
// runique: from m20261120_101500_create_menus_table
```

C'est la dernière migration déjà en place qui touche cette table, ou `none` pour la première. `runique migration up` retire le tag et laisse cette ligne.

Avant tout calcul, `makemigrations` refuse de continuer si deux migrations de `lib.rs` partent du même point pour la même table : elles ont été écrites chacune de leur côté (deux développeurs sur la même table, puis leurs travaux réunis), et Runique ne peut pas savoir laquelle la table doit suivre. Le message nomme la table et les deux migrations ; rien n'est écrit. Il n'y a pas de commande pour les réunir : c'est au développeur de n'en garder qu'une.

Les migrations écrites avant la 4.1 n'ont pas cette ligne et ne sont pas concernées par ce contrôle.

La contrepartie : un snapshot signifie « déjà migré ». Des snapshots copiés d'un autre projet sans leurs fichiers de migration font croire à `makemigrations` que les tables existent ; il ne génère alors rien pour elles.

### Logique `extend!{}`

Quand vous utilisez `extend! { table: "eihwaz_users", ... }`, `makemigrations` :
1. Identifie la table framework ciblée.
2. Stocke l'extension dans un dossier de snapshot dédié.
3. Génère un `ALTER TABLE` au lieu d'un `CREATE TABLE`.

---

## Exemples concrets

### Renommer une colonne sans perte de données

Renommer un champ directement produit un `DROP` + `ADD` → données perdues. L'indice `renamed_from` signale l'intention à l'outil non interactif :

```rust
model! {
    Employe,
    table: "employes",
    pk: id => i32,
    {
        // avant :  job_title: text,
        title: text [renamed_from: "job_title"],
    }
}
```

`makemigrations` émet alors `ALTER TABLE employes RENAME COLUMN job_title TO title` (PostgreSQL, MySQL, SQLite). L'attribut est une directive de migration uniquement : aucun effet sur l'entité ou le formulaire générés. Garde-fou : si l'ancienne colonne existe encore dans le snapshot (hint périmé), aucun rename n'est émis.

### Étendre une table framework avec `extend!{}`

Pour ajouter des colonnes à `eihwaz_users` (ou `eihwaz_groupes`) sans toucher au framework :

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio:         textarea [nullable],
        avatar:      image    [nullable, upload_to: "avatars/"],
        website:     url      [nullable],
        is_verified: bool     [default: false],
    }
}
```

Au prochain `makemigrations`, ces champs deviennent un `ALTER TABLE eihwaz_users ADD COLUMN …` (jamais un `CREATE TABLE`). Les champs d'`extend!{}` suivent les mêmes types, options et règles de nullabilité que `model!{}`, `renamed_from` compris.

### Générer, appliquer, annuler

```bash
# Détecte le diff et écrit les fichiers de migration
runique makemigrations

# Les changements destructifs sont bloqués par défaut. Pour les autoriser :
runique makemigrations --force
# NB : pour un changement de type de colonne, --force débloque juste
# l'exécution — le fichier généré reste un commentaire "Manual migration
# required.", jamais un vrai ALTER (voir l'encart plus haut).

# Chemins personnalisés (défauts : src/entities et migration/src)
runique makemigrations --entities src/entities --migrations migration/src

# Appliquer les migrations générées
runique migration up

# Annuler les N dernières / voir l'état
runique migration down -n 1
runique migration status
```

---

← [**Architecture**](/docs/fr/architecture) | [**Modèles**](/docs/fr/model) →
