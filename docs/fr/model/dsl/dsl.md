# DSL `model!` & AST

## Macros réellement exposées

Dans la crate `derive_form`, les macros disponibles sont :

- `model!(...)` (proc macro)
- `#[form(...)]` (proc macro attribut)

Côté API Runique (`prelude`), `model` et `form` sont ré-exportées.

---

## DSL `model!(...)` : structure attendue

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

model!(
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
        has_many(Post),
    },
    meta: {
        ordering: [-created_at],
    }
);
```

---

## AST interne (ce qui est parsé)

La DSL est convertie en AST `Model` avec notamment :

- `name`, `table`, `pk`
- `fields: Vec<FieldDef>`
- `relations: Vec<RelationDef>`
- `meta: Option<MetaDef>`

### Types pris en charge

- texte : `String`, `text`, `char`, `varchar(n)`
- numériques : `i8/i16/i32/i64/u32/u64/f32/f64`, `decimal(p,s)`
- date/temps : `date`, `time`, `datetime`, `timestamp`, `timestamp_tz`
- autres : `bool`, `uuid`, `json`, `json_binary`, `binary`, `blob`, `enum(...)`, `inet`, `cidr`

### Options de champ

- `required`, `nullable`, `unique`, `index`
- `default(...)`, `max_len(...)`, `min_len(...)`, `max(...)`, `min(...)`
- `auto_now`, `auto_now_update`, `readonly`
- `fk(table.column, cascade|set_null|restrict|set_default)`

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré, `ModelSchema` |
| [Formulaires & enjeux](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/formulaires/formulaires.md) | `#[form(...)]` |

## Retour au sommaire

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
