🌍 **Languages**: [English](SECURITY.md) | [Français](SECURITY.fr.md)

# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 4.0.x   | Yes (upcoming, not released yet) |
| 3.0.x   | Yes (use **3.0.3** or later) |
| < 3.0   | No                 |

Runique 4.0 keeps the 3.0.x security fixes and adds more (listed below). Runique 3.0.2 fixed issues also present in 2.2.x.

## Known Security Advisories

### Active Advisories (fixed in 4.0)

#### `extend!{}` of the accounts table puts `password` and `is_active` in its admin form — High, fixed in 4.0

- **What it does**: the form generated for an `extend!` of `eihwaz_users` (a user profile, say) takes both columns from the account. An admin can then set a user's password at creation or edit, and so know it, and deactivate any account, their own included, past the account rules.
- **Who is affected**: Runique 3.0.x projects that register such an `extend!` in `admin!{}`.
- **How to protect yourself now**: keep the extended accounts out of `admin!{}`, and manage those columns through your own views.
- **Fix**: Runique 4.0 never puts either column in a form, like the rank flags: a new account gets a random password nobody knows (its owner sets theirs from the email link), and an edit leaves both untouched whatever the request carries.

#### `csrf.js` sends the CSRF token to other origins — Medium, fixed in 4.0

- **What it does**: `csrf.js` wraps `fetch` to add the `X-CSRF-Token` header to every `POST`/`PUT`/`PATCH`/`DELETE` request, without checking where the request goes. A `fetch` to another domain made from a page that loads the script (a third-party script, or project code) carries the session's CSRF token to that domain, and that domain's response can replace the page's token. With the token, another site can forge requests on behalf of the signed-in user.
- **Who is affected**: Runique 3.0.x, only when the browser lets the request leave the page. Runique's CSP is active by default and every preset sets `connect-src 'self'`: the browser then blocks any `fetch` to another origin before it is sent, and the token never leaves. Exposed: a project that adds another origin to `connect-src` (the token reaches that origin), or that turns the CSP off.
- **How to protect yourself now**: keep the default `connect-src 'self'`. If the page must call another origin, do it from the server rather than from the browser, or don't load `csrf.js` on that page.
- **Fix**: Runique 4.0 adds the header, and accepts a rotated token, only for requests to the page's own origin.

#### Form global errors rendered without escaping (XSS) — Medium, fixed in 4.0

- **What it does**: a form's global errors (an error raised by `clean()`, a database error quoting the submitted value) go into the page unescaped. A value the user typed and the error repeats can carry HTML or a script.
- **Who is affected**: Runique 3.0.x forms whose global errors quote what the user sent.
- **How to protect yourself now**: don't repeat submitted values in `clean()`'s global errors; keep the CSP active (it is by default), which blocks an injected inline script.
- **Fix**: Runique 4.0 renders global errors through a template, under Tera's autoescape.

#### Debug error page can show a file outside `templates/` — Medium, fixed in 4.0

- **What it does**: on a template rendering error, the debug page reads and shows `templates/{name}` without checking the name. A template chosen from request data (a slug) makes `../../.env` show the project's secrets.
- **Who is affected**: Runique 3.0.x deployments reachable with `DEBUG=true`. With `DEBUG=false` the page doesn't exist.
- **How to protect yourself now**: never expose a deployment with `DEBUG=true`, and don't build template names from request data.
- **Fix**: Runique 4.0 only reads a plain relative path under `templates/`; `..` and absolute paths are refused.

#### Session content readable in the database — Medium, fixed in 4.0

- **What it does**: `eihwaz_sessions.session_data` holds the content of signed-in sessions in clear, CSRF token and whatever the application stores there included. Reading the database (a backup, an SQL injection) gives all of it.
- **Who is affected**: Runique 3.0.x projects using the default session store.
- **How to protect yourself now**: restrict access to the database and its backups, and keep sensitive values out of the session.
- **Fix**: Runique 4.0 encrypts the content (XChaCha20-Poly1305, key derived from `SECRET_KEY`, tied to the row). A row in clear, altered or copied is refused. `user_id` and `expires_at` stay in clear, and someone who can also read `SECRET_KEY` can decrypt. Upgrading empties the table: users sign in again once.

