🌍 **Languages**: [English](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md) | [Français](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.fr.md)

# Changelog

All notable changes to this project will be documented in this file.

---

## [1.1.47] - 2026-03-15

### Breaking

* **CSP — env vars removed:** All `RUNIQUE_POLICY_CSP_*`, `RUNIQUE_ENABLE_CSP`, `RUNIQUE_ENABLE_HEADER_SECURITY`, `ENFORCE_HTTPS`, `RUNIQUE_POLICY_CSP_STRICT_NONCE` environment variables removed. CSP is now configured exclusively via the builder.
* **CSP — disabled by default:** `MiddlewareStaging::from_config()` no longer activates CSP automatically. Must be explicitly enabled via `.with_csp(...)`.
* **`SecurityPolicy::from_env()` removed:** Replaced by `SecurityPolicy::default()`. All call sites updated.
* **`builder.rs`:** Unused `SecurityPolicy` import removed.

### Security

* **CSRF middleware:** Mutating requests (POST/PUT/DELETE/PATCH) without `X-CSRF-Token` header and without a form `Content-Type` (`application/x-www-form-urlencoded` / `multipart/form-data`) are now rejected with 403. Previously they passed through silently.
* **CSRF token masking (BREACH protection):** `extractor.rs` (`build_with_data`) and `template.rs` (`form()`) now inject the **masked** (XOR + base64) token into form hidden fields instead of the raw HMAC-SHA256 hex. This ensures AJAX reads the correct masked value for the `X-CSRF-Token` header.
* **`csrf_gate.rs`:** Submitted form token is now **unmasked** before constant-time comparison against the raw session token — making the full mask/unmask roundtrip consistent end-to-end.
* **CSRF:** Eliminated `expect()` panic vector on malformed token — replaced with graceful `unwrap_or_else` fallback in `csrf.rs`.
* **CSRF:** `HeaderMap::contains_key("X-CSRF-Token")` confirmed case-insensitive — no bypass possible via header casing.
* **Lock safety:** `GLOBAL_LANG` (`RwLock<Lang>`) replaced with `AtomicU8` — lock poisoning impossible, no `unwrap()` required.
* **Lock safety:** `url_registry` and `PENDING_URLS` lock acquisitions now use `unwrap_or_else(|e| e.into_inner())` — survives poisoned mutex in any thread panic scenario.

### Fixed

* **CSRF brace bug (`csrf.rs`):** A misplaced `} else {` caused the `else` branch to belong to `if requires_csrf` instead of `if has_header`, returning "CSRF token required" on every GET request (all views broken). Restructured to correct scoping.

### Added

* **CSP builder API:** New closure-based sub-builder pattern — `.middleware(|m| m.with_csp(|c| c.method()))`.
* **`CspConfig` struct:** Standalone sub-builder with full directive control: `scripts()`, `styles()`, `images()`, `fonts()`, `connect()`, `objects()`, `media()`, `frames()`, `frame_ancestors()`, `base_uri()`, `form_action()`, `default_src()`.
* **`CspConfig` toggles:** `.with_nonce(bool)`, `.with_header_security(bool)`, `.with_upgrade_insecure(bool)`.
* **`CspConfig` presets:** `.policy(SecurityPolicy::strict())`, `.policy(SecurityPolicy::permissive())`.
* **`CspConfig` accessors:** `.get_policy() -> &SecurityPolicy` and `.header_security_enabled() -> bool` (used by tests).
* **`MiddlewareConfig`:** New `enable_header_security: bool` field — controls whether `security_headers_middleware` (HSTS, X-Frame-Options, COEP, COOP, CORP) is activated alongside CSP.
* **Rate limiter (`RateLimiter`):** sliding-window middleware by IP. Configurable request limit and time window. Returns HTTP 429 on excess.
* **Login guard (`LoginGuard`):** brute-force protection by username. Configurable attempt limit and lockout duration. Complementary to `RateLimiter` (IP vs. username).
* **Periodic cleanup (`spawn_cleanup`):** both `RateLimiter` and `LoginGuard` expose `spawn_cleanup(period)` — spawns a Tokio background task that purges expired entries on a configurable interval, mirroring `CleaningMemoryStore`.
* **429 template:** dedicated Tera template (`errors/429.html`) embedded in the binary, rendered by `error_handler_middleware` on `TOO_MANY_REQUESTS`. Includes inline HTML fallback if Tera render fails.
* **i18n — 429 keys:** `html.429_title` and `html.429_text` added to all 9 translation files (fr, en, de, es, it, pt, ja, zh, ru).
* **CLI — language:** application language now configurable via `RUNIQUE_LANG` environment variable. `RuniqueConfig::from_env()` reads and applies it automatically.
* **Prelude:** `dotenvy` re-exported in `runique::prelude` (CONFIGURATION section) and at crate root.
* **`runique/static/js/color_picker.js`:** New static JS file. Uses `data-color-picker` / `data-color-input` / `data-color-text` attributes for zero-inline-JS color picker sync. CSP-safe, idempotent on multiple color fields per page.

