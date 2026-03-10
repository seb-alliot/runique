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
4. `fields: { ... }`,
5. `relations: { ... }` optionnel,
6. `meta: { ... }` optionnel.

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
        is_active: bool [default(true)],
        created_at: datetime [auto_now],
    },
    relations: {
        has_many: Post,
        belongs_to: Team via team_id,
    },
    meta: {
        ordering: [-created_at],
        verbose_name: "utilisateur",
    }
}
```

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
- autres : `bool`, `uuid`, `json`, `json_binary`, `binary(n)`, `binary`, `blob`, `enum(A, B, ...)`, `inet`, `cidr`, `mac_address`

### Options de champ

- `required`, `nullable`, `unique`, `index`, `readonly`
- `default(...)`, `max_len(n)`, `min_len(n)`, `max(n)`, `min(n)`, `max_f(n)`, `min_f(n)`
- `auto_now`, `auto_now_update`
- `label("...")`, `help("...")`, `select_as("...")`
- `fk(table.column, cascade|set_null|restrict|set_default)`

### Relations

| Syntaxe | Description |
| --- | --- |
| `has_many: Model,` | Relation 1-N |
| `has_many: Model as alias,` | 1-N avec nom d'accès personnalisé |
| `has_one: Model,` | Relation 1-1 |
| `has_one: Model as alias,` | 1-1 avec nom d'accès personnalisé |
| `belongs_to: Model via fk_field,` | Clé étrangère entrante |
| `many_to_many: Model through pivot_table,` | Relation N-N |

### Meta

| Clé | Valeur | Description |
| --- | --- | --- |
| `ordering: [field, -field]` | liste | Tri par défaut (`-` = DESC) |
| `unique_together: [(f1, f2), ...]` | liste de tuples | Contrainte d'unicité composite |
| `verbose_name: "..."` | string | Nom affiché (singulier) |
| `verbose_name_plural: "..."` | string | Nom affiché (pluriel) |
| `abstract: true` | bool | Modèle abstrait (pas de table) |
| `indexes: [(f1, f2), ...]` | liste de tuples | Index composites |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré, `ModelSchema` |
| [Formulaires & enjeux](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/formulaires/formulaires.md) | `#[form(...)]` |

## Retour au sommaire

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
