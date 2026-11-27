use tower_http::CompressionLevel;

/// Response compression passed via closure to `.with_compression(|c| { ... })`.
///
/// On by default, with tower-http's own defaults (32 bytes, `CompressionLevel::Default`).
/// Images, Server-Sent Events and gRPC responses are never compressed.
///
/// # Example — bigger threshold, cheaper level
/// ```rust,ignore
/// use runique::tower_http::CompressionLevel;
///
/// .middleware(|m| {
///     m.with_compression(|c| c.min_size(1024).level(CompressionLevel::Fastest))
/// })
/// ```
///
/// # Example — a reverse proxy already compresses
/// ```rust,ignore
/// m.with_compression(|c| c.enabled(false))
/// ```
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub(crate) enabled: bool,
    pub(crate) min_size: u64,
    pub(crate) level: CompressionLevel,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_size: 32,
            level: CompressionLevel::Default,
        }
    }
}

impl CompressionConfig {
    /// Turns response compression on or off.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Smallest response, in bytes, that gets compressed. A response whose
    /// size is unknown (a stream) is compressed whatever this value.
    #[must_use]
    pub fn min_size(mut self, bytes: u64) -> Self {
        self.min_size = bytes;
        self
    }

    /// Compression level, the one of tower-http (`Fastest`, `Default`, `Best`, `Precise(n)`).
    #[must_use]
    pub fn level(mut self, level: CompressionLevel) -> Self {
        self.level = level;
        self
    }
}
