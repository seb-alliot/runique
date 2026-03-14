use crate::middleware::SecurityPolicy;

// ═══════════════════════════════════════════════════════════════
// CspConfig — Configuration CSP passée via closure au builder
// ═══════════════════════════════════════════════════════════════
//
// Utilisé exclusivement dans la closure de `with_csp` :
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
// Tout est `false`/défaut — tu actives explicitement ce dont tu as besoin.
//
// TOGGLES :
//   .with_header_security(bool) → HSTS, X-Frame-Options, COEP, COOP, CORP...
//   .with_nonce(bool)           → nonce CSP par requête
//   .with_upgrade_insecure(bool)→ upgrade-insecure-requests
//
// PRESET :
//   .policy(SecurityPolicy::strict())
//
// DIRECTIVES :
//   .scripts(vec!["'self'"])
//   .styles(vec!["'self'"])
//   .images(vec!["'self'", "data:"])
//   .fonts / .connect / .objects / .media / .frames / .frame_ancestors
//   .base_uri / .form_action / .default_src
//
// ═══════════════════════════════════════════════════════════════

/// Configuration du Content Security Policy, passée via closure à `.with_csp(|c| { ... })`.
///
/// Tout est désactivé ou à sa valeur par défaut — active explicitement ce dont tu as besoin.
///
/// # Exemple complet
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
/// # Exemple — preset strict
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_csp(|c| {
///         c.policy(SecurityPolicy::strict())
///          .with_header_security(true)
///     })
/// })
/// ```
///
/// # Désactiver le CSP — ne pas appeler `.with_csp` du tout.
#[derive(Default)]
pub struct CspConfig {
    pub(crate) policy: SecurityPolicy,
    /// Active les headers de sécurité additionnels (HSTS, X-Frame-Options,
    /// X-Content-Type-Options, Referrer-Policy, Permissions-Policy, COEP, COOP, CORP).
    pub(crate) enable_header_security: bool,
}

impl CspConfig {
    // ═══════════════════════════════════════════════════
    // TOGGLES — true/false
    // ═══════════════════════════════════════════════════

    /// Active les headers de sécurité additionnels aux côtés du CSP :
    /// HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy,
    /// Permissions-Policy, COEP, COOP, CORP.
    pub fn with_header_security(mut self, enable: bool) -> Self {
        self.enable_header_security = enable;
        self
    }

    /// Active ou désactive le nonce CSP (injecté par requête dans `script-src` et `style-src`).
    pub fn with_nonce(mut self, enable: bool) -> Self {
        self.policy.use_nonce = enable;
        self
    }

    /// Active ou désactive `upgrade-insecure-requests`.
    pub fn with_upgrade_insecure(mut self, enable: bool) -> Self {
        self.policy.upgrade_insecure_requests = enable;
        self
    }

    // ═══════════════════════════════════════════════════
    // PRESET
    // ═══════════════════════════════════════════════════

    /// Remplace la politique entière par un preset ou une policy personnalisée.
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
    // DIRECTIVES CSP
    // ═══════════════════════════════════════════════════

    /// Configure `default-src`.
    pub fn default_src(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.default_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `script-src`.
    pub fn scripts(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.script_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `style-src`.
    pub fn styles(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.style_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `img-src`.
    pub fn images(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.img_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `font-src`.
    pub fn fonts(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.font_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `connect-src`.
    pub fn connect(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.connect_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `object-src`.
    pub fn objects(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.object_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `media-src`.
    pub fn media(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.media_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `frame-src`.
    pub fn frames(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.frame_src = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `frame-ancestors`.
    pub fn frame_ancestors(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.frame_ancestors = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `base-uri`.
    pub fn base_uri(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.base_uri = src.into_iter().map(Into::into).collect();
        self
    }

    /// Configure `form-action`.
    pub fn form_action(mut self, src: Vec<impl Into<String>>) -> Self {
        self.policy.form_action = src.into_iter().map(Into::into).collect();
        self
    }

    // ═══════════════════════════════════════════════════
    // ACCESSEURS (utilisés dans les tests)
    // ═══════════════════════════════════════════════════

    /// Retourne la politique CSP courante.
    pub fn get_policy(&self) -> &SecurityPolicy {
        &self.policy
    }

    /// Indique si les headers de sécurité additionnels sont activés.
    pub fn header_security_enabled(&self) -> bool {
        self.enable_header_security
    }
}
