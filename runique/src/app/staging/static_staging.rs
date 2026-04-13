//! Static files staging: enables or disables the asset service.
use crate::app::error_build::BuildError;
//
// Controls whether static files (CSS, JS, media, Runique
// internal assets) are served by the application.
//
// Enabled by default. Can be disabled for pure APIs
// or when a CDN/reverse-proxy manages static files.
// ═══════════════════════════════════════════════════════════════

pub struct StaticStaging {
    /// Indicates whether the static files service is enabled
    pub(crate) enabled: bool,
}

impl StaticStaging {
    /// Creates a StaticStaging (enabled by default)
    pub fn new() -> Self {
        Self { enabled: true }
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

    // ═══════════════════════════════════════════════════
    // Validation
    // ═══════════════════════════════════════════════════

    /// Validates static files configuration
    pub fn validate(&self) -> Result<(), BuildError> {
        // For now, nothing to validate:
        // - If disabled, we simply serve nothing
        // - If enabled, paths are checked by the config
        //
        // Future: check that static folders exist
        // when `enabled` is true
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