### Changed

* **`engine/core.rs`:** `SecurityPolicy::from_env()` → `SecurityPolicy::default()`.
* **`MiddlewareStaging::apply_to_router()`:** Branches on `enable_header_security` to choose between `csp_middleware` (CSP only) and `security_headers_middleware` (CSP + all security headers).
* **`base_color.html`:** Inline `<script>` (color picker sync) replaced with external `color_picker.js` loaded via `<script src defer>`. No nonce required — field templates are rendered without request context so `csp_nonce` was never available.
* **`demo-app/main.rs`:** `upgrade-insecure-requests` is now conditional: enabled only in release builds (`cfg!(not(debug_assertions))`). Prevents Chrome from upgrading HTTP→HTTPS in localhost dev environments.

### Templates

* **Admin — inline `style=` removed:** `create.html` (`max-width:60%` → `card card-form`), `dashboard.html` (`grid-column: 1/-1` → `card-full-width`, `text-decoration:none` removed), `delete.html` (`display:inline` → `form-inline`), `edit.html` (`max-width:60%` → `card card-form`), `login.html` (`margin-bottom:1rem` removed), `admin_base.html` mobile burger (`display:none` → `hidden`).
* **`admin/composant/edit.html`:** Inline `<script>` (image preview) now carries `nonce="{{ csp_nonce }}"`.

### Docs

* **`derive_form/README.md`:** Complete rewrite — field types table, PK types, all options, FK syntax, complete blog example (User/Category/Post/Comment), `impl_objects!` with all query methods, `#[form(...)]` parameters.
* **`doc-tests/macro_db/model_complete.md`:** Rewritten with `model!` macro and `impl_objects!`.
* **`docs/fr/middleware/csp/` + `docs/en/middleware/csp/`:** Full rewrite of `csp.md`, `directives.md`, `nonce.md`, `headers.md`, `profils.md` / `profiles.md` — env vars removed, builder examples added, complete directive/toggle/preset tables.
* **`docs/fr/env/securite/` + `docs/en/env/security/`:** CSP section removed, replaced by a note pointing to the builder docs.
* **`docs/fr/middleware/hosts-cache/` + `docs/en/`:** `RUNIQUE_ENABLE_CSP` row removed.

### Tests

* **`tests/middleware/test_csp.rs`:** All direct field accesses (`csp.policy.*`, `csp.enable_header_security`) replaced with accessor methods. `from_env()` tests removed and replaced with `CspConfig` builder tests.
* **`tests/formulaire/test_csrf_gate.rs`:** `test_csrf_gate_token_valide_retourne_none` updated to use a valid 64-char hex token + `mask_csrf_token()` — matches the new masked-token contract.
* **`tests/middleware/test_csrf_integration.rs`:** `test_csrf_post_sans_header_passe` → `test_csrf_post_sans_header_sans_content_type_retourne_403` (expects 403); same for DELETE variant.

---

## [1.1.45] - 2026-03-10

### Fixed

