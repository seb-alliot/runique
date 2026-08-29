# DSL `model!` & `extend!`

## Macros exposées

- `model! { ... }` — déclare un modèle (entité SeaORM + migrations + formulaire admin)
- `extend! { ... }` — ajoute des colonnes à une table framework existante
- `#[form(...)]` — lie un formulaire Rust à un `model!` (voir [Formulaires & enjeux](/docs/fr/model/formulaires))

Toutes sont disponibles via `use runique::prelude::*`.

---

## Structure du DSL `model!`

Le parseur attend les blocs **dans cet ordre strict** (les blocs optionnels peuvent être absents mais pas réordonnés) :

```rust
model! {
    NomModele,              // 1. Nom (PascalCase)
    table: "nom_table",     // 2. Nom de la table SQL
    pk: champ => type,      // 3. Clé primaire
    enums: { ... },         // 4. Optionnel — enums locaux
    { ... },                // 5. Champs — bloc anonyme, types sémantiques
    relations: { ... },     // 6. Optionnel — relations SeaORM
    meta: { ... },          // 7. Optionnel — contraintes & tri
}
```

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        titre:      text      [required, max_length: 150],
        contenu:    textarea  [required],
        is_active:  bool      [default: true],
        created_at: datetime  [auto_now],
    }
}
```

> Le bloc de champs est **toujours** un bloc anonyme `{ ... }`, jamais précédé du mot-clé
> `fields:`. Ne pas confondre avec `extend!{}`, qui lui exige `fields:` — deux macros
> différentes, deux grammaires différentes (voir plus bas).

---

## Clé primaire (`pk`)

```
pk: nom_champ => type
```

| Type   | SQL Postgres          | SQL MySQL               | Auto-incrément | Création                        |
|--------|-----------------------|--------------------------|----------------|----------------------------------|
| `i32`  | `SERIAL`              | `INT AUTO_INCREMENT`     | ✅ Oui          | séquence DB                      |
| `i64`  | `BIGSERIAL`           | `BIGINT AUTO_INCREMENT`  | ✅ Oui          | séquence DB                      |
| `uuid` | `UUID`                | `VARCHAR(36)`            | ❌ Non          | `Uuid::now_v7()` côté Rust        |
| `Pk`   | alias `i32`/`i64`/`Uuid` | idem                   | selon le type   | selon la feature active          |

**L'alias `Pk`** défère au type global de l'application, résolu par feature Cargo — **une seule**
peut être active à la fois (`compile_error!` si deux sont déclarées ensemble) :

```toml
[dependencies]
# rien de déclaré → Pk = i32 (défaut)
runique = { version = "2.2.0", features = ["big-pk"] }    # Pk = i64
runique = { version = "2.2.0", features = ["pk-uuid"] }   # Pk = Uuid (généré via Uuid::now_v7())
```

Utilisez `big-pk` quand vous anticipez plus de ~2 milliards de lignes dans une table, ou pour interopérer avec un schéma existant utilisant des clés primaires `BIGINT`. Utilisez `pk-uuid` pour des identifiants non séquentiels (multi-tenant, génération côté client, exposition publique des ids sans révéler le volume de lignes).

### `Pk` sur un champ normal (pas seulement la PK)

Le mot-clé `Pk` est aussi utilisable sur un **champ FK ordinaire**, pas uniquement dans
`pk: id => Pk`. C'est la façon recommandée de déclarer une colonne qui référence la clé
primaire d'une autre table : elle suit alors automatiquement la même feature, sans jamais se
désynchroniser si vous changez `big-pk`/`pk-uuid` plus tard.

```rust
model! {
    Chapitre,
    table: "chapitre",
    pk: id => Pk,
    {
        cour_id: Pk [required],   // suit automatiquement le type de Cours.id
        titre:   text [required],
    },
    relations: {
        belongs_to: Cour via cour_id,
    }
}
```

**À éviter** : déclarer une FK avec un type figé (`cour_id: int [required]`) quand la table
référencée utilise `pk: id => Pk`. Ça compile et fonctionne tant que `Pk` vaut `i32`, mais se
désynchronise silencieusement dès qu'une feature (`big-pk`/`pk-uuid`) change — le même type de
bug que documenté plus bas pour `big-pk`. Utiliser `Pk` sur le champ FK élimine le risque à la
source, sans code de conversion (`.try_into()`) à maintenir.

**Contrainte lors de l'activation de `big-pk`/`pk-uuid`** : chaque colonne FK pointant vers une
clé primaire `Pk` doit rester cohérente avec elle. La forme `cour_id: Pk` (ci-dessus) le garantit
automatiquement ; une colonne figée en `bigint`/`int`/`uuid` doit être mise à jour manuellement
si vous changez de feature après coup.

> **Le choix de `big-pk`/`pk-uuid` doit être fait avant la première migration.**
> Une fois les migrations appliquées, basculer de mode est un changement cassant : les colonnes
> en base ont déjà un type concret, et changer la feature ne modifie que le type Rust — le
> schéma reste intact. Changer après coup nécessite une migration manuelle pour `ALTER` chaque
> colonne PK et FK, avec un risque de troncature (`big-pk` → défaut) ou d'incompatibilité totale
> de format (`pk-uuid` ↔ n'importe quel type entier). Choisissez un mode au démarrage du projet.

---

## Types de champs

| Type DSL          | Type Rust généré          | Colonne SQL                    |
|--------------------|---------------------------|---------------------------------|
| `text`             | `String`                  | `VARCHAR(255)` ou `VARCHAR(n)` si `max_length: n` |
| `char`             | `String`                  | `CHAR`                          |
| `email`            | `String`                  | `VARCHAR(254)` — format validé  |
| `password`         | `String`                  | `VARCHAR(255)` — haché automatiquement |
| `richtext`         | `String`                  | `TEXT` — éditeur HTML           |
| `textarea`         | `String`                  | `TEXT` — multi-ligne            |
| `url`              | `String`                  | `VARCHAR(255)` — format validé  |
| `slug`             | `String`                  | `VARCHAR(255)`                  |
| `color`            | `String`                  | `VARCHAR(255)` — couleur hex    |
| `phone`            | `String`                  | `VARCHAR(20)` ou `VARCHAR(n)` si `max_length: n` |
| `i8`               | `i8`                      | `TINYINT`                       |
| `i16`              | `i16`                     | `SMALLINT`                      |
| `int`              | `i32`                     | `INTEGER`                       |
| `bigint`           | `i64`                     | `BIGINT`                        |
| `u32`              | `u32`                     | `INTEGER UNSIGNED`              |
| `u64`              | `u64`                     | `BIGINT UNSIGNED`               |
| `f32`              | `f32`                     | `FLOAT`                         |
| `float`            | `f64`                     | `DOUBLE`                        |
| `percent`          | `f64`                     | `DOUBLE` — stocké comme float   |
| `decimal`          | `Decimal`                 | `DECIMAL`                       |
| `bool`             | `bool`                    | `BOOLEAN`                       |
| `date`             | `NaiveDate`                | `DATE`                          |
| `time`             | `NaiveTime`                | `TIME`                          |
| `datetime`         | `NaiveDateTime`            | `DATETIME`                      |
| `timestamp`        | `NaiveDateTime`            | `TIMESTAMP`                     |
| `timestamp_tz`     | `DateTime<Utc>`            | `TIMESTAMPTZ`                   |
| `uuid`             | `Uuid`                     | `UUID`                          |
| `Pk`               | `i32`/`i64`/`Uuid`         | selon la feature — voir ci-dessus |
| `json`             | `serde_json::Value`        | `JSON`                          |
| `json_binary`      | `serde_json::Value`        | `JSON BINARY`                   |
| `binary`           | `Vec<u8>`                  | `BINARY` — taille via `max_length: n` |
| `var_binary`       | `Vec<u8>`                  | `VARBINARY(n)` — `max_length: n` requis |
| `blob`             | `Vec<u8>`                  | `BLOB`                          |
| `ip`               | `String`                   | `INET`                          |
| `cidr`             | `String`                   | `CIDR`                          |
| `mac_address`      | `String`                   | `MACADDR`                       |
| `interval`         | `String`                   | `INTERVAL`                      |
| `image`            | `String`                   | `VARCHAR(255)` — chemin de fichier |
| `document`         | `String`                   | `VARCHAR(255)` — chemin de fichier |
| `file`             | `String`                   | `VARCHAR(255)` — chemin de fichier |
| `choice`           | `EnumName`                 | `VARCHAR` / `ENUM` natif — requiert `enum(NomEnum)` |
| `radio`            | `EnumName`                 | Idem `choice`, widget différent |
| `checkbox`         | `EnumName`                 | Idem `choice`, widget différent |

`binary`/`var_binary` réutilisent l'option `max_length: n` (même mécanisme que `text` +
`max_length` → `VARCHAR(n)`) — il n'existe pas de syntaxe `binary(n)` inline séparée.

> **Non disponible** : `decimal(precision, scale)` inline (ex. `decimal(10, 2)`) n'a pas
> d'équivalent actuel — seul `decimal` sans paramètres est supporté. Contournement : appliquer
> la précision/l'échelle côté validation applicative plutôt que dans le schéma.

---

## Options de champ

Dans un bloc `[...]`, séparées par des virgules, valeur après `:` quand l'option en prend une :

```rust
username: text [required, max_length: 150, unique],
```

| Option                   | Description                                                     |
|--------------------------|-------------------------------------------------------------------|
| `required`               | Colonne `NOT NULL` + validation formulaire                       |
| `nullable`               | Colonne `NULL` — type Rust `Option<T>`                           |
| `unique`                 | Contrainte `UNIQUE`                                              |
| `max_length: n`          | Longueur max (validation + taille de colonne)                    |
| `min_length: n`          | Longueur min (validation)                                        |
| `min: n`                 | Valeur min entière (validation)                                  |
| `max: n`                 | Valeur max entière (validation)                                  |
| `min: n.0`               | Valeur min flottante (validation)                                |
| `max: n.0`               | Valeur max flottante (validation)                                |
| `default: valeur`        | Valeur par défaut SQL (`true`, `0`, `"draft"`, etc.)             |
| `auto_now`               | Assigné à `NOW()` à chaque `INSERT` — exclu des formulaires      |
| `auto_now_update`        | Assigné à `NOW()` à chaque `UPDATE` — exclu des formulaires      |
| `readonly`               | Exclu de la migration générée (colonne existe côté Rust, non gérée par `derive_form`) |
| `label: "str"`           | Libellé personnalisé dans les formulaires admin                  |
| `help: "str"`            | Réservé — pas encore branché au rendu                            |
| `upload_to: "path"`      | Champ fichier — dossier d'upload                                 |
| `max_size: n MB`         | Champ fichier — taille max (`KB`/`MB`/`GB`)                      |
| `rows: n`                | `textarea`/`richtext` — hauteur du widget                        |
| `step: n`                | Champs numériques — pas du widget                                |
| `fk(table.col, action)`  | Contrainte clé étrangère (voir Relations)                        |
| `enum(NomEnum)`          | Lie le champ à un enum déclaré dans `enums:`                     |
| `renamed_from: "x"`      | Renomme la colonne (voir plus bas)                                |
| `skip`                   | Exclu des formulaires générés                                    |
| `no_hash`                | Champs `password` uniquement — désactive le hachage automatique  |

> **`readonly`** (nouveau, DB-level) est distinct du `readonly` de `#[form]`
> (`field_readonly()`, désactive un champ dans le rendu HTML d'une instance de formulaire
> précise). `readonly` sur le champ du modèle exclut la colonne de la migration générée ;
> `field_readonly()` désactive juste un widget au runtime. Les deux peuvent coexister.

