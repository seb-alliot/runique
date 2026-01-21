# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Known Security Advisories

### Active Advisories

#### RUSTSEC-2023-0071: RSA Marvin Attack
- **Severity**: Medium (5.9)
- **Affected**: `rsa 0.9.10` (transitive dependency via `sqlx-mysql`)
- **Impact**: Potential timing sidechannel attack on RSA decryption
- **Mitigation**: 
  - Use SQLite or PostgreSQL instead of MySQL in production
  - If MySQL is required, this only affects SSL/TLS connections
  - No fix available yet, waiting for upstream `sqlx` update
- **Status**: Tracked, accepted risk for MySQL users

#### RUSTSEC-2025-0052: async-std Unmaintained
- **Severity**: Warning
- **Affected**: `async-std 1.13.2` (transitive dependency via `sea-orm`/`sqlx`)
- **Impact**: None (indirect usage only, no security vulnerability)
- **Mitigation**: None required
- **Status**: Waiting for SeaORM 3.0 / sqlx migration to pure Tokio

## Reporting a Vulnerability

If you discover a security vulnerability in Runique itself (not dependencies), please:

1. **Do NOT** open a public issue
2. Email: [your-email@example.com]
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
5. **Use sanitize_inputs** (`sanitize_inputs = true`)
6. **Keep dependencies updated**: `cargo update`
7. **Run security audits**: `cargo audit`

## Vulnerability Disclosure Timeline

- **Day 0**: Vulnerability reported
- **Day 1-2**: Acknowledgment and initial assessment
- **Day 3-7**: Fix development and testing
- **Day 7-14**: Release preparation and security advisory
- **Day 14+**: Public disclosure after fix is available
