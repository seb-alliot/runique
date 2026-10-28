# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 3.0.x   | :white_check_mark: (use **3.0.3** or later) |
| < 3.0   | :x:                |

Runique 3.0.2 fixes security issues that are also present in 2.2.x (listed below): upgrade with the [3.0 migration guide](MIGRATION-3.0.md).

## Known Security Advisories

### Active Advisories

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

None currently identified. The dependency tree was checked against the two advisories previously tracked here (below) — neither applies to the current lockfile.

### Previously Tracked (now resolved)

#### RUSTSEC-2023-0071: RSA Marvin Attack — no longer applicable
- **Was affected via**: `rsa` (transitive dependency of an older `sqlx-mysql`)
- **Current state**: `rsa` does not appear anywhere in `Cargo.lock` — `sqlx-mysql` (now `0.9.0`) no longer pulls it in.

#### RUSTSEC-2025-0052: async-std Unmaintained — no longer applicable
- **Was affected via**: `async-std` (transitive dependency of `sea-orm`/`sqlx`)
- **Current state**: `async-std` does not appear anywhere in `Cargo.lock` — the SeaORM/sqlx stack Runique depends on has moved to pure Tokio.

Re-checked against `Cargo.lock` on 2026-10-10. The CI runs `cargo audit` on every push, with no ignored advisory, and a known vulnerability fails the build. If you maintain a fork with different dependency versions, verify with `cargo audit` before relying on this section.

## Fixed in Runique

| Version | Issue | Severity |
| ------- | ----- | -------- |
| 4.0 (upcoming) | `extend!{}` of the accounts table putting `password` and `is_active` in its admin form | High |
| 4.0 (upcoming) | CSRF token sent by `csrf.js` with `fetch` requests to other origins | Medium |
| 4.0 (upcoming) | Form global errors rendered without escaping (XSS) | Medium |
| 4.0 (upcoming) | Debug error page able to show a file outside `templates/` | Medium |
| 4.0 (upcoming) | Admin refusal message telling that an account is a superuser | Low |
| 4.0 (upcoming) | "Forgot password" response time telling whether an account exists | Low |
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

The 4.0 fixes are described under Active Advisories above. Details in the CHANGELOG: [3.0.3](CHANGELOG.md#303---2026-10-10), [3.0.2](CHANGELOG.md#302---2026-10-09). Earlier fixes (2.1.x) are listed in [PROJECT_STATUS](docs/en/PROJECT_STATUS.en.md).

## Reporting a Vulnerability

If you discover a security vulnerability in Runique itself (not dependencies), please:

1. **Do NOT** open a public issue
2. Email: [alliotsebastien04@gmail.com]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work on a fix as soon as possible.

## Security Best Practices

When using Runique in production:

1. **Always use HTTPS** (`ENFORCE_HTTPS=true` behind a TLS proxy, or ACME) — it also turns on HSTS
2. **Run with `DEBUG=false`**: error pages then show no internal detail
3. **Set strong SECRET_KEY** (32+ random characters)
4. **Enable host validation** in the builder: `.middleware(|m| m.with_allowed_hosts(|h| h.enabled(true).host("mysite.com")))`
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