> **`auto_now` / `auto_now_update`** : ces champs sont exclus de `admin_from_form` et d'`admin_partial_update`. Leur valeur est gérée uniquement par la base. Ils apparaissent dans `Model` et `Column` comme `Option<T>`.

### Renommer une colonne — `renamed_from`

Renommer un champ sans cette option produit un `DROP COLUMN` + `ADD COLUMN` → **perte de données**.
L'outil étant non interactif, il ne peut pas deviner l'intention : il faut le signaler explicitement.

```rust
// avant :  job_title: text,
// après :
title: text [renamed_from: "job_title"],
```

`makemigrations` génère alors un `ALTER TABLE … RENAME COLUMN job_title TO title` (supporté par
PostgreSQL, MySQL/MariaDB et SQLite), sans perte de données. L'attribut est une directive de
migration uniquement : il n'a aucun effet sur l'entité ou le formulaire générés. Garde-fou : si
l'ancienne colonne existe toujours dans le snapshot (hint périmé), aucun rename n'est émis.

Fonctionne aussi bien dans `model!{}` que dans `extend!{}`.

---

## Enums

Les enums se déclarent dans un bloc `enums: { ... }` distinct des champs, puis sont référencés via `enum(NomEnum)`.

```rust
model! {
    Commande,
    table: "commandes",
    pk: id => i32,
    enums: {
        StatutCommande: [
            EnAttente  = ("en_attente",  "En attente"),
            EnCours    = ("en_cours",    "En cours"),
            Livree     = ("livree",      "Livrée"),
            Annulee    = ("annulee",     "Annulée"),
        ],
        Priorite: i32 [Basse = 0, Normale = 1, Haute = 2, Urgente = 9],
    },
    {
        statut:   choice [enum(StatutCommande), required],
        priorite: choice [enum(Priorite), required],
    },
}
```

