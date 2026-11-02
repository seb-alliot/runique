# Boot validation

No admin file generation anymore. Instead, `AdminStaging::validate()` checks the whole config **before** routes are mounted. An error stops startup with a clear message.

## When it runs

1. `.with_admin(admin::config)` builds the staging
2. `build()` calls `staging.validate()`
3. On `Ok`, CRUD routes mount under `prefix + path`

You can replay the same checks in a test:

```rust
#[test]
fn every_registered_model_passes_the_boot_checks() {
    let staging = super::config(AdminStaging::new().enable());
    if let Err(e) = staging.validate() {
        panic!("{e}");
    }
}
```

## What is checked

### Per model

**List**

- `links` and `sortable` only name columns of `display`;
- `search_help` without a search (`search([])`) is refused;
- a grouped action doesn't target the primary key, its value fits the column, and one column doesn't get the same value twice.

**Form**

- each field of `fields` exists in the model's form (`ModelMeta::Form`) and can hold the column's type;
- fields named elsewhere (labels, read-only…) are in `fields`;
- a secret column required on create (NOT NULL, no default) is refused: the admin could never fill it, give it a default.

**Both**

- a secret column (`password`…) appears in no list: display, search, filter, sort, form, action;
- a grouped action targets a column the form writes on edit (in `fields`, neither `readonly` nor `create_only`);
- a grouped action never targets the owner column: a transfer is its own operation;
- on the accounts: no grouped action on the ranks or on `is_active`, and no rank flag in `fields`.

### Site level

- a model registered twice is refused;
- the accounts admin is replaced only through `accounts_users`, with the `eihwaz_users` table (its key and protections);
- the groups are served by the framework: registering them is refused.

### Staging level

- the admin path isn't empty;
- admin routes declared in `urlpatterns!` while the admin is disabled: refused;
- each declared admin route targets a registered model, has an absolute path (exactly one `{id}` segment, and nothing else, to replace the detail), overlaps neither a public route nor the admin's namespace, is declared once (path and name), and a view is replaced only once.

### Elsewhere at boot

- the template overrides (see [Template override](/docs/en/admin/template-surcharge));
- in production, `with_public_url` is required when the admin is enabled;
- group rights targeting an unregistered model are deleted from the database (only once the project registered at least one of its models).

## Defaults

- without `display`: every column but the secret ones; on the accounts, only `username`, `email`, `is_active`;
- without `.form()`: every field of the model's form (`ModelMeta::Form`).

At runtime, each column list is then cut to what the user's rights let them see or write.

## Relevant files

| File | Role |
| --- | --- |
| `src/admin.rs` | Your config (maintain this) |
| `runique/src/admin/builder/` | Builder + validation |
| `runique/src/app/staging/admin_staging.rs` | Staging + `validate()` |

Nothing is written under `src/admins/`: that folder is gone.

## See also

| Section | Description |
| --- | --- |
| [ModelAdmin builder](/docs/en/admin/declaration-macro) | API feeding validation |
| [Setup](/docs/en/admin/setup) | Wiring |
| [Permissions](/docs/en/admin/permission) | Runtime gate |

## Back to summary

- [Admin summary](/docs/en/admin)
