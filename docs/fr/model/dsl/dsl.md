# DSL `model!` & AST

## Macros réellement exposées

Dans la crate `derive_form`, les macros disponibles sont :

- `model! { ... }` (proc macro)
- `#[form(...)]` (proc macro attribut)

Côté API Runique (`prelude`), `model` et `form` sont ré-exportées.

---

## DSL `model! { ... }` : structure attendue

Le parseur attend une structure stricte :

1. nom du modèle,
2. `table: "..."`,
3. `pk: id => i32|i64|uuid`,
4. `enums: { ... }` optionnel,
5. `fields: { ... }`,
6. `relations: { ... }` optionnel,
7. `meta: { ... }` optionnel.

Exemple concret :

```rust
use runique::prelude::*;

model! {
    User,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required],
        is_active: bool,
        team_id: i32 [required],
        created_at: datetime [auto_now],
    },
    relations: {
        has_many: Post,
        belongs_to: Team via team_id,
    },
}
```

---

## La Clé Primaire (`pk`)

Le bloc `pk` définit le nom et le type de la clé primaire. Trois types sont supportés nativement :

| Type | SQL (Postgres / MySQL) | Auto-incrément |
| --- | --- | --- |
| `i32` | `SERIAL` / `INT AUTO_INC` | ✅ Oui (défaut) |
| `i64` | `BIGSERIAL` / `BIGINT AUTO_INC` | ✅ Oui |
| `uuid` | `UUID` / `VARCHAR(36)` | ❌ Non |

**Exemple UUID :**
```rust
model! {
    Log,
    table: "logs",
    pk: id => uuid,
    fields: { ... }
}
```

#### Lien avec le modèle `User` et la feature `big-pk`

Si vous utilisez `i64` pour votre modèle utilisateur (entité liée à la session), vous devez activer la feature `big-pk` dans votre `Cargo.toml` pour que l'alias global `Pk` utilisé par le framework soit synchronisé :

```toml
[dependencies]
runique = { version = "2.1.0", features = ["big-pk"] }
```

> **Attention :** Le type `uuid` pour l''alias global `Pk` (authentification) n''est pas encore supporté nativement dans la version actuelle.

---

---

## AST interne (ce qui est parsé)

La DSL est convertie en AST `Model` avec notamment :

- `name`, `table`, `pk`
- `fields: Vec<FieldDef>`
- `relations: Vec<RelationDef>`
- `meta: Option<MetaDef>`

### Types pris en charge

- texte : `String`, `text`, `char`, `varchar(n)`, `var_binary(n)`
- numériques : `i8/i16/i32/i64/u32/u64/f32/f64`, `decimal(p,s)`, `decimal`
- date/temps : `date`, `time`, `datetime`, `timestamp`, `timestamp_tz`, `interval`
- autres : `bool`, `uuid`, `json`, `json_binary`, `binary(n)`, `binary`, `blob`, `enum(NomEnum)`, `inet`, `cidr`, `mac_address`

#### Enums — `enums: { ... }` + `enum(NomEnum)`

Les enums se déclarent dans un bloc `enums: { ... }` distinct des champs, puis sont référencés dans `fields` via `enum(NomEnum)`.

```rust
model! {
    DocSection,
    table: "doc_section",
    pk: id => i32,
    enums: {
        SectionTheme: [Demarrage, Web, Database, Security, Admin, Autres],
    },
    fields: {
        theme: enum(SectionTheme) [nullable],
    },
}
```

**Types de backing :**

| Syntaxe | Stockage DB |
| --- | --- |
| `NomEnum: [A, B]` | Détecté automatiquement depuis `DB_ENGINE` : `Enum` natif sur PostgreSQL, `VARCHAR` sur MySQL/SQLite |
| `NomEnum: i32 [A, B]` | `Integer` |
| `NomEnum: i64 [A, B]` | `BigInteger` |

**Valeurs en base :** les variants sont stockés **exactement tels qu'écrits dans le DSL** — aucune transformation automatique. Si vous déclarez `[Demarrage, Web]`, la base contient `"Demarrage"` et `"Web"`. Si vous déclarez `[demarrage, web]`, la base contient `"demarrage"` et `"web"`.

**Syntaxes de variant :**

| Syntaxe | Valeur DB | Libellé affiché (admin) |
| --- | --- | --- |
| `Variant` | nom du variant tel qu'écrit | nom du variant |
| `Variant = "ordre"` | `"ordre"` | `"ordre"` |
| `Variant = ("ordre", "Ordre d'affichage")` | `"ordre"` | `"Ordre d'affichage"` |

```rust
enums: {
    SortMode: [
        Manual,
        Alphabetical = "alpha",
        ByDate = ("date", "Par date de création"),
    ],
},
```