### Quatre formes de variant — à ne jamais confondre

> **Attention, piège fréquent** : `:` et `=` ne font **pas** la même chose. Une seule
> différence de symbole change complètement le comportement, sans erreur de compilation pour
> vous avertir. Toujours vérifier sur ce tableau, ne pas deviner par analogie.

| Syntaxe                              | Valeur stockée en DB | Libellé affiché (`Display`)                  |
|---------------------------------------|----------------------|-----------------------------------------------|
| `Variant`                             | `"Variant"` (le nom)  | `"Variant"` — retombe sur la valeur DB         |
| `Variant: "Libellé"`                  | `"Variant"` (**inchangé**) | `"Libellé"`                               |
| `Variant = "valeur_db"`               | `"valeur_db"`         | `"valeur_db"` — retombe sur la valeur DB, **pas** sur `Variant` |
| `Variant = ("valeur_db", "Libellé")`  | `"valeur_db"`         | `"Libellé"`                                    |

Résumé de la règle :
- `:` (deux-points) ne touche **que l'affichage** — la valeur stockée reste toujours le nom du variant.
- `=` seul (sans parenthèses) ne touche **que la valeur stockée** — l'affichage retombe dessus, jamais sur le nom du variant.
- `= (a, b)` fixe les deux indépendamment — c'est la seule forme qui permet un nom de variant, une valeur DB et un libellé tous différents.

