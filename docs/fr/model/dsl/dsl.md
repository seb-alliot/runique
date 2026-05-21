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
    fields: { ... },        // 5. Champs (syntaxe v1 ou v2)
    relations: { ... },     // 6. Optionnel — relations SeaORM
    meta: { ... },          // 7. Optionnel — contraintes & tri
}
```

### Deux syntaxes de champs

**Syntaxe v1 — types SQL explicites** (bloc nommé `fields:`) :

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    fields: {
        titre:      String [required, max_len(150)],
        contenu:    text   [required],
        is_active:  bool,
        created_at: datetime [auto_now],
    },
}
```

**Syntaxe v2 — types sémantiques** (bloc anonyme `{ ... }`) :

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

> En syntaxe v2, le bloc anonyme remplace à la fois `fields:` et `form_fields:` — ces deux blocs nommés n'existent pas en v2. En v1, `form_fields:` est un bloc optionnel parsé après `meta:`, permettant d'ajouter des annotations sémantiques sur des champs déjà déclarés en SQL.

---

## Clé primaire (`pk`)

```
pk: nom_champ => type
```

| Type   | SQL Postgres        | SQL MySQL              | Auto-incrément | Création |
|--------|---------------------|------------------------|----------------|----------|
| `i32`  | `SERIAL`            | `INT AUTO_INCREMENT`   | ✅ Oui          | séquence DB |
| `i64`  | `BIGSERIAL`         | `BIGINT AUTO_INCREMENT`| ✅ Oui          | séquence DB |
| `uuid` | `UUID`              | `VARCHAR(36)`          | ❌ Non          | `Uuid::new_v4()` côté Rust |
| `Pk`   | alias `i32` ou `i64`| idem                   | ✅ Oui          | selon feature `big-pk` |

**L'alias `Pk`** résout en `i32` par défaut, ou `i64` si la feature `big-pk` est activée :

```toml
[dependencies]
runique = { version = "2.1.5", features = ["big-pk"] }
```

Utilisez `big-pk` quand vous anticipez plus de ~2 milliards de lignes dans une table, ou pour interopérer avec un schéma existant utilisant des clés primaires `BIGINT`.

**Contraintes lors de l'activation de `big-pk` :**

- Chaque colonne FK pointant vers une clé primaire `Pk` doit aussi être déclarée `bigint`, sinon vous obtenez une erreur de type à la compilation :

```rust
derive_form! {
    Commande {
        fields: {
            user_id: bigint [required]   // doit correspondre à users.id qui est Pk (i64)
        }
    }
}
```

- Le daemon admin génère `parse::<Pk>()` par défaut dans `admin.rs`, le code généré suit donc automatiquement la feature — aucun ajustement manuel nécessaire.

- Les fichiers de seeds et tout code manuel qui assigne `entity.id` (un `Pk`) à un champ FK `i32` doivent utiliser `.try_into().unwrap()` ou changer la colonne FK en `bigint`.

> **`big-pk` doit être décidé avant la première migration.**
> Une fois les migrations appliquées, basculer entre `big-pk` et le mode par défaut (`i32`) est un changement cassant : les colonnes en base sont déjà `INT` ou `BIGINT`, et changer la feature flag ne modifie que le type Rust — le schéma reste intact. Changer après coup nécessite une migration manuelle pour `ALTER` chaque colonne PK et FK, avec un risque de troncature des données si des IDs existants dépassent `i32::MAX`. Choisissez un mode au démarrage du projet et ne le changez pas.

---

## Types de champs — syntaxe v1

Types SQL déclarés directement :

| Type DSL          | Type Rust généré          | Colonne SQL               |
|-------------------|---------------------------|---------------------------|
| `String`          | `String`                  | `VARCHAR(255)`            |
| `text`            | `String`                  | `TEXT`                    |
| `char`            | `String`                  | `CHAR`                    |
| `varchar(n)`      | `String`                  | `VARCHAR(n)`              |
| `i8`              | `i32`                     | `TINYINT`                 |
| `i16`             | `i32`                     | `SMALLINT`                |
| `i32`             | `i32`                     | `INTEGER`                 |
| `i64`             | `i64`                     | `BIGINT`                  |
| `u32`             | `u32`                     | `INTEGER UNSIGNED`        |
| `u64`             | `u64`                     | `BIGINT UNSIGNED`         |
| `f32`             | `f32`                     | `FLOAT`                   |
| `f64`             | `f64`                     | `DOUBLE`                  |
| `decimal`         | `Decimal`                 | `DECIMAL`                 |
| `decimal(p, s)`   | `Decimal`                 | `DECIMAL(p, s)`           |
| `bool`            | `bool`                    | `BOOLEAN`                 |
| `date`            | `NaiveDate`               | `DATE`                    |
| `time`            | `NaiveTime`               | `TIME`                    |
| `datetime`        | `NaiveDateTime`           | `DATETIME`                |
| `timestamp`       | `NaiveDateTime`           | `TIMESTAMP`               |
| `timestamp_tz`    | `NaiveDateTime`           | `TIMESTAMPTZ`             |
| `uuid`            | `Uuid`                    | `UUID`                    |
| `json`            | `serde_json::Value`       | `JSON`                    |
| `json_binary`     | `serde_json::Value`       | `JSON BINARY`             |
| `binary`          | `Vec<u8>`                 | `BINARY`                  |
| `binary(n)`       | `Vec<u8>`                 | `BINARY(n)`               |
| `var_binary(n)`   | `Vec<u8>`                 | `VARBINARY(n)`            |
| `blob`            | `Vec<u8>`                 | `BLOB`                    |
| `inet`            | `String`                  | `INET`                    |
| `cidr`            | `String`                  | `CIDR`                    |
| `mac_address`     | `String`                  | `MACADDR`                 |
| `interval`        | `String`                  | `INTERVAL`                |
| `enum(NomEnum)`   | `NomEnum`                 | `INTEGER` / `ENUM` / `VARCHAR` |