La forme tuple `= ("valeur_db", "Libellé")` permet de stocker une valeur courte en base tout en affichant un libellé lisible dans le formulaire admin.

`FromStr` accepte les trois formes en insensible à la casse : le nom du variant, la valeur DB, et le libellé (si présent) redirigent tous vers le même variant.

**Dans les templates Tera**, la valeur de comparaison doit correspondre **exactement** à ce qui est stocké en base (sensible à la casse) :

```jinja2
{# Correct — correspond à la valeur stockée "Web" #}
{% for section in sections | filter(attribute="theme", value="Web") %}

{# Incorrect — ne correspondra jamais si la base contient "Web" #}
{% for section in sections | filter(attribute="theme", value="web") %}
```

Le filtre Tera `filter` est **strictement sensible à la casse**.

### Options de champ

- `required`, `nullable`, `unique`, `readonly`
- `max_len(n)`, `min_len(n)`, `max(n)`, `min(n)`, `max_f(n)`, `min_f(n)`
- `auto_now`, `auto_now_update`
- `label(...)`, `help(...)`, `select_as(...)`
- `file(kind)`, `file(kind, "chemin/upload")` — champ fichier (voir ci-dessous)

#### `label(...)`

Par défaut, le formulaire admin génère le libellé d'un champ à partir de son nom snake_case (`sort_order` → `Sort order`). L'option `label(...)` permet de définir un libellé personnalisé :

```rust
model! {
    Chapitre,
    table: "chapitres",
    pk: id => i32,
    fields: {
        title: String [required, label("Titre du chapitre")],
        sort_order: i32 [label("Ordre d'affichage")],
        is_published: bool [label("Publié")],
    },
}
```

Le libellé est utilisé dans le formulaire d'édition admin (balise `<label>`) et dans les en-têtes de colonnes si le champ est déclaré dans `list_display`. Il n'a aucun effet sur la migration ou l'entité SeaORM générée.

### Champs fichier — `file()`

Un champ `String` peut être déclaré comme champ fichier avec l'option `file()`. Le formulaire auto-généré (`AdminForm`) utilisera alors un `FileField` au lieu d'un `TextField`.

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    fields: {
        titre: String [required],

        // image — dossier explicite
        image: String [file(image, "media/articles")],

        // document — dossier auto depuis MEDIA_ROOT + nom du champ
        fichier: String [file(document)],

        // tout type de fichier
        piece_jointe: String [file(any, "media/uploads")],
    },
}
```

**Types disponibles :**

| Valeur | Extensions autorisées par défaut | Correspondance |
| --- | --- | --- |
| `image` | `jpg jpeg png gif webp avif` | `FileField::image()` |
| `document` | `pdf doc docx txt odt` | `FileField::document()` |
| `any` | aucun filtre | `FileField::any()` |

**Chemin d'upload :**

| Syntaxe | Destination |
| --- | --- |
| `file(image, "media/articles")` | `media/articles/` (chemin exact) |
| `file(image)` | `{MEDIA_ROOT}/{nom_du_champ}/` (lit `MEDIA_ROOT` depuis `.env`) |

> Les fichiers invalides sont supprimés du disque si la validation échoue. Le dossier de destination est créé automatiquement lors du premier upload valide.

### Relations

Les relations sont déclarées dans un bloc `relations: { ... }` optionnel après `fields`.

| Syntaxe | Contrainte DB | Description |
| --- | --- | --- |
| `belongs_to: Model via fk_field,` | ✅ `FOREIGN KEY` générée | Clé étrangère vers `model.id` |
| `belongs_to: Model via fk_field [cascade],` | ✅ `ON DELETE CASCADE` | FK avec on_delete cascade |
| `belongs_to: Model via fk_field [cascade, restrict],` | ✅ | FK avec on_delete + on_update |
| `has_many: Model,` | ❌ (code uniquement) | Relation 1-N |
| `has_one: Model,` | ❌ (code uniquement) | Relation 1-1 |
| `many_to_many: Model through PivotTable via via_field,` | ❌ (code uniquement) | Relation N-N |

Actions FK disponibles : `cascade`, `restrict`, `set_null`, `set_default` (défaut : `no_action`).

> `belongs_to` génère automatiquement une `FOREIGN KEY` dans la migration. La colonne FK (`fk_field`) doit être déclarée dans `fields`.

### Meta

> Le bloc `meta` est réservé aux futures versions (ordering, verbose_name, etc.). Il est parsé sans erreur mais ignoré.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](/docs/fr/model/generation) | Code généré, `ModelSchema` |
| [Formulaires & enjeux](/docs/fr/model/formulaires) | `#[form(...)]` |

## Retour au sommaire

- [Models](/docs/fr/model)