**Le libellé est purement cosmétique.** Il n'affecte :
- ni le stockage réel (`#[sea_orm(string_value = ...)]` / `#[sea_orm(num_value = ...)]` utilisent toujours la valeur DB, jamais le libellé),
- ni la comparaison de parsing (`FromStr` compare en priorité contre la valeur DB et le nom du variant Rust — le libellé n'est accepté qu'en repli, seulement s'il diffère des deux autres).

Changer un libellé (`Published: "Publié"` → `Published: "Mis en ligne"`) n'a donc **aucun** impact sur les données stockées ni sur le code qui compare des valeurs d'enum.

> **La valeur DB est stockée exactement telle qu'écrite.** Aucune transformation automatique.

### Types de backing

| Syntaxe              | Stockage DB                                     |
|----------------------|--------------------------------------------------|
| `NomEnum: [A, B]`     | `ENUM` natif (Postgres) ou `VARCHAR` (MySQL/SQLite) |
| `NomEnum: i32 [...]`  | `INTEGER` — `=` fixe alors la valeur numérique, pas une chaîne |
| `NomEnum: i64 [...]`  | `BIGINT` — idem                                   |

### Méthodes générées

| Méthode | Retour | Description |
|---------|--------|-------------|
| `.to_string()` | `String` | Libellé d'affichage |
| `.db_value()` | `&'static str` / `i32` / `i64` | Valeur exacte en base |
| `::from_str(s)` / `.parse()` | `Result<Self, ()>` | Parsing depuis valeur DB, libellé, ou nom variant |
| `::iter()` | `impl Iterator<Item = Self>` | Itération sur tous les variants |

```rust
use sea_orm::Iterable;

let s = StatutCommande::EnAttente;
s.db_value()   // → "en_attente"
s.to_string()  // → "En attente"

// Pour un <select>
let options: Vec<(String, String)> = StatutCommande::iter()
    .map(|v| (v.db_value().to_string(), v.to_string()))
    .collect();

// Parser depuis une valeur DB
let statut: Option<StatutCommande> = "en_attente".parse().ok();
```

**Dans les templates Tera**, la valeur de comparaison doit correspondre **exactement** à ce qui est stocké en base (sensible à la casse).

---

## Champs fichier

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        image:        image    [upload_to: "media/articles"],
        fichier:      document [upload_to: "docs/"],
        piece_jointe: file     [upload_to: "media/uploads"],
    },
}
```

| Type      | Extensions autorisées          |
|-----------|--------------------------------|
| `image`   | `jpg jpeg png gif webp avif`   |
| `document`| `pdf doc docx txt odt`         |
| `file`    | aucun filtre                   |

`upload_to:` est obligatoire pour les trois types. Le chemin est relatif à `MEDIA_ROOT`.

---

## Relations

```rust
relations: {
    belongs_to: Model via champ_fk,
    has_many: Model,
    has_many: Comments as user_comments,   // alias optionnel
    has_one: Profile as user_profile,
    many_to_many: Roles through UserRoles via self_id,
}
```

| Type             | Contrainte DB   | Description                  |
|------------------|-----------------|-------------------------------|
| `belongs_to`     | ❌ code seul     | Relation N-1 (SeaORM)        |
| `has_many`       | ❌ code seul     | Relation 1-N                 |
| `has_one`        | ❌ code seul     | Relation 1-1                 |
| `many_to_many`   | ❌ code seul     | Relation N-N via pivot       |

> **Contrainte FK réelle** : la contrainte SQL `FOREIGN KEY` et son action (`cascade`, `restrict`, `set_null`, `set_default`) sont déclarées sur l'option `fk(table.col, action)` du champ, pas dans le bloc `relations:`. Le bloc `relations:` génère uniquement les traits SeaORM pour la navigation objet.

Actions FK disponibles sur l'option `fk(...)` : `cascade` · `restrict` · `set_null` · `set_default`

---

## Meta

```rust
meta: {
    ordering: [-created_at, titre],
    unique_together: [(slug, lang)],
    indexes: [(lang, sort_order)],
    verbose_name: "Article",
    verbose_name_plural: "Articles",
}
```

| Clé                   | Syntaxe               | Effet                                       |
|-----------------------|-----------------------|---------------------------------------------|
| `ordering`            | `[champ, -champ]`     | Tri par défaut, `-` = `DESC`                |
| `unique_together`     | `[(col1, col2)]`      | Contrainte `UNIQUE` multi-colonnes          |
| `indexes`             | `[(col1, col2)]`      | Index simple multi-colonnes                 |
| `verbose_name`        | `"chaîne"`            | Nom singulier dans l'interface admin        |
| `verbose_name_plural` | `"chaîne"`            | Nom pluriel dans l'interface admin          |
| `abstract`            | `true`                | Modèle abstrait — aucune table générée      |

---

## `extend!{}` — extension des tables framework

Ajoute des colonnes à une table Runique et génère une entité SeaORM complète sur cette table.

`extend!{}` produit deux choses :

1. **Schema SQL** — `makemigrations` détecte le bloc et génère des instructions `ALTER TABLE ADD COLUMN`
2. **Entité complète** — `Model`, `Column`, `Entity`, `AdminForm`, `admin_from_form`, `admin_partial_update` couvrant **toutes** les colonnes de la table (colonnes de base + colonnes étendues)

```rust
// src/entities/user_profile.rs
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio:        textarea,
        avatar:     image  [upload_to: "avatars/"],
        website:    url,
        phone:      phone,
        birth_date: date,
        is_verified: bool  [default: false],
    }
}
```

> `extend!{}` exige **toujours** le mot-clé `fields:` avant le bloc de champs — contrairement à
> `model!{}` (bloc anonyme direct). Ce sont deux macros différentes, deux grammaires
> différentes ; ne pas transposer la syntaxe de l'une à l'autre.

Tables autorisées : `eihwaz_users`, `eihwaz_groupes`, `eihwaz_droits`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Tout autre nom provoque une erreur à la compilation.

Les champs déclarés dans `extend!{}` utilisent les mêmes types et options que `model!` (y compris `renamed_from`). Pas de bloc `relations:` dans `extend!{}` — les relations se déclarent dans `model!{}` cible avec `has_many(user_profile)` etc.

### Enums dans `extend!{}`

`extend!{}` accepte un bloc `enums: { ... }` optionnel (entre `table:` et `fields:`), identique à celui de `model!`. La colonne `choice [enum(NomEnum)]` génère le type Rust enum, la colonne typée et le `ChoiceField` peuplé :

```rust
extend! {
    table: "eihwaz_users",
    enums: {
        Seniority: [Junior="junior", Mid="mid", Senior="senior", Lead="lead"],
    },
    fields: {
        job_title: text,
        seniority: choice [enum(Seniority)],
    }
}
```

`makemigrations` émet la colonne (sur PostgreSQL, un `CREATE TYPE … AS ENUM` ; ailleurs, un `VARCHAR`/`ENUM` natif).

### Workflow complet

```bash
# 1. Déclarer l'extension dans src/entities/
# 2. Générer la migration
runique makemigrations

