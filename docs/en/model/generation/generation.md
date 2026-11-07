# Generation & ModelSchema

## What `model!(...)` generates

From the DSL read by `runique_dsl`, the macro generates, among others:

- the SeaORM entity (`Model`, `Entity`, `Column`, `ActiveModel`) and its relations (`belongs_to` targets the related entity's real primary key, with its actions);
- the `ActiveModelBehavior` that fills `auto_now` / `auto_now_update`;
- one Rust enum per entry of the `enums:` block;
- the model's form (`{Model}ModelForm`, alias `Form`) and the functions the admin uses;
- for each list field (`multichoice` / `checkbox`), the entity of its table and the methods `field()`, `set_field()`, `load_field()`;
- a `schema() -> ModelSchema` function.

```rust
pub fn schema() -> runique::migration::schema::ModelSchema {
    runique::migration::ModelSchema::new("User")
        .table_name("users")
        // pk, columns (types, bounds, enums), FK, indexes, meta...
        .build()
        .unwrap()
}
```

---

## Role of `ModelSchema`

`ModelSchema` describes the model's **columns** (types, declared bounds, enums, foreign keys, indexes) for forms. It isn't used for migrations: `makemigrations` reads the DSL directly. List fields aren't part of it, having no column.

### Methods used by forms

- `fill_form(form, fields, exclude)`: fills a `#[form(schema = ...)]` form from the schema;
- `enforce_limits(form)`: called after `customize`, checks the form didn't loosen what the model declares (`min_length`, `max_length`, `min`, `max`, a `decimal`'s digits, integer type, password field) — panics otherwise. It runs every time the form is built: the error shows on the first request that uses it, not at startup. Tightening a limit stays allowed.

### `fill_form` behavior

- the PK is never a field: it is not one of the schema's columns. Listing it in `fields = [...]` compiles, but makes building the form panic ("field does not exist in the schema"),
- if `fields` is provided: whitelist takes priority (order preserved),
- otherwise `exclude` is used as a blacklist.

---

## See also

| Section | Description |
| --- | --- |
| [DSL & AST](/docs/en/model/dsl) | `model!` syntax, types, options |
| [Forms & technical considerations](/docs/en/model/forms) | `#[form(...)]` |

## Back to summary

- [Models](/docs/en/model)
