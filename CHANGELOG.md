🌍 **Languages**:[English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Changelog

All notable changes to this project will be documented in this file.

---

## [2.1.17] - 2026-06-15

### Fix — `runique` (CLI `makemigrations` — enum column defaults)

* **`[default: …]` was still dropped on enum columns:** the 2.1.15 fix made `render_column_def` emit `.default(<value>)`, but only on the non-enum branch — the `ColumnType::Enum` branch was rendered with `{null}{uniq}` and never appended `{default}`. A `choice [enum(X)]` column with a `[default: "Y"]` therefore lost its default, and a `required` (NOT NULL) enum column added to a populated table failed at migration time with `column "…" contains null values`. Fixed: the enum branch of `render_column_def` now appends `{default}` like every other column, so an `ADD COLUMN <enum> NOT NULL DEFAULT '<variant>'` lets Postgres backfill existing rows instead of erroring. The emission is engine-independent (Postgres/MySQL/SQLite); the Postgres-only `CREATE TYPE` gating is unchanged.

* **3 pipeline tests added** (`tests_pipeline.rs`, now 14): the enum default reaches the parsed `ParsedColumn`, is emitted in CREATE across all three engines, and is emitted on the `extend!{}` ADD COLUMN path across all three engines (with `CREATE TYPE` still Postgres-only).

### Fix — `runique` (admin display of rich-text fields + two template filters)

* **Rich-text field values were double-escaped in the admin detail/list views:** a `richtext` field is sanitized at write time by ammonia, which normalizes a stray `>` in text to the entity `&gt;` (valid HTML). The admin detail and list templates rendered the stored value through `{{ value }}` (auto-escaped), so the already-encoded `&gt;` was escaped again to `&amp;gt;` and shown literally as `&gt;` on screen. Fixed without weakening security, via **output-side** sanitization: the admin now knows which columns are rich (`RICH_CONTENT_FIELDS`, the same classification used on write, injected as `rich_fields`), and renders them through new filters instead of trusting storage.

* **New `| sanitize` filter (rich HTML, output-sanitized):** re-runs `sanitize_rich` (ammonia) on the value **at render time**, and the template preprocessor forces `| safe` on every `| sanitize` (mirroring `| markdown`). The `| safe` therefore only ever emits ammonia's own XSS-free output, re-cleaned regardless of how the value reached the database — sanitize-on-output, not trust-on-input. Used by the admin detail view to render rich fields as real HTML.

* **New `| plaintext` filter (text preview):** projects a value to plain text via `sanitize_strict` — strips every tag and decodes entities, so a stored `&gt;` becomes a real `>` that Tera then escapes once. No `| safe` is forced (the output is plain text and stays auto-escaped). Used by the admin list cells, where rendering block-level rich HTML would break the truncated single-line layout.

---

## [2.1.16] - 2026-06-15

### Chore — `runique` (dependencies, toolchain)

* **SeaORM bumped to `2.0.0-rc.40` and MSRV raised to Rust 1.94:** updated `sea-orm` / `sea-orm-migration` from `rc.38` to the pinned `=2.0.0-rc.40`. The new release candidate raises its minimum supported Rust version, so the workspace `rust-version` is bumped to `1.94` accordingly.

---

## [2.1.15] - 2026-06-13

### Feature — `runique` (routing, templates)

* **Named URLs for built-in routes (`forgot_password`, `reset_password`, `admin`):** framework-registered routes (password reset and admin panel) were mounted directly via Axum `Router::route()` without being registered in the URL name registry. They could not be referenced via `{% link %}` in templates. Fixed: `build.rs` now calls `register_name_url` after mounting password-reset routes (`"forgot_password"` → configured `forgot_route`, `"reset_password"` → configured `reset_route` with `{token}/{encrypted_email}` placeholders) and after mounting the admin panel (`"admin"` → the configured prefix). The registration picks up any custom routes set by the developer via `PasswordResetConfig::forgot_route()` / `.reset_route()` / `AdminConfig::prefix()`.

### Security — `runique` (auth)

* **User enumeration via timing attack on login (medium):** `authenticate_user` and `DefaultAdminAuth::authenticate` returned `None` immediately via `?` when the username was not found in the database, skipping the password hash verification entirely. An attacker measuring response times could distinguish "user does not exist" (fast — no hash work) from "wrong password" (slow — full Argon2 verification), allowing silent username enumeration. Fixed: both functions now always call `verify()` before short-circuiting. When the user is not found, the password is verified against a pre-computed dummy Argon2 hash (`DUMMY_HASH`, initialised once at first use via `LazyLock`) — burning the same CPU time regardless of whether the account exists. The `?` on the user lookup is deferred to after `verify()` returns, so the result is always discarded once timing-sensitive work is done.

### Security — `runique` (admin)

* **Missing authorization on the admin `reset-password` action (medium):** in `admin_post_id` (`admin/admin_main/mod.rs`), the `edit` and `delete` actions were gated by a permission check (`can_update` / `can_delete`, with `_own` + ownership fallback), but the `reset-password` action had no authorization gate at all — and, unlike `admin_get_id`, this handler does not check `can_access` either. Any user authenticated to the admin panel (`is_staff` or `is_superuser`) with **zero permissions** on the resource could `POST {prefix}/{resource}/{id}/reset-password` to trigger a password-reset for any record, including superuser accounts. CSRF and authentication were enforced, but not authorization. Aggravating factor: when no mailer is configured, the reset link (token + encrypted email) is returned to the calling admin in a flash message — a low-privilege staff user could obtain a working reset link for a higher-privileged account. Fixed: `reset-password` now requires `can_update` (global, or `can_update_own` on an owned record), aligned with `edit`.

### Security — `runique` (admin generator — SQL hardening)

* **Manual SQL escaping in foreign-key label resolution:** the admin generator (`admin/daemon/generator.rs`) built the FK label lookup queries (`list_fn` and `get_fn`) by string-concatenating an `IN (...)` clause with `Expr::cust(format!("CAST(id AS TEXT) IN ({})", ids_csv))`, where `ids_csv` was assembled via `format!("'{}'", s.replace('\'', "''"))`. Escaping only single quotes is insufficient on MySQL/MariaDB, where the backslash is an escape character by default — a value containing a backslash could break out of the literal. In practice the values are DB-stored FK ids (integers), so the exploitability was negligible, but it was manual escaping where the rest of the generator already uses whitelisted identifiers and bound values. Fixed: both queries now use bound values via the typed sea-query API — `Expr::cust("CAST(id AS TEXT)").is_in(fk_ids.clone())` in `list_fn` and `.eq(fk_key.clone())` in `get_fn`. No data is interpolated into the SQL string anymore; the placeholder is backend-correct on PostgreSQL/MySQL/SQLite.

### Fix — `runique` (CLI `makemigrations` — refactor for solid, reliable use)

* **Column `[default: …]` values were parsed but never emitted:** `ParsedColumn` only carried a `has_default_now` flag (for `CURRENT_TIMESTAMP`); the literal value of `[default: 0]` / `[default: true]` / `[default: "x"]` was consumed by the parser and discarded, so no `.default(...)` ever reached the generated SQL — only auto timestamps got a default. Worst case: a `bool [default: true]` rendered as `NOT NULL` *without* a default, an `ADD COLUMN` that fails on a populated table. Fixed: `ParsedColumn` gains a `default_value: Option<String>`; `parser_builder` and `parser_extend` now capture the literal, `render_column_def` emits `.default(<value>)` (covering CREATE, snapshot and ALTER). The snapshot round-trip is preserved — `parser_seaorm` only treats `.default(Expr::current_timestamp())` as a timestamp default and reads literal defaults back into `default_value`.

* **`bool` was not-null by default (inconsistent with other scalars):** `bool`/`boolean` were missing from the v2 semantic-type list, so a boolean field without `required` was generated `NOT NULL` (v1 behaviour) while `int`, `text`, etc. were nullable. Combined with the dropped default above, a `bool [default: true]` produced an unrunnable `ADD COLUMN NOT NULL`. Fixed: `bool`/`boolean` added to the v2 type set — nullable unless `required`, consistent with the other scalar types.

* **CASCADE foreign keys on brand-new tables flagged as destructive:** the destructive guard reported `ADD FOREIGN KEY … ON DELETE CASCADE (existing rows may be deleted)` for every CASCADE FK, including tables created in the same batch — which have no rows to lose. A first-time / from-scratch generation of any schema with CASCADE FKs wrongly required `--force`. Fixed: the CASCADE check now skips new tables (`is_new_table`), since only an existing populated table carries the risk.

* **Pure pipeline test suite added:** `migration/utils/tests_pipeline.rs` — 11 dependency-free tests (no DB/Docker) covering default parsing & emission, bool nullability, the snapshot round-trip stability invariant ("generate twice = no changes"), the destructive guard (CASCADE new vs existing, DROP COLUMN), enum value additions, and Postgres-only `CREATE TYPE`/`DROP TYPE` gating.

* **Rollback deleted pre-existing snapshots instead of restoring them:** on a mid-batch write failure, the rollback ran `fs::remove_file` on every file already written, including `snapshots/{table}.rs`. For an ALTER on an existing table the snapshot already existed and was *overwritten* during the run, so deleting it lost the previous content — the next `makemigrations` would see no snapshot and regenerate a full `CREATE TABLE` for an already-migrated table. Fixed: the content of every existing target is backed up in memory before writing (like the existing `lib.rs` backup); on rollback each file is *restored* if it had a backup, otherwise removed.

* **Destructive guard didn't cover `extend!{}` blocks:** `check_destructive` only ran on the main model changes; the extend pass computed its own diffs and generated ALTER files directly, with no destructive check and ignoring `--force`. A DROP COLUMN / type change / nullable→required / dropped or CASCADE FK introduced via an `extend!{}` block was emitted silently. Fixed: extend changes are now included in a single destructive guard that honors the same `--force` flag as the main pass.

* **Non-atomic generation across passes (refactor to plan → validate → commit):** the command wrote in three independent passes (main models, then `extend!{}`, then `AdminTableMigration` positioning), each committing on its own. A failure in a later pass left the earlier ones committed. Reworked into a single flow: both the main and extend changes are planned in memory first (`Plan { files, dirs, lib_modules }`), validated by one destructive guard, then committed atomically by a single `commit_plan` — directory creation, target backups, file writes, `lib.rs` registration and admin-migration positioning all run under one rollback. As a side effect, a project containing *only* `extend!{}` blocks (no own models) now generates its migrations correctly — previously an early `return` on empty model changes skipped the extend pass entirely.

### Feature — `runique` / `derive_form` (DSL `model!` / `extend!`)

* **Enums in `extend!{}`:** `extend!{}` now accepts an optional `enums: { … }` block (between `table:` and `fields:`), identical to `model!`. A `choice [enum(EnumName)]` column generates the real Rust enum type (`DeriveActiveEnum`), maps the entity field to it (instead of `String`), parses it in `admin_from_form` / `admin_partial_update` via `FromStr`, and populates the admin `ChoiceField` with the variants. `makemigrations` emits the column (Postgres: `CREATE TYPE … AS ENUM`; other engines: native `VARCHAR`/`ENUM`). The shared `generate_enums` was factored into `generate_enum_defs(&[EnumDef])`, reused by both macros. Degrades cleanly: `choice [enum(X)]` without a declared `enums:` block keeps the old `String` behaviour.

* **Column rename via `[renamed_from: "old"]`:** renaming a field previously produced a `DROP COLUMN` + `ADD COLUMN` — silent data loss. The new explicit hint makes `makemigrations` emit `ALTER TABLE … RENAME COLUMN old TO new` (portable across PostgreSQL, MySQL/MariaDB and SQLite), with no data loss. It is a migration-only directive (no effect on the generated entity/form) and is tolerated by both DSL field parsers (`FormFieldDecl` for v2/`extend!`, `FieldDef` for v1). Guard against a stale hint: if the old column still exists in the snapshot, no rename is emitted (falls back to add). `ParsedColumn.renamed_from` is transient and never written to snapshots.

### Fix — `runique` (CLI `makemigrations` — cross-engine correctness)

* **Enum value rename produced broken SQL on Postgres:** a positional change of an enum value generated a data `UPDATE` *before* `ALTER TYPE … ADD VALUE`, plus a misleading "value removed" warning — unrunnable on a native PG enum (a value cannot be written before it exists, and a freshly added value cannot be used in the same transaction). A rename is now modelled as **one** operation: on Postgres it emits `ALTER TYPE … RENAME VALUE 'old' TO 'new'` (atomic, updates existing rows); on VARCHAR-backed engines it emits a plain data `UPDATE`. The renamed pair is excluded from the add/drop lists. Enum value add/drop DDL is now also gated to Postgres only (previously `ALTER TYPE` was emitted on MySQL/SQLite too — invalid SQL).

* **Foreign keys broke `migrate` on SQLite:** all FK constraints were grouped into a separate `create_relations` migration applied via `ALTER TABLE … ADD CONSTRAINT`. PostgreSQL and MySQL/MariaDB accept this, but SQLite cannot add a FK to an existing table — `migrate` panicked. Fixed: for SQLite (`DbKind::Other`), FKs are now declared **inline in the `CREATE TABLE`** (`build_create_table_cols` gains an `inline_fks` flag) and the `create_relations` migration is skipped. PostgreSQL/MariaDB are unchanged; the snapshot keeps FKs as separate statements (round-trip stable). Validated end-to-end (`fresh` + full `reset`) on all three engines via a multi-engine smoke script.

* **Pipeline test suite expanded (11 → 63):** `migration/utils/tests_pipeline.rs` now covers enums in `extend!`, enum value rename per engine, column rename (hint, portability, stale hint, `extend!`), primary-key types (i32/i64/uuid), per-engine timestamps, FK actions, `i32`-backed enums, the full destructive matrix, ALTER add/drop/modify column + FK + index, semantic type mapping, `Changes::is_empty`, the no-op diff, ignored (`readonly`) columns, topological FK ordering, and inline-FK-on-SQLite. All dependency-free (no DB/Docker).

### Fix — `runique` (forms, admin)

* **`add_js()` scripts blocked by CSP `strict-dynamic` in admin forms:** `render_js()` built its own isolated Tera context containing only `js_files`, with no access to the request-scoped `csp_nonce`. Under `strict-dynamic`, any `<script>` tag without the matching nonce is blocked by the browser. Fixed: `FormRenderer` gains a `csp_nonce: Option<String>` field and a `set_nonce()` method; `Forms` exposes `set_csp_nonce()`; the admin CRUD handlers (`handle_crud.rs`) inject the nonce from the request context into the form renderer before serialisation via a new `inject_csp_nonce()` helper, called at every `context.insert(FORM_FIELDS, …)` site (create GET/POST, edit GET/POST).

* **`TextField` double-encoded `&` when storing plain-text input:** `sanitize_strict()` used `ammonia::Builder::new().tags(HashSet::new()).clean(input)` to strip all HTML tags. This is correct, but ammonia always HTML-encodes `&` → `&amp;` in its output even when no tags remain — a side effect of its HTML-entity serialiser. The raw output was then stored in the database. When Tera later rendered it with autoescaping active, the `&` in `&amp;` was re-encoded to `&amp;amp;`, so the browser displayed the literal text `&amp;` instead of `&`. Fixed: after ammonia strips the tags, the result is decoded back to plain text via `html_escape::decode_html_entities()` — removing the spurious entity encoding before storage. Dangerous protocol stripping (`javascript:`, `vbscript:`, `data:`, `file:`) runs on the decoded string, preserving security guarantees. New dependency: `html-escape = "0.2"`.

---

## [2.1.14] - 2026-06-06

### Fix — `runique` (admin filters & search — PostgreSQL regression)

* **`?` placeholder in `cust_with_values` broke every admin filter and search on PostgreSQL (regression introduced by the 2.1.12 SQL-injection fix):** to bind values safely, 2.1.12 replaced inline interpolation with `Expr::cust_with_values(format!("CAST(col AS TEXT) = ?"), [val])`. The `?` marker is only the bind placeholder on MySQL/SQLite — PostgreSQL uses `$N`. In sea-query's `CustomWithExpr` renderer the placeholder is matched against the backend's own marker (`$` on Postgres), so an unrecognized `?` is emitted verbatim and the value is never bound. The condition collapsed to `WHERE CAST(col AS TEXT) = ` immediately followed by ` LIMIT`, producing a `syntax error at or near "LIMIT"` 500 on any filtered list or search. Completely silent on MySQL/SQLite. Fixed: all raw-SQL conditions are rebuilt with typed sea-query expressions that emit the backend-correct placeholder automatically — equality via `Expr::col(Alias::new(col)).cast_as(Alias::new("TEXT")).eq(val)`, case-insensitive search via `Expr::expr(Func::lower(Expr::col(...).cast_as(Alias::new("TEXT")))).like(pattern)` with the pattern lowered in Rust. Applied to the generated `list_fn` column filters, the `search_cond!` macro (both `all_columns` and `or(...)` arms), the bulk-upsert lookup and the m2m option query in the admin generator, and the builtin user / group / permission resources. The `search!` macro was unaffected — it already builds conditions through typed `ColumnTrait` methods.

* **Filter sidebar values sorted in Rust, descending:** the `DISTINCT` query no longer carries the `ORDER BY CAST(col AS TEXT) DESC` introduced in 2.1.13 (it was entangled with the broken raw-SQL path). Distinct values are now sorted inside the generated closure (`sort_by` — numeric when both sides parse as integers, lexicographic otherwise), descending — highest version / id and most recent value first in each dropdown. The main list ordering is untouched (still driven by the clickable column headers).

---

## [2.1.13] - 2026-06-01

### Fix — `runique` (admin templates)

* **Admin sort links lost active filters:** clicking a column header to sort reset all active `list_filter` values. The sort links in `list_partial.html` did not include `{{ filter_qs }}`, unlike the pagination links. Fixed: `filter_qs` is now appended to the `href` and `hx-get` of both the `id` column and dynamic column headers.

* **Admin filter sidebar values unordered:** generated `filter_fn` closures retrieved distinct values then re-sorted them in Rust with `sort_by` (numeric-then-lexicographic ascending), overriding any DB-side ordering. Fixed: the in-memory `sort_by` is removed; the `DISTINCT` query now uses `ORDER BY CAST(col AS TEXT) DESC` so values arrive pre-sorted from the DB — dates show newest first, string values show highest alphabetically first.

### Fix — `runique` (admin templates, admin CSS)

* **Double-escaping of string values in admin list and detail views:** `{{ value | escape }}` was used in `list_partial.html` and `detail.html` while Tera's autoescape was already active. The `escape` filter converts `/` to `&#x2F;`, then autoescape re-encodes `&` to `&amp;`, so the browser rendered the literal text `&#x2F;` instead of `/`. Same issue in the hidden filter inputs (`value="{{ val | escape }}"`), which would have caused filter comparisons to fail for values containing `/`. Fixed: all three occurrences replaced with plain `{{ value }}` / `{{ val }}` — autoescape alone is sufficient for XSS protection here.

* **Admin filter sidebar CSS used mismatched selectors:** the CSS section for the list filter panel used hyphen-style selectors (`.admin-filter-sidebar`, `.filter-group`, `.admin-list-layout`, etc.) while the templates had already been refactored to BEM (`.admin-filter__sidebar`, `.admin-filter__group`, `.admin-list__layout`, etc.). The filter panel had no effective styling. Fixed: the entire section is rewritten with BEM selectors; mobile now uses an offcanvas pattern (`position: fixed; right: -300px` → `.mobile-open { right: 0 }`) with a backdrop overlay.

### Fix — `runique` (templates)

* **Internal templates not autoescaped — XSS vector on admin form fields:** Tera's autoescape is activated for logical keys ending in `.html` or `.xml`. Internal framework templates were registered without the `.html` suffix, so their variables were rendered raw. In particular, `{{ form_fields.html }}` (which holds Runique-generated form HTML) was not autoescaped and would have been misinterpreted as a missing variable. Fixed: all internal template keys now include the `.html` suffix; the template preprocessor (`process_content`) rewrites `{{ form_fields.html }}` to `{{ form_fields.html | safe }}` via `ADMIN_FORM_HTML_REGEX` — the only variable exempt from autoescaping because it is always Runique-generated HTML, never user input. The `{% extends %}` keys in doc examples were updated accordingly (`"admin/admin_template.html"`, `"admin_base.html"`).

* **`resolve_og_image`: CSS version hash applied to media URLs, and potential double slash:** the function appended `?v=<hash>` to the OG image URL unconditionally when a CSS build token was present. The hash is computed from static assets (CSS/JS) at build time and has no relation to the media file's content — applying it to an uploaded image is semantically wrong and breaks scraper cache-busting when the image changes without a redeploy. Additionally, if an `allowed_hosts` entry contained a trailing slash, the host prefix (`https://host/`) concatenated with an `og_image` starting with `/` produced a double slash (`https://host//media/...`). Fixed: the `?v=` is removed entirely from OG image URLs; `trim_end_matches('/')` is applied to the host and `trim_start_matches('/')` to the image path before concatenation; absolute URLs (`http://` / `https://`) are returned early without modification.

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
