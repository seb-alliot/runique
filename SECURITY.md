# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 3.0.x   | :white_check_mark: (use **3.0.3** or later) |
| < 3.0   | :x:                |

Runique 3.0.2 fixes security issues that are also present in 2.2.x (listed below): upgrade with the [3.0 migration guide](MIGRATION-3.0.md).

## Known Security Advisories

### Active Advisories

#### `csrf.js` sends the CSRF token to other origins — Medium, fixed in 4.0

- **What it does**: `csrf.js` wraps `fetch` to add the `X-CSRF-Token` header to every `POST`/`PUT`/`PATCH`/`DELETE` request, without checking where the request goes. A `fetch` to another domain made from a page that loads the script (a third-party script, or project code) carries the session's CSRF token to that domain, and that domain's response can replace the page's token. With the token, another site can forge requests on behalf of the signed-in user.
- **Who is affected**: Runique 3.0.x, only when the browser lets the request leave the page. Runique's CSP is active by default and every preset sets `connect-src 'self'`: the browser then blocks any `fetch` to another origin before it is sent, and the token never leaves. Exposed: a project that adds another origin to `connect-src` (the token reaches that origin), or that turns the CSP off.
- **How to protect yourself now**: keep the default `connect-src 'self'`. If the page must call another origin, do it from the server rather than from the browser, or don't load `csrf.js` on that page.
- **Fix**: Runique 4.0 adds the header, and accepts a rotated token, only for requests to the page's own origin.

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
| 4.0 (upcoming) | CSRF token sent by `csrf.js` with `fetch` requests to other origins (see Active Advisories) | Medium |
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

Details in the CHANGELOG: [3.0.3](CHANGELOG.md#303---2026-10-10), [3.0.2](CHANGELOG.md#302---2026-10-09). Earlier fixes (2.1.x) are listed in [PROJECT_STATUS](docs/en/PROJECT_STATUS.en.md).

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
