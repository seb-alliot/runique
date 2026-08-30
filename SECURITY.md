# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.2.x   | :white_check_mark: |
| < 2.0   | :x:                |

## Known Security Advisories

### Active Advisories

None currently identified. The dependency tree was checked against the two advisories previously tracked here (below) — neither applies to the current lockfile.

### Previously Tracked (now resolved)

#### RUSTSEC-2023-0071: RSA Marvin Attack — no longer applicable
- **Was affected via**: `rsa` (transitive dependency of an older `sqlx-mysql`)
- **Current state**: `rsa` does not appear anywhere in `Cargo.lock` — `sqlx-mysql` (now `0.9.0`) no longer pulls it in.

#### RUSTSEC-2025-0052: async-std Unmaintained — no longer applicable
- **Was affected via**: `async-std` (transitive dependency of `sea-orm`/`sqlx`)
- **Current state**: `async-std` does not appear anywhere in `Cargo.lock` — the SeaORM/sqlx stack Runique depends on has moved to pure Tokio.

Re-checked against `Cargo.lock` on 2026-08-30. If you maintain a fork with different dependency versions, verify with `cargo audit` before relying on this section.

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

1. **Always use HTTPS** (`enforce_https = true` in settings)
2. **Set strong SECRET_KEY** (32+ random characters)
3. **Configure ALLOWED_HOSTS** properly
4. **Enable CSP** (`strict_csp = true`)
5. **Keep dependencies updated**: `cargo update`
6. **Run security audits**: `cargo audit`

HTML output sanitization (via `ammonia`) and Tera auto-escaping are on by default — there is no `sanitize_inputs` flag to set; this isn't an opt-in behavior.

## Vulnerability Disclosure Timeline

- **Day 0**: Vulnerability reported
- **Day 1-2**: Acknowledgment and initial assessment
- **Day 3-7**: Fix development and testing
- **Day 7-14**: Release preparation and security advisory
- **Day 14+**: Public disclosure after fix is available