#### Form sent from another site reaches handlers that don't check the token — Medium, fixed in 4.0

- **What it does**: a handler reading its body with an Axum extractor (`Form<T>`, `Multipart`) instead of `request.form("name")` doesn't check the CSRF token, and the middleware lets a form without an `X-CSRF-Token` header through. Another site can then submit that form on behalf of a signed-in user.
- **Who is affected**: Runique 3.0.x projects with such a handler on a route that writes. Forms going through `request.form` are not affected: their token is checked.
- **How to protect yourself now**: check the token in those handlers, or move them to `request.form`.
- **Fix**: Runique 4.0 refuses (403), before the handler, a form the browser announces as sent from another site or a subdomain (`Sec-Fetch-Site`). Browsers only send that header over HTTPS and on localhost, so a handler using an extractor must still check the token. A path in `csrf_exempt()` or a request with `X-CSRF-Token` isn't concerned.

#### Admin refusal message tells that an account is a superuser — Low, fixed in 4.0

- **What it does**: refused an action on a superuser's account, a staff member reads "Only a superuser can change a superuser's account", which tells them which accounts are superusers.
- **Who is affected**: Runique 3.0.x admins with staff members.
- **How to protect yourself now**: nothing to do; the refusal itself is right, only its wording tells too much.
- **Fix**: Runique 4.0 shows the generic "Action not allowed" (9 languages).

#### "Forgot password" response time tells whether an account exists — Low, fixed in 4.0

- **What it does**: the "forgot password" page answers every address with the same message, but it writes the reset token, and waits for that write, only when the address belongs to an account. The page is a few milliseconds slower for an existing account, which tells someone timing it whether an address has one.
- **Who is affected**: Runique 3.0.x projects that call `with_password_reset()`.
- **How to protect yourself now**: keep the page's rate limit (5 requests per 5 minutes and per IP by default, `max_requests` / `retry_after`): it slows how many addresses one client can try.
- **Fix**: Runique 4.0 runs the account's lookup, the token and the email in the background: the page answers in the same time for every address.

#### Dependencies

No known vulnerability. The dependency tree was checked against the two advisories previously tracked here (below): neither applies. `cargo audit` also reports two crates as unmaintained, which is not a vulnerability: `paste` (RUSTSEC-2024-0436) and `rustls-pemfile` (RUSTSEC-2025-0134).

### Previously Tracked (now resolved)

#### RUSTSEC-2023-0071: RSA Marvin Attack — no longer applicable
- **Was affected via**: `rsa` (transitive dependency of an older `sqlx-mysql`)
- **Current state**: `rsa` does not appear anywhere in `Cargo.lock` — `sqlx-mysql` (now `0.9.0`) no longer pulls it in.

#### RUSTSEC-2025-0052: async-std Unmaintained — no longer applicable
- **Was affected via**: `async-std` (transitive dependency of `sea-orm`/`sqlx`)
- **Current state**: `async-std` does not appear anywhere in `Cargo.lock` — the SeaORM/sqlx stack Runique depends on has moved to pure Tokio.

Re-checked against a local `Cargo.lock` on 2026-11-07; that file isn't committed, so your resolved versions may differ. The CI runs `cargo audit` on every push, with no ignored advisory, and a known vulnerability fails the build. If you maintain a fork with different dependency versions, verify with `cargo audit` before relying on this section.

## Fixed in Runique

