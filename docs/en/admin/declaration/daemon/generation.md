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

### Per model (`ModelAdmin::validate`)

- list / form consistency (action on a non-writable column, etc.)
- secrets never exposed (display, search, filter, sort, form, action)
- on the accounts table: no bulk action on ranks / `is_active`
- ranks absent from `fields` (gate-only)
- owner never the target of a disguised transfer action

### Site level (`AdminSite::check`)

- every registered model passes its checks
- framework accounts admin is not duplicated by a rogue `register`
- groups and accounts stay consistent

### Staging level

- valid admin path
- no declared route collisions
- rate limiter / login guard coherent when set

## Defaults before serving

- without `display`: non-secret columns (accounts → username, email, is_active)
- without `links`: first display column
- without `form.fields`: all model form fields, filtered by the gate

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
