🌍 **Languages**:[English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Changelog

All notable changes to this project will be documented in this file.

---

## [2.1.12] - 2026-05-30

### Fix — `runique` (sessions)

* **DB session fallback broken after `cycle_id()` (critical regression introduced in 2.1.9):** the session fixation fix added `session.cycle_id()` on every privilege elevation. tower-sessions calls `create()` (not `save()`) at response commit for a recycled session. `create()` in `CleaningMemoryStore` had no DB persistence, so the authenticated session data was never written to the DB store after a login — breaking the warm-restart fallback entirely. Fixed: `create()` now persists to `RuniqueSessionStore` when `SESSION_USER_ID_KEY` is present, identical to the existing logic in `save()`.

* **Orphaned DB session entries after `cycle_id()` (cleanup regression):** tower-sessions calls `delete(old_id)` after `create(new_id)` during `cycle_id()`. `CleaningMemoryStore::delete()` only removed the entry from memory, leaving the old session ID as an orphan in the DB. Fixed: `delete()` now also calls `db.delete()` when the DB fallback is configured. The operation is idempotent — `logout()` already removes the entry via `RuniqueSessionStore::delete()` before calling `session.delete()`.

* **`exclusive_login` only invalidated memory sessions, not DB sessions:** `CleaningMemoryStore::save()` evicted in-memory sessions for the same user when `exclusive_login = true`, but never called `RuniqueSessionStore::invalidate_other_sessions()`. After a server restart, the evicted sessions were restored from DB, making the exclusive-login guarantee ineffective. Fixed: the DB invalidation is now collected inside the lock and executed after release, symmetric with the memory cleanup. Using `Pk` directly (`serde_json::from_value::<Pk>`) instead of `as_i64` — correct under the `big-pk` feature flag.

### Security — `runique` (forms, admin, templates)

* **CSRF token computed but never enforced on public forms (critical):** the Prisme pipeline computes `csrf_valid` for every mutating request but returns it as a flag without rejecting. The form layer (`Request::form()` / `is_valid()`) never consumed it, and the CSRF hidden-field validator is a no-op since `set_expected_value` was removed (masked tokens differ per request). Only the admin panel re-checked the flag manually, so any public POST handler built on the documented `form.is_valid()` pattern accepted cross-site forged submissions — verified end-to-end against a live registration endpoint (an account was created with no session cookie and no token). Fixed: `Request::form()` now sets `force_invalid` when the request is mutating and `prisme.csrf_valid` is false, so `is_valid()` fails closed — reusing the existing honeypot mechanism without reintroducing `set_expected_value`.

* **SQL injection on MySQL/MariaDB via raw-SQL value interpolation (high):** admin list filters, search (`search_cond!`), bulk `group_set`, and the m2m option query built conditions with `Expr::cust(format!("... = '{}'", val))` escaped only by doubling single quotes (`'` → `''`). This is sufficient on PostgreSQL/SQLite (`standard_conforming_strings`) but bypassable on MySQL/MariaDB, where a backslash escapes the following quote (`\'` followed by `''` breaks out of the string literal). An authenticated staff user with read access could execute arbitrary SQL. Fixed: all attacker-controlled values are now bound parameters via `Expr::cust_with_values(..., [val])`, delegating escaping to sea-query's backend-aware layer. Column identifiers stay inline but remain whitelisted (`FILTER_COLS` / `SORT_COLS`) or schema-fixed.

* **SQL injection in built-in admin resources (high):** the hand-written `list_fn` of the built-in `users`, `groupes`, and `droits` resources interpolated the filter column name (`?filter_<col>=`) directly into `CAST({col} AS TEXT)` with **no** whitelist — an identifier injection exploitable on every backend, not just MySQL — in addition to the unsafe value escaping. Fixed: column names are validated against an `[A-Za-z0-9_]` charset before use, and values are bound via `cust_with_values`.

* **Stored XSS via the `| markdown` filter (high):** the template preprocessor rewrites every `{{ x | markdown }}` to `{{ x | markdown | safe }}`, and the filter emitted `pulldown-cmark` output without sanitization. Raw inline HTML (`<script>`, `onerror=`) and `javascript:` link/image URLs passed through unescaped, making any user-authored Markdown a stored-XSS vector. Fixed: the filter now runs its output through a new `sanitize_markdown()` (ammonia) — http/https/mailto schemes only, no `style` attribute, raw HTML stripped, `rel="noopener noreferrer"` on links. The shared `ALLOWED_TAGS` / `ALLOWED_ATTRS` whitelist was widened (h1–h6, tables, `del`/`s`/`sub`/`sup`, `hr`, `img`, `code[class]`) to cover Markdown output without enabling any script-bearing element.

* **Open-redirect filter bypass via backslash (medium):** `is_safe_redirect` treated `/\evil.com` as a safe relative path (`starts_with('/')` but not `"//"`). Browsers normalize `\` to `/`, turning it into the protocol-relative `//evil.com`. Fixed: backslashes are normalized to forward slashes before the same-origin determination.

* **IP spoofing via `X-Forwarded-For` in standalone-TLS mode (medium):** the built-in TLS server (`axum_server::bind_rustls`, used for ACME / standalone HTTPS) served the router via `into_make_service()` without connect-info. With no `ConnectInfo<SocketAddr>`, `trusted_proxies` saw `conn_ip = None`, defaulted to loopback (a trusted CIDR), and therefore honored the client-controlled `X-Forwarded-For` header — letting any client forge its IP (rate-limit bypass, forged audit logs). Fixed on three layers: (1) the TLS serve path now uses `into_make_service_with_connect_info::<SocketAddr>()`, exposing the real peer IP; (2) `extract_client_ip` returns loopback without ever reading `X-Forwarded-For` when the peer IP is unknown, so a missing connect-info can no longer enable spoofing; (3) IPv4-mapped IPv6 peers and XFF entries (`::ffff:a.b.c.d`, seen on dual-stack sockets) are canonicalized to IPv4 before the trusted-CIDR check, so a private reverse proxy is correctly recognized. Covered by unit tests in `trusted_proxies.rs`.

* **`is_authenticated` deserialized the user id as `i32` (low):** under the `big-pk` feature (`i64`), a user id exceeding `i32::MAX` failed to deserialize, so `is_authenticated` returned `false` inconsistently with `get_user_id` (which uses `Pk`). Fixed: it now reads `Pk`.

---

## [2.1.10] - 2026-05-30

### Fix — `runique` (admin)

* **Edit and delete blocked for all resources without `own_field` (critical regression):** operator precedence in the ownership check produced `(action == "edit" && !can_update && !can_update_own) || !check_owns_record(...)`. Because `check_owns_record` returns `false` when `own_field` is not declared, `!check_owns_record()` was always `true`, causing every edit and delete request to return "permission denied" regardless of the user's actual rights. Fixed: the condition is now `!can_update && !(can_update_own && check_owns_record(...))`, applied separately in `admin_get_id` and `admin_post_id`.

---

## [2.1.9] - 2026-05-28

### Security — `runique` (admin, auth)

* **SQL injection in admin list filters (high):** the column name from URL parameters (`?filter_<col>=val`) was interpolated directly into `Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, ...))` without any validation. An authenticated staff user with minimal view rights could execute arbitrary SQL against the database. Fixed: the generator now emits two static whitelists (`SORT_COLS`, `FILTER_COLS`) built at code generation time from the declared `list_display` and `list_filter` columns. Any column name not in the whitelist is silently discarded before reaching the query.

* **Session fixation on login (medium):** `login()` did not call `session.cycle_id()` on privilege elevation (anonymous → authenticated). An attacker who planted a session ID in the victim's browser before login could reuse it after authentication. Fixed: `session.cycle_id().await` is now called on every privilege elevation (new session or user switch). Mitigated in practice by `SameSite=Strict` + `HttpOnly` cookie attributes, but the standard mitigation was absent.

* **Admin write access granularity (medium):** `check_write_access` returned `true` if any of `can_create`, `can_update`, or `can_delete` was set. A staff user granted only `can_create` could also edit and delete any record. Fixed: three separate guards (`check_can_create`, `check_can_update`, `check_can_delete`) are now applied per operation and per HTTP method. Bulk POST actions are also gated per action type (`delete` → `can_delete`, others → `can_update`).

* **IDOR — `can_update_own` / `can_delete_own` not enforced (low):** the "own" permission flags existed in the permission model and were injected into templates, but the CRUD closures `(db, id)` / `(db, id, data)` carried no user identity, making ownership verification structurally impossible. Edit and delete routes silently fell back to allowing any record. Fixed: a new `own_field: "field_name"` DSL option declares the JSON field used for ownership comparison. When a user has `can_update_own` (without `can_update`), the handler fetches the record via `get_fn` and compares `record[own_field]` against `current_user.id`. If `own_field` is not declared, "own" permissions are blocked by default (safe fallback).

### Added — `runique` (forms, debug)

* **`eprintln!` debug output for the full form processing pipeline:** when `DEBUG=true` and the corresponding `FormTracing` field is configured, each stage now emits both a `tracing` structured event (filtered by subscriber level) and an `eprintln!` directly to stderr (bypasses the subscriber filter). Stages covered: field registration, `set_value` per field (POST), checkbox normalization, validate per field, validate result, finalize per field, render per field.

### Added — `runique` (admin DSL)

* **`own_field` in `admin!{}`:** new optional DSL key that declares the record ownership field for `can_update_own` / `can_delete_own` enforcement. Example: `own_field: "user_id"`.

### Security — `runique` (forms)

* **`save()` / `save_as()` guard against skipped validation (low):** a developer could call `form.save()` without a prior successful `is_valid()` call, bypassing field validation, CSRF token verification, and `clean()` business rules entirely. Fixed: both methods now return `Err(DbErr::Custom(...))` immediately if `is_valid()` was not called or returned `false`. The check is performed via the internal `is_save_allowed()` method (`!force_invalid && validated && !has_errors()`). A `#[doc(hidden)]` `Forms::mark_validated()` helper is provided for tests that verify save/hook behavior in isolation.

---

## [2.1.8] - 2026-05-28

### Fixed — `runique` (admin, bulk)

* **`bulk_create` violated UNIQUE constraint on re-submit:** the generated `create_fn` performed a plain INSERT per value. Re-submitting the same days caused a UNIQUE violation that stopped the loop. The generator now emits an upsert: for each value, it checks whether a record with that value already exists (via `Expr::cust(format!("CAST({field} AS TEXT) = '{}'"...))`), then updates if found or inserts if not.
* **Edit view used multi-select form when `bulk_create` was declared:** when `bulk_create` is declared without an explicit `edit_form`, the daemon now auto-generates an `edit_form_builder` using `module::AdminForm` (standard single-record form). The individual edit view no longer uses the multi-select create form.
* **Unique fields appeared in bulk edit form:** bulk editing a resource with UNIQUE-constrained fields could produce a UNIQUE violation when the same value was applied to multiple records. The generator now emits `UNIQUE_FIELDS` per entity (from `derive_form!{}` `unique` constraints). These fields are automatically excluded from the bulk edit form (both GET rendering and POST update map).

### Added — `runique` (middleware)

* **Anti-bot honeypot middleware:** `AntiBot::new("field_name")` injects a hidden trap field into all forms on the protected scope. If the field is filled on POST, `form.is_valid()` returns `false` immediately without running field-level validation.
* **`RateLimiter` per-method configuration:** `rate_limit_get()`, `rate_limit_post()`, `rate_limit_put()`, `rate_limit_delete()` allow setting independent limits per HTTP method in addition to the global `rate_limit()`.

### Added — `runique` (forms)

* **`FormTracing` structured tracing for all form pipeline stages:** when `RuniqueLog::forms` is configured, each stage (field registration, `set_value`, validate, finalize, render) emits a structured `tracing` event at the configured level.
* **`cleaned_enum<T>()` on `RuniqueForm`:** reads a validated field value and tries to convert it to a SeaORM `ActiveEnum`.
* **`add_value()` on `RuniqueForm`:** forces a value on a named field, bypassing `fill()`. Useful for fields skipped by the form pipeline (e.g. password hash pre-computed before form processing).

---

## [2.1.6] - 2026-05-23

### Added — `derive_form` (extend)

* **`extend!{}` block in `derive_form!{}`:** a new `extend { Table { fields: { ... } } }` block allows adding custom columns to framework tables (e.g. `eihwaz_users`) using the same field DSL as `derive_form!{}`. The macro generates the `ALTER TABLE` migration, injects the columns into the existing SeaORM entity, and produces an `AdminForm` for use in `admin!{}`. The base table columns remain invisible to the user — only the declared extensions are surfaced.

### Added — `runique` (admin)

* **Structured tracing in admin CRUD operations and all party to of framework:** `handle_create_post` and `handle_edit_post` now emit structured log events controlled by `RuniqueLog::admin.crud`. Events cover form validation outcome, successful save, and database errors (unique violations distinguished from other errors).

### Fixed — `runique` (migrations)

* **`EihwazSessionsMigration::down()` failed with "no such table: eihwaz_sessions":** `AdminTableMigration::down()` already drops `eihwaz_sessions` (with `.if_exists()`). When `migrate reset` ran all DOWN migrations in reverse, `AdminTableMigration::down()` executed first, leaving the table gone. `EihwazSessionsMigration::down()` then tried to drop it again without `.if_exists()` and crashed. Fixed by adding `.if_exists()` to `EihwazSessionsMigration::down()`.

---

## [2.1.5] - 2026-05-20

### Fixed — `runique` (forms)

* **`parse_constraint_name` extracted table-name segments as field names for multi-word tables:** for a table named `changelog_entry`, the primary key constraint `changelog_entry_pkey` was split into `["changelog", "entry", "pkey"]` and the middle part `"entry"` was returned as a field name, producing a spurious "Field 'entry' value is already taken" error on every INSERT. Constraints ending in `_pkey` or `_fkey` now return `None` immediately, so primary key and foreign key violations fall through to the generic error message instead.

### Fixed — `runique` (admin)

* **Admin sidebar filters were not cumulative:** clicking a filter value in one column silently discarded active filters from other columns, because each filter link only included its own `filter_col=val` parameter. Links in `list_partial.html` now iterate over `active_filters` and preserve every other active column filter in the generated URL, both for value selection and for the per-column clear (✕) link.

---

## [2.1.4] - 2026-05-20

### Fixed — `runique` (admin daemon)

* **Admin generator emitted hardcoded `i32`/`i64` for PK parsing:** the `detect_big_pk` approach read the project's `Cargo.toml` to decide the parse type, but failed when `cargo clippy --all-features` was used on the workspace (workspace-wide feature activation made `Pk = i64` even for projects without `big-pk` in their own `Cargo.toml`). The generator now emits `parse::<Pk>()` by default, which resolves to the correct type at compile time via the `Pk` type alias. Explicit `id_type: I32 | I64 | Uuid` overrides still emit concrete types.

---

## [2.1.3] - 2026-05-20

### Fixed — `runique` (file uploads)

* **`parse_multipart` created upload directories for all multipart requests:** `create_dir_all` was called unconditionally at the start of `parse_multipart`, causing a crash in production on any form POST when `MEDIA_ROOT` was not set — even for forms with no file fields. Upload directories are now created lazily, only when an actual file part is encountered.
* **`resolve_media_root()` defaulted to relative `"media"` string:** the fallback was a bare relative path, making the effective directory unpredictable depending on the process working directory. The resolution now follows a three-level priority chain: `MEDIA_ROOT` env var → `{BASE_DIR}/media` → `{cwd}/media`, anchoring the path to the project root in all environments.

### Fixed — `runique` (admin daemon)

* **Admin generator used `i32` for all entity PKs regardless of `big-pk` feature:** the daemon always emitted `id.parse::<i32>()` in generated handlers. When a project enables the `big-pk` feature (making `pk: id => Pk` generate `i64`), the generated `admin.rs` failed to compile with type mismatch errors. The daemon now reads the project's `Cargo.toml` at startup — if `big-pk` is present in the features, the default id type is `i64`; otherwise `i32`. An explicit `id_type: I32 | I64 | Uuid` in `admin!{}` always takes precedence.

### Fixed — `runique` (makemigrations)

* **No destructive-change prompt before generating migrations:** `makemigrations` silently generated DROP COLUMN, type changes, nullable→NOT NULL alterations, dropped foreign keys, and CASCADE foreign keys without warning. A `collect_destructive_messages()` function now inspects all pending changes and, if any are destructive, prints a summary and prompts for confirmation (bypassed by `--force`).

---

## [2.1.2] - 2026-05-17

### Fixed — `runique` (migration utils)

* **`unique_together` generates `.unique_key()` — not found on `IndexCreateStatement`:** sea-query rc.27+ renamed `IndexCreateStatement::unique_key()` to `unique()`. The call in `generators.rs` is updated; `.unique_key()` on `ColumnDef` is unaffected.
* **Enum tuple syntax `Variant = ("db_value", "Display")` ignored in migrations:** `parser_builder.rs` only handled `syn::Lit` directly after `=`. When the value was a tuple `(...)`, parsing failed and fell back to the Rust variant name (e.g. `'Entree'` instead of `'entree'`), causing SeaORM deserialization failures. Fixed with a `parenthesized!` branch that extracts the first string from the tuple.

### Fixed — `runique` (admin prefix)

* **Admin middleware redirected to `/` on unauthenticated access:** now redirects to `{prefix}/login` using the configured prefix from `AdminState`. Unmatched routes pass through without triggering the redirect.
* **`admin_prefix` missing from all admin template contexts:** `inject_admin_prefix` was not called in `inject_context` (shared handler entry point), causing `Variable admin_prefix not found` in templates. Now injected centrally so every admin view has access to it.
* **`AdminRoutes` struct added:** `admins::routes(prefix)` now returns `AdminRoutes { router, prefix }` instead of a bare `axum::Router`, so the staging layer can propagate the prefix to `AdminConfig` automatically without a separate `.prefix()` call.
* **`list_filter` in `configure {}` for builtin resources:** sidebar filters declared via `configure { users: { list_filter: [...] } }` were silently ignored — the generator didn't pass them to `DisplayConfig`. The generator now includes the `list_filter` chain in the `configure` call, consistent with resource-level declarations.

### Fixed — `derive_form` 2.0.3

* **Time/Date/Datetime fields not saved in `partial_update`:** a `return None` arm at the top of the match in `generate_partial_update` was silently discarding all temporal fields before reaching the correct chrono-parsing arms added in 2.0.2 — those arms were unreachable dead code. The blocking arm is removed; `NaiveTime`, `NaiveDate`, `NaiveDateTime`, and `DateTime<Utc>` now persist correctly via `admin_partial_update`.
* **`auto_now`/`auto_now_update` fields absent from `Column` enum and `Model` struct:** the filter in `generate_sea_model` excluded these fields from both `ActiveModel` and `Column`, making `Entity::Column::CreatedAt` unavailable for sorting or filtering. The filter is removed; `auto_now` fields now appear in `Model` and `Column` as `Option<T>` and remain excluded only from `ActiveModel` to prevent manual overwrites.

### Added — `runique` 2.1.2

* **CORS support:** new `with_cors(|c| c.origin("https://app.example.com").allow_credentials(true))` on `MiddlewareStaging`. `CorsConfig` accepts `.origin()`, `.any_origin()`, `.allow_credentials()`, `.max_age()`. Wildcard origin combined with `allow_credentials(true)` is rejected at build time with a `BuildError`.
* **Trusted proxies:** new `with_trusted_proxies(|t| t.private_networks().proxy("203.0.113.5"))` middleware. Validates `X-Forwarded-For` chains and injects `ClientIp` into handler extensions. Defaults to RFC 1918 + loopback — covers nginx on the same machine and Docker networks without configuration. `.none()` clears all trust for direct-exposure deployments.
* **`Permissions-Policy` header:** new `with_permissions_policy(|p| ...)` middleware. Sends the `Permissions-Policy` header; all sensors, hardware APIs, and payment are denied by default. Individual directives can be overridden via the builder.
* **Open redirect protection:** automatic middleware on all 3xx responses. `Location` headers pointing to external origins are blocked unless the destination is in the configured allowed hosts list. Stops unintentional redirects introduced by handler logic.
* **`RuniqueAppBuilder::with_custom_db`:** attaches any `Any + Send + Sync + 'static` value as an Axum extension, making secondary connections (Redis pools, alternate databases) available in handlers via `Extension<T>`.
* **`EihwazSessionsMigration` included in `AdminTableMigration`:** `create_eihwaz_sessions_table()` is now called inside `AdminTableMigration::up()` (between `eihwaz_users_groupes` and `eihwaz_history`). The corresponding `DROP` is added to `down()`. New projects no longer need to add this migration manually.
* **`makemigrations` injects `EihwazSessionsMigration`:** `ensure_admin_migration_positioned()` now inserts `Box::new(migrations_table::EihwazSessionsMigration)` between `EihwazUsersMigration` and `AdminTableMigration` in the generated `lib.rs`. The duplicate-filter and `FRAMEWORK_TABLE_PATTERNS` are updated accordingly.
* **Admin login — `admin_prefix` injected in all error paths:** `inject_admin_prefix` was missing from the four error render paths in `admin_login_post` (CSRF invalid, account locked, session error, wrong credentials), causing a `Variable admin_prefix not found` 500 error on failed logins. Fixed in all four paths.
* **Admin bulk JS — checkboxes rebound after HTMX swap:** `admin-bulk.js` now listens to `htmx:afterSwap` on `#list-content` and re-attaches all checkbox listeners (`#bulk-check-all` and `.bulk-check`). Previously, pagination and filter navigation via HTMX recreated DOM elements without event listeners, breaking the select-all checkbox.
* **Admin bulk edit:** new `GET /{resource}/bulk_edit` and `POST /{resource}/bulk_edit` handlers. When IDs are selected in the list view and the bulk-edit action is triggered, a form is rendered with the shared fields editable. On submit, each record is updated independently; unique-constraint violations are skipped with a warning rather than aborting the whole batch.
* **M2M support in admin DSL:** `m2m: [["field", "Label", "junction_table", "self_fk", "target_fk", "entity::path"]]` in `admin!{}` generates a `M2mLoaderFn` closure. In create/edit forms, all available choices are loaded from the target table and pre-selected IDs are read from the junction table. Submitted values (prefixed `m2m_field__`) are diffed against the current state; only inserts and deletes are applied.
* **`AdminConfig::extra_routes()`:** `.with_admin(|a| a.extra_routes(vec![("/path", get(handler))]))` attaches custom routes inside the admin prefix without needing a separate `merge()` call on the router.
* **`Request` query/path helpers:** four new methods on `runique::context::Request`:
  * `get_path(key) -> Option<&str>` — raw path parameter.
  * `get_path_as::<T>(key) -> Option<T>` — typed path parameter (parses via `FromStr`).
  * `get_query(key) -> Option<&str>` — raw query string parameter (replaces `from_url`).
  * `query::<T>() -> T` — deserializes the full query string into a struct via `serde_qs`; `raw_query` is now stored on `Request` at extraction time.
* **DSL `bulk_create: field` — multi-record creation from a single form submit:** when `bulk_create: field_name` is declared on a resource in `admin!{}`, the generated `create_fn` splits `data[field_name]` by comma and inserts one record per value. Designed for `CheckboxField` multi-select (e.g. selecting multiple days of the week to create one `horaire` row per day).
* **FK resolution in `list_display` — optional 3rd element `"table.column"`:** declaring `["col", "Label", "table.column"]` in `list_display` resolves the raw FK id to a human-readable label in the list view. A `SELECT CAST(id AS TEXT), column FROM table WHERE id IN (...)` query runs after the main fetch and replaces each id in-place. Compatible with `i32`, `i64` and UUID. FK columns are automatically excluded from full-text search.
* **FK select in admin create/edit forms:** when a `list_display` entry has a 3rd FK element, the generated `form_builder` closure loads all rows from the related table and injects a `<select>` dropdown (via `Forms::field_choices`) for that field, with the existing value pre-selected in edit mode.
* **`Forms::field_choices` added:** new method on `Forms` that replaces a field by name with a `ChoiceField` populated from a `Vec<(String, String)>` of `(value, label)` pairs. Preserves the current value and the required flag.
* **History pagination uses `AdminConfig::page_size`:** the two history handlers (`/admin/history` and per-object history) previously used a hardcoded `PER_PAGE = 50`. They now read `admin.config.page_size`, controlled via `.with_admin(|a| a.page_size(N))` in the builder.
* **`GroupAction::val(field, label, value)` — fixed-value group action:** new constructor for enum-type fields. The 3-element DSL syntax `["field", "Label", "value"]` generates `GroupAction::val` instead of `GroupAction::bool`, submitting the exact string value (e.g. `"valide"`) rather than `"true"`/`"false"`.
* **`with_group_actions` merges same-field actions:** multiple `GroupAction` entries targeting the same field are merged into a single `<select>` with all choices combined. Previously, duplicate `name="ga_*"` selects caused the last (empty) value to overwrite the selected one, silently discarding the update.
* **`RuniqueQueryBuilder::order_by_random()`:** orders results by `RANDOM()` without raw SQL.
* **`RuniqueQueryBuilder::order_by_expr(expr, order)`:** orders by an arbitrary SeaORM `IntoSimpleExpr` expression.
* **`RuniqueQueryBuilder::one()`:** returns `Result<Option<E::Model>, DbErr>`. Returns `Err` if more than one row matches — analogous to Django's `.get()`. Fetches at most 2 rows internally to detect the ambiguous case without a full scan.
* **`Request::headers`:** HTTP request headers (`axum::http::HeaderMap`) now available on `Request` in all handlers.
* **`PasswordResetConfig::email_template(path)`:** optional custom Tera template for password reset emails; falls back to the built-in template if not set.
* **Translation placeholders unified:** all language files (`fr`, `en`, `de`, `es`, `it`, `ja`, `pt`, `ru`, `zh`) migrated from positional `{0}`/`{1}`/`{2}` to anonymous `{}` to match the Rust `format!` convention used at runtime.

### Added — `derive_form` 2.0.3

* **`extend!{}` macro — extend framework tables:** generates a `schema()` function that `makemigrations` uses to emit `ALTER TABLE ADD COLUMN` statements for the named framework table. Only allowed on built-in tables (`eihwaz_users`, `eihwaz_groupes`, `eihwaz_droits`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`). Other names are rejected at compile time.
* **`phone` field type:** `phone: phone [required]` in `model!{}` — stored as VARCHAR, rendered as `<input type="tel">` in forms.

---

## [2.1.1] - 2026-05-02

### Fixed — `derive_form` 2.0.2

* **`fk()` in v2 blocks silently ignored:** `FormFieldAttr::Fk(FkDef)` added to AST, parser, and propagation to `FieldOption::Fk`.
* **`skip` attribute unknown:** `FormFieldAttr::Skip` added to AST, parser, and generator (field excluded from form rendering).
* **`many_to_many(target).through(via)` syntax broken:** corrected to `many_to_many(target, via)` in `foreignkey.rs`.
* **`sea_query::ForeignKeyAction` not found:** re-exported as `runique::migration::ForeignKeyAction`; generator paths updated.
* **`.references_column()` method missing:** replaced with `.to_column()` in FK builder.
* **PascalCase model names in relation paths:** `to_snake_case()` used consistently instead of `.to_lowercase()` across `relation_enum.rs` and `foreignkey.rs` (e.g. `super::menuimage` → `super::menu_image`).
* **`rust_decimal::Decimal` not found:** type mapped to `::runique::sea_orm::prelude::Decimal` in `sea_model.rs`.
* **`via_self` FK column → wrong relation variant:** `_id` suffix stripped and PascalCase applied to derive the correct variant name in `ManyToMany` `Related` impl.
* **`Decimal` missing from `generate_partial_update`:** `FieldType::Decimal(_)` added to the numeric arm.
* **`Decimal` missing from `generate_from_str_map`:** `FieldType::Decimal(_)` added to the float/decimal arm.
* **`unique_together` / `indexes` never generated in SQL:** `parser_builder.rs` was silently ignoring the `meta` block. Now parsed and converted to `ParsedIndex` entries (`{table}_{cols}_uniq` for unique constraints, `idx_{table}_{cols}` for plain indexes).

### Added — `runique` 2.1.1-alpha.3

* **`OrderDir` enum** added to `migration::schema` (`Asc` / `Desc`).
* **`ModelSchema` builder methods:** `order_by()`, `unique_together()`, `verbose_name()`, `verbose_name_plural()`.
* **`ForeignKeyAction` re-exported** from `runique::migration`.
* **`RelationDef::as_name()`** no-op method added for DSL compatibility.

---

## [2.1.0] - 2026-04-20

### Breaking

* **`Prisme<T>` removed — form extraction via `req.form::<T>()`:**
  Handler parameters no longer accept `Prisme<MyForm>`. Use `let form = req.form::<MyForm>()` instead.
  `Request` must be the **last parameter** of every handler (body-consuming extractor).
  `AdminBody` extractor removed — admin POST handlers read form data from `req.prisme.data`.

### Added

* **`EihwazSessionsMigration` — persistent session table:**
  `migrations_table::EihwazSessionsMigration` creates the `eihwaz_sessions` table.
  To be added to the `Migrator` vec after `EihwazUsersMigration`.
  `eihwaz_sessions` is now listed in `FRAMEWORK_TABLES` and excluded from `makemigrations` scan.

### Fixed

* **`auth_login` — sessions now persisted in DB:**
  `auth_login()` now passes a `RuniqueSessionStore` to `login()`, ensuring a row is created
  in `eihwaz_sessions` at login. Sessions survive server restarts via the DB fallback.

---