| Version | Issue | Severity |
| ------- | ----- | -------- |
| 4.0 | `extend!{}` of the accounts table putting `password` and `is_active` in its admin form | High |
| 4.0 | CSRF token sent by `csrf.js` with `fetch` requests to other origins | Medium |
| 4.0 | Form global errors rendered without escaping (XSS) | Medium |
| 4.0 | Debug error page able to show a file outside `templates/` | Medium |
| 4.0 | Session content stored in clear in the database | Medium |
| 4.0 | Password reset leaving the other sessions of a custom store valid | Medium |
| 4.0 | Form sent from another site reaching a handler that doesn't check the token | Medium |
| 4.0 | Uploads validated before being fully written, so a file over a field's limit could pass | Medium |
| 4.0 | A database delete error could leave a reusable session; session revocation outside the reset's transaction | Medium |
| 4.0 | Non-atomic reset token replacement: two valid tokens under concurrency | Medium |
| 4.0 | Admin refusal message telling that an account is a superuser | Low |
| 4.0 | "Forgot password" response time telling whether an account exists | Low |
| 4.0 | Account created without its activation in a project generated by `runique new` | Low |
| 4.0 | Hash with an unrecognized prefix: faster, hence measurable, check | Low |
| 4.0 | Duplicate CSRF token in a form body (fixed part) | Low |
| 4.0 | Ambiguous string enum aliases read as another variant | Low |
| 4.0 | `add_js` scripts without an SRI hash or a CSP nonce | Hardening |
| 3.0.3 | `bulk_create`: a create-only right could update existing rows (regenerate the admin) | High |
| 3.0.3 | Staff member able to take over a superuser's account (email change + password reset) | High |
| 3.0.3 | Open redirect through a tab (`/\t/evil.com`) or a non-ASCII `Location` left unchecked | Medium |
| 3.0.3 | Open redirect through the `Refresh` header, never checked | Medium |
| 3.0.2 | Stored XSS through an upload waiting for its form (path shown back, staging folder served without CSP) | High |
| 3.0.2 | Uploaded files written to disk before the CSRF check (multipart) | Medium |
| 3.0.2 | Open redirect through credentials in a URL (`allowed.com:x@evil.com`) | Medium |
| 3.0.2 | urlencoded / JSON request bodies with no size limit | Medium |
| 3.0.2 | Admin password reset reachable from a resource that isn't an account | Low |
| 3.0.2 | Admin accepted the raw CSRF token (per-response masking bypassed) | Hardening |

The main 4.0 fixes are described under "Active Advisories (fixed in 4.0)" above. Details in the CHANGELOG: [3.0.3](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md#303---2026-10-10), [3.0.2](https://github.com/seb-alliot/runique/blob/main/CHANGELOG.md#302---2026-10-09). Earlier fixes (2.1.x) are listed in [PROJECT_STATUS](https://github.com/seb-alliot/runique/blob/main/docs/en/PROJECT_STATUS.en.md).

## Reporting a Vulnerability

If you discover a security vulnerability in Runique itself (not dependencies), please:

1. **Do NOT** open a public issue
2. Email: alliotsebastien04@gmail.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work on a fix as soon as possible.

## Security Best Practices

When using Runique in production:

1. **Always use HTTPS** (`ENFORCE_HTTPS=true` behind a TLS proxy, or ACME) — it also turns on HSTS
2. **Run with `RUNIQUE_ENV=prod` and `DEBUG=false`**: the startup checks apply and error pages show no internal detail
3. **Set strong SECRET_KEY** (32+ random characters): it also encrypts the sessions saved in the database
4. **Enable host validation** in the builder: `.middleware(|m| m.with_allowed_hosts(|h| h.host("mysite.com")))`
5. **Use the strict CSP preset** (`.with_csp(|c| c.policy(SecurityPolicy::strict()))`) — CSP itself is always active by default
6. **Protect the admin sign-in against brute force**: `.with_admin(|a| a.with_login_guard(LoginGuard::new()))` — it's opt-in
7. **Size uploads explicitly**: `RUNIQUE_MAX_UPLOAD_MB` (unset, a request is capped at 2 MB)
8. **In a hand-written upload form, put `{% csrf %}` before the file inputs**: no file is written to disk before a valid token
9. **Keep dependencies updated**: `cargo update`
10. **Run security audits**: `cargo audit`

HTML output sanitization (via `ammonia`) and Tera auto-escaping are on by default — there is no `sanitize_inputs` flag to set; this isn't an opt-in behavior.

## Vulnerability Disclosure Timeline

- **Day 0**: Vulnerability reported
- **Day 1-2**: Acknowledgment and initial assessment
- **Day 3-7**: Fix development and testing
- **Day 7-14**: Release preparation and security advisory
- **Day 14+**: Public disclosure after fix is available
