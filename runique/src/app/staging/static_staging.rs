//! Static files staging: enables or disables the asset service.
use crate::app::error_build::BuildError;
//
// Controls whether static files (CSS, JS, media, Runique
// internal assets) are served by the application.
//
// Enabled by default. Can be disabled for pure APIs
// or when a CDN/reverse-proxy manages static files.
// ═══════════════════════════════════════════════════════════════

const DEFAULT_STATIC_CACHE: &str = "public, max-age=31536000, immutable";
const DEFAULT_MEDIA_CACHE: &str = "public, max-age=31536000, immutable";

pub struct StaticStaging {
    /// Indicates whether the static files service is enabled
    pub(crate) enabled: bool,
    /// Cache-Control header for static assets (/static/, /runique/static/)
    pub(crate) static_cache: &'static str,
    /// Cache-Control header for user-uploaded media (/media/)
    pub(crate) media_cache: &'static str,
}

impl StaticStaging {
    /// Creates a StaticStaging (enabled by default)
    pub fn new() -> Self {
        Self {
            enabled: true,
            static_cache: DEFAULT_STATIC_CACHE,
            media_cache: DEFAULT_MEDIA_CACHE,
        }
    }

    // ═══════════════════════════════════════════════════
    // Static files configuration
    // ═══════════════════════════════════════════════════

    /// Enables the static files service
    ///
    /// ```rust,ignore
    /// .static_files(|s| s.enable())
    /// ```
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disables the static files service
    ///
    /// Useful for pure APIs or when a CDN/reverse-proxy
    /// manages static files.
    ///
    /// ```rust,ignore
    /// .static_files(|s| s.disable())
    /// ```
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Enables or disables the static files service
    ///
    /// ```rust,ignore
    /// .static_files(|s| s.enabled(false))
    /// ```
    pub fn enabled(mut self, enable: bool) -> Self {
        self.enabled = enable;
        self
    }

    /// Overrides the Cache-Control header for versioned static assets.
    ///
    /// Default: `"public, max-age=31536000, immutable"`
    ///
    /// ```rust,ignore
    /// .static_files(|s| s.static_cache("public, max-age=86400"))
    /// ```
    pub fn static_cache(mut self, value: &'static str) -> Self {
        self.static_cache = value;
        self
    }

    /// Overrides the Cache-Control header for user-uploaded media.
    ///
    /// Default: `"public, max-age=3600"`
    ///
    /// ```rust,ignore
    /// .static_files(|s| s.media_cache("no-cache"))
    /// ```
    pub fn media_cache(mut self, value: &'static str) -> Self {
        self.media_cache = value;
        self
    }

    // ═══════════════════════════════════════════════════
    // Validation
    // ═══════════════════════════════════════════════════

    /// Validates static files configuration
    pub fn validate(&self) -> Result<(), BuildError> {
        Ok(())
    }

    /// Returns `true` if the static files service is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Static files are always ready
    pub fn is_ready(&self) -> bool {
        true
    }
}

impl Default for StaticStaging {
    fn default() -> Self {
        Self::new()
    }
}