---

## Types de champs — syntaxe v2 (sémantiques)

Convertis automatiquement en types SQL :

| Type sémantique | SQL généré                            | Notes                               |
|-----------------|---------------------------------------|-------------------------------------|
| `text`          | `VARCHAR(255)` ou `VARCHAR(n)` si `max_length: n` |                       |
| `email`         | `VARCHAR(254)`                        | Format email validé                 |
| `password`      | `VARCHAR(255)`                        | Haché automatiquement               |
| `richtext`      | `TEXT`                                | Éditeur HTML                        |
| `textarea`      | `TEXT`                                | Multi-ligne                         |
| `url`           | `VARCHAR(255)`                        | Format URL validé                   |
| `slug`          | `VARCHAR(255)`                        |                                     |
| `color`         | `VARCHAR(255)`                        | Couleur hexadécimale                |
| `ip`            | `INET`                                |                                     |
| `phone`         | `VARCHAR(20)` ou `VARCHAR(n)` si `max_length: n` | `<input type="tel">` |
| `int`           | `INTEGER`                             |                                     |
| `bigint`        | `BIGINT`                              |                                     |
| `float`         | `DOUBLE`                              |                                     |
| `decimal`       | `DECIMAL`                             |                                     |
| `percent`       | `DOUBLE`                              | Stocké comme float                  |
| `bool`          | `BOOLEAN`                             |                                     |
| `date`          | `DATE`                                |                                     |
| `time`          | `TIME`                                |                                     |
| `datetime`      | `DATETIME`                            |                                     |
| `uuid`          | `UUID`                                |                                     |
| `json`          | `TEXT`                                |                                     |
| `image`         | `VARCHAR(255)`                        | Stocke le chemin du fichier         |
| `document`      | `VARCHAR(255)`                        | Stocke le chemin du fichier         |
| `file`          | `VARCHAR(255)`                        | Stocke le chemin du fichier         |
| `choice`        | `VARCHAR` / `ENUM` natif              | Requiert `enum(NomEnum)`            |
| `radio`         | Idem `choice`                         | Widget différent, même SQL          |
| `checkbox`      | Idem `choice`                         | Widget différent, même SQL          |

---

## Options de champ — syntaxe v1

Dans un bloc `[...]`, séparées par des virgules :

```rust
username: String [required, max_len(150), unique],
```

| Option              | Description                                                    |
|---------------------|----------------------------------------------------------------|
| `required`          | Colonne `NOT NULL` + validation formulaire                     |
| `nullable`          | Colonne `NULL` — type Rust `Option<T>`                         |
| `unique`            | Contrainte `UNIQUE`                                            |
| `index`             | Index simple (non unique)                                      |
| `default(valeur)`   | Valeur par défaut SQL (`true`, `0`, `"draft"`, etc.)           |
| `max_len(n)`        | Longueur max (validation + `VARCHAR(n)`)                       |
| `min_len(n)`        | Longueur min (validation)                                      |
| `max(n)`            | Valeur max entière (validation)                                |
| `min(n)`            | Valeur min entière (validation)                                |
| `max_f(n)`          | Valeur max flottante                                           |
| `min_f(n)`          | Valeur min flottante                                           |
| `auto_now`          | Assigné à `NOW()` à chaque `INSERT` — exclu des formulaires    |
| `auto_now_update`   | Assigné à `NOW()` à chaque `UPDATE` — exclu des formulaires    |
| `readonly`          | Exclu des formulaires générés                                  |
| `select_as(str)`    | Alias SQL dans les SELECT                                      |
| `label("str")`      | Libellé personnalisé dans les formulaires admin                |
| `help("str")`       | Texte d'aide (réservé)                                        |
| `fk(table.col, action)` | Contrainte clé étrangère (voir Relations)                 |
| `file(kind)`        | Champ fichier — `image`, `document`, `any`                     |
| `file(kind, "path")`| Champ fichier avec dossier d'upload explicite                  |
| `max_size(n)`       | Taille max upload — `n KB`, `n MB`, `n GB`                     |