* **Docs:** `admin!{}` — removed `template_*` fields (template overrides are now handled exclusively via the builder).
* **Docs:** `.with_proto_state()` → `.with_state()` in `admin/setup.md` (method does not exist in the codebase).
* **Docs:** `mon_theme/` → `my_theme/` in `admin/template/surcharge/surcharge.md` (EN — untranslated FR names).
* **Docs:** navigation labels reversed in `admin/template/surcharge/` and `admin/template/clef/` (FR).
* **Docs:** corrected `urlpatterns!` syntax in `architecture/` (FR+EN):
  `get "/path" handler` → `"/path" => view!{ handler }, name = "name"`.
* **Docs:** `src/forms.rs` → `src/entities/` + `src/formulaire/` in `architecture/` (FR+EN).
* **Docs:** migration warning — `runique migration up/down/status` bypassed SeaORM tracking. Documentation reorganized into **“Recommended”** vs **“Advanced”** sections.
* **Docs:** `model!` syntax corrected: `model!(...)` → `model! { ... }` (braces, no semicolon).
* **Docs:** `impl_objects!` previously presented as a manual declaration → clarified as **automatically generated by the daemon**. Added note: *“pure syntactic sugar, SQL identical to native SeaORM.”*
* **Docs:** `use demo_app::models::users` → `use demo_app::entities::users` (6 occurrences in `orm/` and `routing/`).
* **Clippy:** removed unnecessary `&` borrows on `&'static str` returns in `admin_main.rs` and `admin_router.rs`.
* **Clippy:** `.to_string().into()` → `.to_string()` (unnecessary conversions in `demo-app/admins/admin_panel.rs`).

### Added

* **Docs:** **“Start a New Project”** section added to `architecture/` (FR+EN).
* **Docs:** sections **12–15** (Model, Auth, Sessions, Env) added to README hubs (FR+EN).
* **Docs:** English architecture documentation fully rewritten to match the French version.

---

## [1.1.44]

### Fixed

* CLI working correctly.

---

## [1.1.42]

### Security

* **CSRF:** CSRF token removal on `GET` requests.

---

## [1.1.38] - 2026-03-06

### Fixed

* **Memory leak:** `MemoryStore` (tower-sessions) never deleted expired sessions, causing unbounded memory growth under load
  (~1,369 MB after 5 minutes at 500 concurrent users).
  Replaced with `CleaningMemoryStore` with automatic periodic cleanup.

  Peak memory under identical load: **79 MB** (**-94%**).
  See `benchmark.md`.

### Added

* `CleaningMemoryStore`: in-process session store with periodic cleanup (60s timer, configurable via `RUNIQUE_SESSION_CLEANUP_SECS`).
* **Two-tier watermark system:**

  * **Low watermark (128 MB):** triggers asynchronous background purge of expired anonymous sessions.
  * **High watermark (256 MB):** triggers synchronous emergency purge + **503 refusal** if the store remains saturated.
    Configurable via `RUNIQUE_SESSION_LOW_WATERMARK` and `RUNIQUE_SESSION_HIGH_WATERMARK`.
* **Session protection:** sessions containing `user_id` (authenticated) or `session_active` (future timestamp set by `protect_session()`) are never sacrificed under memory pressure.
* `protect_session(&session, duration_secs)` / `unprotect_session(&session)` helpers for high-value anonymous sessions (shopping carts, multi-step forms).
* Builder methods:

  * `with_session_memory_limit(low, high)`
  * `with_session_cleanup_interval(secs)`
* Warning log when a session record exceeds **50 KB** (file or image accidentally stored in session).

### Changed

* Anonymous sessions now expire after **5 minutes of inactivity** (configurable).
* When a user authenticates, the session lifetime is automatically extended to **24 hours** (configurable).
* **Middleware slot 55:** dynamic session TTL upgrade after login without affecting CSRF logic or application handlers.

### Dev

* Added builder methods:

  * `with_session_duration`
  * `with_anonymous_session_duration`
    to customize session TTLs.

---

## [1.1.35] - 2026-03-04

### Changed

* Form system stabilized with several internal improvements.
* Builder updated with a new, more flexible middleware system.

### Security

* CSRF protection is now transparently enforced on all forms by default.

### Upcoming

* Initial design and planning phase for a basic admin view.

---