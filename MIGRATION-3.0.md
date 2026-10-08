🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/MIGRATION-3.0.md) | [Français](https://github.com/seb-alliot/runique/blob/main/MIGRATION-3.0.fr.md)

# Upgrading from Runique 2.x to 3.0

This guide lists every breaking change of 3.0 and what to do about it, in the order you'll meet them while upgrading. The [CHANGELOG](CHANGELOG.md) has the full list of changes, including security fixes and additions that need no action.

---

## Step by step

1. **`Cargo.toml`**: pick one database engine (see [Cargo features](#cargo-features)).
2. **Models**: add `nullable` to optional columns and replace `fk(...)` with `belongs_to` (see [Model DSL](#model-dsl)).
3. **Database**: add the `activated_at` column to `eihwaz_users` (see [User accounts](#user-accounts)).
4. **Builder**: add `.with_public_url(...)` (see [Builder](#builder)).
5. **Code**: replace `&DatabaseConnection` with `&ADb`, then fix what the compiler points out, using the sections below.
6. **Admin**: regenerate `src/admins/` with `runique start`.
7. **Migrations**: run `runique makemigrations`; delete `migration/src/applied/` if it exists.

---

## Cargo features

**`default` no longer pulls in every engine, and the engines are mutually exclusive.** `default` is now `["orm"]`; enable exactly one of `sqlite`, `postgres` or `mysql` (`mariadb` is an alias of `mysql`). Two at once is a compile error, except through `all-databases`, meant for multi-engine tooling.

```toml
# 2.x — compiled SQLite, Postgres and MySQL whatever you used
runique = "2.2"

# 3.0
runique = { version = "3.0.0", features = ["postgres"] }
```

**`argon2` 0.6 and `scrypt` 0.12**: only matters if your own code uses these crates directly. `scrypt` 0.12 has no default features: declare `features = ["phc", "getrandom"]`.

---

## Builder

**`.with_public_url(...)` is required in production** when the password reset or the admin is enabled: every link the app sends by email is built on it, never on the request's `Host` header. Without it, the app refuses to boot outside debug mode. It replaces `PasswordResetConfig::base_url()` and `AdminConfig::reset_password_url()`.

```rust
RuniqueApp::builder(config)
    .with_public_url("https://mysite.com")
```

To build an absolute link yourself, use `request.public_url()`.

**Once-only settings can't be called twice.** `with_public_url`, `routes`, `with_log`, `with_password_reset`, `with_database` / `with_database_config`, `with_session_duration` and `with_mailer` / `with_mailer_from_env`: a second call used to replace (or silently ignore) the first; it no longer compiles. A function returning a half-built builder names its state with `app::builder::state::{No, Yes}`.

**Removed duplicates**:

| 2.x | 3.0 |
|---|---|
| `.no_statics()` | `.static_files(\|s\| s.enabled(false))` |
| `StaticStaging::enable()` / `disable()` | `.enabled(true)` / `.enabled(false)` |
| `.with_error_handler(b)` | `.middleware(\|m\| m.with_debug_errors(b))` |
| `SessionConfig`, `SessionBackend`, `ASessionStore` | `.with_session_duration(...)` on the builder, `.middleware(\|m\| m.with_session_store(...))` |
| `PasswordConfig::oauth(p)` | `PasswordConfig::Delegated(p)` |
| `MiddlewareConfig::with_host_validation` | `.middleware(\|m\| m.with_allowed_hosts(...))` |
| `RuniqueEngine::attach_middlewares` | the builder (it was never called) |

**`.env` keys no longer read**:

| Key | Replacement |
|---|---|
| `RUNIQUE_ENABLE_CACHE` | follows `DEBUG`; override with `.middleware(\|m\| m.with_cache(bool))` |
| `ALLOWED_HOSTS` | `.middleware(\|m\| m.with_allowed_hosts(\|h\| h.enabled(true).host("mysite.com")))` |
| `RATE_LIMITING` | `.rate_limit(...)` on routes, `with_rate_limiter(...)` on the admin |
| `RUNIQUE_USER_TABLE` | none: `eihwaz_users` is the only user table |

None of these did what its name said: `ALLOWED_HOSTS=mysite.com` left host validation off.

**`DEBUG` and the other `.env` flags are read whatever their case**: `True`, `YES`, `On` now mean true. Check that no production `.env` holds such a value by mistake. The `RuniqueEnv` enum is removed; use `is_debug()`.

**Log levels are `LogLevel` in the prelude**: `use runique::prelude::*` exports `tracing::Level` as `LogLevel`, so it no longer clashes with a model enum named `Level`. Replace `Level::INFO` with `LogLevel::INFO` in `.with_log(...)`, or import `tracing::Level` yourself.

---

## Database handle: `ADb`

Every public signature that took `&DatabaseConnection` or `Arc<DatabaseConnection>` takes `&ADb`: `engine.db`, `BuiltinUserEntity`, `RuniqueSessionStore::new`, `order_by_random`, `search!`, forms, admin, auth. `ADb` implements `ConnectionTrait` and `TransactionTrait` directly, so `.insert(db)`, `.one(db)` and `db.begin()` work unchanged, without `.as_ref()`.

```rust
// 2.x
pub async fn get_article(db: &DatabaseConnection, id: i32) -> Option<blog::Model>

// 3.0
pub async fn get_article(db: &ADb, id: i32) -> Option<blog::Model>
```

`.with_database(DatabaseConnection)` is unchanged; `ADb::from_connection(conn)` builds one by hand.

---

## User accounts

**A single user model: `eihwaz_users`**, extended with `extend!{}`. Removed: the `UserEntity` and `AdminAuth` traits, `DefaultAdminAuth<E>`, `AdminLoginResult`, `RuniqueAdminAuth`, `.auth()` on the admin builder, `PasswordResetAdapter` / `PasswordResetHandler`, and the type parameter of `with_password_reset`.

```rust
// 2.x
.with_password_reset::<UserEntity>(|pr| pr.forgot_route("/forgot"))
.with_admin(|a| a.auth(RuniqueAdminAuth::new()).routes(admins::routes("/admin")))

// 3.0
.with_password_reset(|pr| pr.forgot_route("/forgot"))
.with_admin(|a| a.routes(admins::routes("/admin")))
```

Lookups go through `BuiltinUserEntity::find_by_id` / `find_by_username` / `find_by_email`; admin sign-in through `auth::authenticate_admin()`.

**New `activated_at` column and `CHECK` constraint.** A password reset no longer reactivates an account the staff deactivated: the account now separates its state (`is_active`) from its first activation (`activated_at`), and the database enforces it. Upgrade an existing table without dropping it (that would cascade to sessions, groups and `extend!{}` columns). On Postgres:

```sql
BEGIN;
ALTER TABLE eihwaz_users ADD COLUMN activated_at timestamp without time zone;
UPDATE eihwaz_users SET activated_at = COALESCE(created_at, now()) WHERE is_active;
ALTER TABLE eihwaz_users ADD CONSTRAINT eihwaz_users_active_needs_activation
    CHECK (NOT is_active OR activated_at IS NOT NULL);
COMMIT;
```

An inactive account becomes pending: its owner activates it through the email link.

---

## Sign-in and sessions

**`login()` takes the user, and no longer the database.**

```rust
// 2.x
login(&session, &db, user.id, &user.username, user.is_staff, user.is_superuser, db_store, exclusive)

// 3.0
login(&session, &user, db_store, exclusive)
```

**`login()` checks the account and returns `LoginError`.** An account that may not sign in (`can_sign_in()`: inactive, or never activated) gets no session, whatever path loaded it. Handle the refusal:

```rust
match login(&session, &user, None, false).await {
    Ok(()) => { /* signed in */ }
    Err(LoginError::CannotSignIn) => { /* inactive, or not activated yet */ }
    Err(LoginError::Session(e)) => { /* the session store failed */ }
}
```

**`auth_login()` is removed.** It reloaded an account you already had, and returned `Ok(())` without signing anyone in when the account couldn't. Pass the account to `login()`; the default session store already saves signed-in sessions to the database.

```rust
// 2.x
auth_login(&session, &db, user.id).await?;

// 3.0 — after authenticate_user, a registration, an activation…
login(&session, &user, None, false).await?;
```

**`activate_pending()` becomes `activate_account()`** and returns the activated account (`Option<Model>`) instead of a `bool`, ready for `login()`. `None`: already activated, or deactivated since by the staff (reactivation stays theirs).

```rust
if let Some(user) = BuiltinUserEntity::activate_account(&db, id).await? {
    login(&session, &user, None, false).await?;
}
```

**The session holds only the user id.** The name and the `is_staff` / `is_superuser` flags are no longer copied into it: the account is read from the database on every request, like Django's `request.user`, so a rename, a demotion or a deactivation applies to the next request.

| 2.x | 3.0 |
|---|---|
| `get_username(&session)` | `request.user` → `user.username` |
| `get_user_id(&session)` | `request.user` → `user.id` (`get_user_id` is now internal) |
| `is_admin_authenticated(&session)` | `request.user` → `user.can_access_admin()`; in a middleware, the `CurrentUser` extension |
| `SESSION_USER_USERNAME_KEY`, `SESSION_USER_IS_SUPERUSER_KEY` | removed |
| `session::SESSION_USER_IS_STAFF_KEY`, `session::IS_ACTIVE` (form field names, not session keys) | `admin_context::user::IS_STAFF`, `admin_context::user::IS_ACTIVE` |
| `session::SESSION_USER_DROITS_KEY` (admin resource key) | `admin_context::permission::DROITS` |
| `admin_context::<template>::REQUIRED` | removed (never checked) |
| `.with_log(\|l\| l.auth(\|a\| a.permissions(...)))` | removed (it logged the permission cache, gone) |

`is_authenticated(&session)` is unchanged.

**`logout()` clears the whole session**, flash messages included, like Django. Add a flash message meant for after logging out *after* the call.

**Rights are read from the database, not cached.** `cache_permissions`, `get_permissions`, `evict_permissions`, `clear_cache`, `restore_permissions`, `CachedPermissions`, `refresh_cache_for_user` and `load_user_middleware` are removed, with nothing to replace them. Outside the admin, `CurrentUser.groupes` is empty unless the handler calls `req.load_user_rights().await`.

**`GuardRules`**: `GuardContext` is removed (nothing ever filled it, so every rule refused). Roles are group names, with a single method:

```rust
// 2.x
GuardRules::login_and_role("editeur")

// 3.0
GuardRules::roles(["editeur"])   // also: GuardRules::staff(), GuardRules::superuser()
```

`role`, `login_and_role`, `login_and_roles` and `with_role` are removed.

---

## Handlers and forms

**`Request::is_get` / `is_post` / `is_put` / `is_delete` are removed.** Use `ValidationForm` for forms, or compare the method:

```rust
match ValidationForm::try_new(form, &request).await {
    Ok(valid) => { /* save */ }
    Err(form) => { /* render with errors */ }
}

// or
if request.method == Method::POST { ... }
```

**`request.query::<T>()` returns `AppResult<T>`**: a query string that doesn't fit `T` is a 400, rendered with `400.html` (overridable). `T` no longer needs `Default`.

```rust
// 2.x
let filters: Filters = request.query();

// 3.0
let filters: Filters = request.query()?;
```

**A custom `FormField` implementation**: `validate` and `finalize` are `async fn`. Add `#[async_trait::async_trait]` above the `impl` block.

**`cleaned_enum::<T>()` relies on `FromStr`.** Enums generated by `model!{}` have it; a hand-written `ActiveEnum` needs it.

**`Prisme::for_test` and `Forms::mark_validated`** are only compiled with the `test-utils` feature.

**Flash messages**: the CSS class is lowercase. Rename `.message-Success` / `Error` / `Info` / `Warning` to `.message-success` and so on.

**Other removed APIs**: `Request::render_with` (call `insert`, then `render`), `ErrorContext::with_request` (`with_request_helper`), `ErrorContext::with_details`, `RuniqueUser::roles`, `RuniqueUser::password_hash` (read the `password` field of the model), `RuniqueSessionStore::find_by_user`, `update_password` by email, `RuniqueQueryBuilder::all_from_engine`, `sanitize_with_fallback`, the aliases `Bdd`, `OADb`, `OSecurityCsp`, `OSecurityHosts`, `TResult`, `DbResult`, and the constants `NONCE_KEY`, `SESSION_USER_ROLES_KEY`, `REGISTERED_ROLES`.

---

## Admin

Regenerate `src/admins/` with `runique start` after upgrading: the generated code follows every change below.

**`extra_routes` names the operation**, which decides the right checked (read, create, update, delete):

```rust
("/orders/{id}/detail", "orders", CrudOperation::Edit, get(order_detail))
```

**`AdminResource::new` takes 4 arguments**: `ResourcePermissions`, `with_permissions`, the `permissions` field, the `roles` parameter and the roles registry (`register_roles`, `get_roles`) are removed. Rights come from groups.

**A hand-written `count_fn`** receives the column filters: `(ADb, search, column_filters, scope)`, to apply like `list_fn`.

---

## Model DSL

The `model!{}` / `extend!{}` DSL is now read by `runique_dsl`, shared by the macro and `makemigrations`. Full grammar: [`runique_dsl` README](runique/runique_dsl/README.md).

**Columns are NOT NULL by default.** `nullable` allows NULL; `required` only makes the form field mandatory. Without `nullable` on an existing optional column, `makemigrations` stops on `nullable -> not_null`.

```rust
// 2.x — optional because not `required`
phone: text [max_length: 20],

// 3.0
phone: text [max_length: 20, nullable],
```

**`[step: x]` is removed**: it was accepted and then ignored on `int`, `float`, `decimal` and `percent`. Remove it; a hand-written slider keeps `NumericField::range(...).step(x)`.

**`fk(...)` is removed**: declare foreign keys in `relations` with `belongs_to`.

```rust
// 2.x
author_id: int [fk(eihwaz_users.id, cascade)],

// 3.0
author_id: int,
// ...
relations: {
    belongs_to: eihwaz_users via author_id [cascade],
},
```

**`auto_now` / `auto_now_update`**: the Rust field is `T`, no longer `Option<T>`. They're set by the entity (`before_save`), the same way on every engine: migrations no longer create a Postgres trigger or `ON UPDATE`.

**A column's name no longer decides anything**: `created_at` / `updated_at` without `auto_now` lose their `DEFAULT CURRENT_TIMESTAMP`, and a `cache_key` column is no longer left out of migrations.

**`i32` / `i64` enums**: every variant needs its own integer value.

**`checkbox [enum(X)]` is a list**, stored in its own `{table}_{field}` table.

**Also**: the CLI refuses an unreadable model, a v1 type (`String`, `i32`…), two `model!{}` in one file or an unresolved `belongs_to` target; `customize` panics if it loosens `min_length`, `max_length`, `min` or `max`; `RelationDef`, `RelationKind` and `ModelSchema::relation()` are removed.

Republish (or update) `derive_form` with `runique`: the generated conversions return a `Result`.

---

## Migrations

**`runique migration down` and `runique migration status` are removed**: they never updated `seaql_migrations`. Use SeaORM, which runs the real `down()`:

```bash
sea-orm-cli migrate down -n 1
sea-orm-cli migrate status
```

`runique migration up` stays. The `migration/src/applied/` folder is no longer generated: delete it. Keep `snapshots/`, which `makemigrations` diffs against.