## Options de champ — syntaxe v2

Utilisent `:` au lieu de `()` pour les valeurs :

```rust
username: text [required, max_length: 150, unique],
```

| Option v2              | Équivalent v1          | Notes                          |
|------------------------|------------------------|--------------------------------|
| `required`             | `required`             |                                |
| `nullable`             | `nullable`             |                                |
| `unique`               | `unique`               |                                |
| `max_length: n`        | `max_len(n)`           |                                |
| `min_length: n`        | `min_len(n)`           |                                |
| `min: n`               | `min(n)`               |                                |
| `max: n`               | `max(n)`               |                                |
| `min: n.0`             | `min_f(n)`             |                                |
| `max: n.0`             | `max_f(n)`             |                                |
| `default: valeur`      | `default(valeur)`      |                                |
| `auto_now`             | `auto_now`             |                                |
| `auto_now_update`      | `auto_now_update`      |                                |
| `upload_to: "path"`    | `file(kind, "path")`   |                                |
| `max_size: n MB`       | `max_size(n MB)`       |                                |
| `rows: n`              | —                      | V2 uniquement (textarea)       |
| `step: n`              | —                      | V2 uniquement (numériques)     |
| `fk(table.col, action)`| `fk(table.col, action)`|                                |
| `enum(NomEnum)`        | `enum(NomEnum)`        |                                |
| `skip`                 | `readonly`             |                                |
| `no_hash`              | —                      | Champs `password` uniquement   |

> **`auto_now` / `auto_now_update`** : ces champs sont exclus de `admin_from_form` et d'`admin_partial_update`. Leur valeur est gérée uniquement par la base. Ils apparaissent dans `Model` et `Column` comme `Option<T>`.

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

### Quatre formes de variant

| Syntaxe                              | Valeur DB        | Libellé affiché (Display) |
|--------------------------------------|------------------|---------------------------|
| `Variant`                            | `"Variant"`      | `"Variant"`               |
| `Variant: "Libellé"`                 | `"Variant"`      | `"Libellé"`               |
| `Variant = "valeur_db"`              | `"valeur_db"`    | `"valeur_db"`             |
| `Variant = ("valeur_db", "Libellé")` | `"valeur_db"`    | `"Libellé"`               |

> **La valeur DB est stockée exactement telle qu'écrite.** Aucune transformation automatique.

### Types de backing

| Syntaxe              | Stockage DB                                     |
|----------------------|-------------------------------------------------|
| `NomEnum: [A, B]`    | `ENUM` natif (Postgres) ou `VARCHAR` (MySQL/SQLite) |
| `NomEnum: i32 [...]` | `INTEGER`                                       |
| `NomEnum: i64 [...]` | `BIGINT`                                        |

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
|------------------|-----------------|------------------------------|
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

## `label` et `help`

Par défaut, le libellé est généré depuis le nom snake_case (`sort_order` → `Sort order`). L'option `label(...)` le remplace :

```rust
fields: {
    titre:        text [required, label("Titre de l'article")],
    sort_order:   i32  [label("Ordre d'affichage")],
    is_published: bool [label("Publié")],
},
```

> `label` et `help` sont des options **v1 uniquement** — non disponibles dans le bloc anonyme v2.

Le libellé s'applique au formulaire admin et aux en-têtes de colonnes dans `list_display`. Il n'a aucun effet sur la migration.

---

## `extend!{}` — extension des tables framework

Ajoute des colonnes à une table Runique sans la redéfinir. Le CLI `makemigrations` détecte les blocs `extend!{}` et génère des instructions `ALTER TABLE ADD COLUMN`.

```rust
extend! {
    table: "eihwaz_users",
    fields: {
        avatar:  image   [upload_to: "avatars/"],
        bio:     textarea,
        website: url     [required],
    }
}
```

Tables autorisées : `eihwaz_users`, `eihwaz_groupes`, `eihwaz_droits`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Tout autre nom provoque une erreur à la compilation.

Les champs déclarés dans `extend!{}` utilisent les mêmes types et options que la syntaxe v2 de `model!`.

> **Limitation** : la macro `search` ne fonctionne pas encore sur les colonnes ajoutées via `extend!{}`.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré, `schema()`, `ModelSchema` |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]`, liaison modèle/formulaire |

## Retour au sommaire

- [Models](/docs/fr/model)
