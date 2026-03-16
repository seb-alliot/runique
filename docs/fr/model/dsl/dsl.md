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

- `required`, `nullable`, `unique`, `readonly`
- `max_len(n)`, `min_len(n)`, `max(n)`, `min(n)`, `max_f(n)`, `min_f(n)`
- `auto_now`, `auto_now_update`
- `label(...)`, `help(...)`, `select_as(...)`

### Relations

Les relations sont déclarées dans un bloc `relations: { ... }` optionnel après `fields`.

| Syntaxe | Contrainte DB | Description |
| --- | --- | --- |
| `belongs_to: Model via fk_field,` | ✅ `FOREIGN KEY` générée | Clé étrangère vers `model.id` |
| `belongs_to: Model via fk_field [cascade],` | ✅ `ON DELETE CASCADE` | FK avec on_delete cascade |
| `belongs_to: Model via fk_field [cascade, restrict],` | ✅ | FK avec on_delete + on_update |
| `has_many: Model,` | ❌ (code uniquement) | Relation 1-N |
| `has_one: Model,` | ❌ (code uniquement) | Relation 1-1 |
| `many_to_many: Model via pivot_table,` | ❌ (code uniquement) | Relation N-N |

Actions FK disponibles : `cascade`, `restrict`, `set_null`, `set_default` (défaut : `no_action`).

> `belongs_to` génère automatiquement une `FOREIGN KEY` dans la migration. La colonne FK (`fk_field`) doit être déclarée dans `fields`.

### Meta

> Le bloc `meta` est réservé aux futures versions (ordering, verbose_name, etc.). Il est parsé sans erreur mais ignoré.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Génération & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/generation/generation.md) | Code généré, `ModelSchema` |
| [Formulaires & enjeux](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/formulaires/formulaires.md) | `#[form(...)]` |

## Retour au sommaire

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/fr/model/12-model.md)
