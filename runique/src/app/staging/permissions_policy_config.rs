//! Permissions-Policy configuration passed via closure to the builder.
use crate::middleware::security::permissions_policy::{PermissionValue, PermissionsPolicy};

// ═══════════════════════════════════════════════════════════════
// PermissionsPolicyConfig
// ═══════════════════════════════════════════════════════════════
//
// Used exclusively in the `with_permissions_policy` closure:
//
//   .middleware(|m| {
//       m.with_permissions_policy(|p| {
//           p.deny("geolocation")
//            .allow_self("fullscreen")
//            .allow("payment", vec!["https://pay.example.com"])
//       })
//   })
//
// Default: secure preset — denies geolocation, camera, microphone, payment,
// usb, bluetooth, accelerometer, gyroscope, magnetometer, midi, interest-cohort.
// Allows fullscreen and picture-in-picture for same origin.
//
// METHODS:
//   .deny(feature)                — `feature=()`
//   .allow_self(feature)          — `feature=(self)`
//   .allow_any(feature)           — `feature=*`
//   .allow(feature, origins)      — `feature=("url1" "url2")`
//
// ═══════════════════════════════════════════════════════════════

/// Permissions-Policy configuration, passed via closure to `.with_permissions_policy(|p| { ... })`.
///
/// Starts from a secure default. Call methods to override individual directives.
///
/// # Example
/// ```rust,ignore
/// .middleware(|m| {
///     m.with_permissions_policy(|p| {
///         p.deny("geolocation")
///          .allow_self("fullscreen")
///          .allow("payment", vec!["https://pay.example.com"])
///     })
/// })
/// ```
///
/// # Example — open up everything (not recommended)
/// ```rust,ignore
/// m.with_permissions_policy(|p| p.allow_any("camera").allow_any("microphone"))
/// ```
#[derive(Default)]
pub struct PermissionsPolicyConfig {
    inner: PermissionsPolicy,
}

impl PermissionsPolicyConfig {
    // ═══════════════════════════════════════════════════
    // DIRECTIVES
    // ═══════════════════════════════════════════════════

    /// Denies a feature for all origins: `feature=()`.
    pub fn deny(mut self, feature: impl Into<String>) -> Self {
        self.set(feature.into(), PermissionValue::Deny);
        self
    }

    /// Allows a feature for the same origin only: `feature=(self)`.
    pub fn allow_self(mut self, feature: impl Into<String>) -> Self {
        self.set(feature.into(), PermissionValue::AllowSelf);
        self
    }

    /// Allows a feature for any origin: `feature=*`.
    pub fn allow_any(mut self, feature: impl Into<String>) -> Self {
        self.set(feature.into(), PermissionValue::AllowAny);
        self
    }

    /// Allows a feature for a list of specific origins:
    /// `feature=("https://origin1" "https://origin2")`.
    pub fn allow(mut self, feature: impl Into<String>, origins: Vec<impl Into<String>>) -> Self {
        let origins = origins.into_iter().map(Into::into).collect();
        self.set(feature.into(), PermissionValue::AllowList(origins));
        self
    }

    // ═══════════════════════════════════════════════════
    // INTERNAL
    // ═══════════════════════════════════════════════════

    fn set(&mut self, feature: String, value: PermissionValue) {
        if let Some(d) = self
            .inner
            .directives
            .iter_mut()
            .find(|(f, _)| f == &feature)
        {
            d.1 = value;
        } else {
            self.inner.directives.push((feature, value));
        }
    }

    pub(crate) fn build(self) -> PermissionsPolicy {
        self.inner
    }

    // ═══════════════════════════════════════════════════
    // ACCESSOR (used in tests)
    // ═══════════════════════════════════════════════════

    /// Returns the current policy for inspection.
    pub fn get_policy(&self) -> &PermissionsPolicy {
        &self.inner
    }
}