# 3. Appliquer
runique migrate

# 4. Enregistrer dans admin!{} (src/admin.rs)
```

```rust
admin! {
    configure {
        users: { hidden: true }   // masque le panel builtin "Utilisateurs"
    }
    user_profile: user_profile::Model => user_profile::AdminForm {
        title: "Profils utilisateurs",
        list_display: [
            ["username", "Utilisateur"],
            ["bio", "Bio"],
            ["is_verified", "Vérifié"],
        ],
    }
}
```

### Ce qui est généré

| Symbole | Description |
| ------- | ----------- |
| `Model` | Struct avec toutes les colonnes (base + étendues) |
| `Column` | Enum SeaORM pour les colonnes |
| `Entity` | `EntityTrait` complet — utilisable avec `search!` |
| `AdminForm` | Formulaire admin couvrant toutes les colonnes |
| `admin_from_form` | Construit un `ActiveModel` depuis les données du formulaire |
| `admin_partial_update` | Construit un `ActiveModel` partiel pour la mise à jour |

### Requêtes depuis les vues

L'entité générée est un `EntityTrait` SeaORM standard — `search!` fonctionne directement :

```rust
// Tous les profils vérifiés
let profiles = search!(user_profile::Entity => is_verified eq true).fetch(&db).await?;

// Recherche multi-colonnes
let results = search!(user_profile::Entity => or(username icontains q, bio icontains q)).fetch(&db).await?;
```

### Relations vers l'entité étendue

D'autres entités peuvent pointer vers l'entité étendue via le bloc `relations:` habituel de `model!{}` :

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    { auteur_id: int [required] },
    relations: {
        belongs_to: user_profile::Model via auteur_id,
    }
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré, `schema()`, `ModelSchema` |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]`, liaison modèle/formulaire |

## Retour au sommaire

- [Models](/docs/fr/model)
