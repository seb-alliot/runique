//! Content Security Policy configuration passed via closure to the builder.
use crate::middleware::SecurityPolicy;

// ═══════════════════════════════════════════════════════════════
// CspConfig — CSP configuration passed via closure to the builder
// ═══════════════════════════════════════════════════════════════
//
// Used exclusively in the `with_csp` closure:
//
//   .middleware(|m| {
//       m.with_csp(|c| {
//           c.with_header_security(true)
//            .with_nonce(true)
//            .scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
//            .images(vec!["'self'", "data:"])
//       })
//   })
//
// Everything is `false`/default — you explicitly enable what you need.
//
// TOGGLES:
//   .with_header_security(bool) → HSTS, X-Frame-Options, COEP, COOP, CORP...
//   .with_nonce(bool)           → CSP nonce per request
//   .with_upgrade_insecure(bool)→ upgrade-insecure-requests
//
// PRESET:
//   .policy(SecurityPolicy::strict())
//
// DIRECTIVES:
//   .scripts(vec!["'self'"])
//   .styles(vec!["'self'"])
//   .images(vec!["'self'", "data:"])
//   .fonts / .connect / .objects / .media / .frames / .frame_ancestors
//   .base_uri / .form_action / .default_src
//
// ═══════════════════════════════════════════════════════════════

/// Content Security Policy configuration, passed via closure to `.with_csp(|c| { ... })`.
///
/// Everything is disabled or at its default value — explicitly enable what you need.
///
/// # Full Example
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_csp(|c| {
///         c.with_header_security(true)
///          .with_nonce(true)
///          .with_upgrade_insecure(true)
///          .scripts(vec!["'self'", "https://cdn.jsdelivr.net"])
///          .styles(vec!["'self'", "https://cdn.jsdelivr.net"])
///          .images(vec!["'self'", "data:"])
///     })
/// })
/// ```
///
/// # Example — strict preset
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_csp(|c| {
///         c.policy(SecurityPolicy::strict())
///          .with_header_security(true)
///     })
/// })
/// ```
///
/// # Disable CSP — do not call `.with_csp` at all.
#[derive(Default)]
pub struct CspConfig {
    pub(crate) policy: SecurityPolicy,
    /// Enables additional security headers (HSTS, X-Frame-Options,
    /// X-Content-Type-Options, Referrer-Policy, Permissions-Policy, COEP, COOP, CORP).
    pub(crate) enable_header_security: bool,
}

impl CspConfig {
    // ═══════════════════════════════════════════════════
    // TOGGLES — true/false
    // ═══════════════════════════════════════════════════

    /// Enables additional security headers alongside CSP:
    /// HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy,
    /// Permissions-Policy, COEP, COOP, CORP.
    pub fn with_header_security(mut self, enable: bool) -> Self {
        self.enable_header_security = enable;
        self
    }

    /// Enables or disables the CSP nonce (injected per request into `script-src` and `style-src`).
    pub fn with_nonce(mut self, enable: bool) -> Self {
        self.policy.use_nonce = enable;
        self
    }

    /// Enables or disables `upgrade-insecure-requests`.
    pub fn with_upgrade_insecure(mut self, enable: bool) -> Self {
        self.policy.upgrade_insecure_requests = enable;
        self
    }

    // ═══════════════════════════════════════════════════
    // PRESET
    // ═══════════════════════════════════════════════════

    /// Replaces the entire policy with a preset or a custom policy.
    ///
    /// ```rust,ignore
    /// c.policy(SecurityPolicy::strict())
    /// c.policy(SecurityPolicy::permissive())
    /// ```
    pub fn policy(mut self, policy: SecurityPolicy) -> Self {
        self.policy = policy;
        self
    }

    // ═══════════════════════════════════════════════════
    // CSP DIRECTIVES
    // ═══════════════════════════════════════════════════

    /// Configures `default-src`.
    pub fn default_src(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.default_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `script-src`.
    pub fn scripts(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.script_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `style-src`.
    pub fn styles(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.style_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `img-src`.
    pub fn images(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.img_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `font-src`.
    pub fn fonts(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.font_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `connect-src`.
    pub fn connect(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.connect_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `object-src`.
    pub fn objects(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.object_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `media-src`.
    pub fn media(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.media_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `frame-src`.
    pub fn frames(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.frame_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `frame-ancestors`.
    pub fn frame_ancestors(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.frame_ancestors = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `base-uri`.
    pub fn base_uri(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.base_uri = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configures `form-action`.
    pub fn form_action(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.form_action = src.into_iter().map(Into::into).collect();
        self
    }

    // ═══════════════════════════════════════════════════
    // ACCESSORS (used in tests)
    // ═══════════════════════════════════════════════════

    /// Returns the current CSP policy.
    pub fn get_policy(&self) -> &SecurityPolicy {
        &self.policy
    }

    /// Indicates whether additional security headers are enabled.
    pub fn header_security_enabled(&self) -> bool {
        self.enable_header_security
    }
}
