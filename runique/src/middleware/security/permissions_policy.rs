use serde::{Deserialize, Serialize};

/// Allowed origins for a Permissions-Policy directive.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PermissionValue {
    /// `feature=()` — deny all origins.
    Deny,
    /// `feature=(self)` — allow same origin only.
    AllowSelf,
    /// `feature=*` — allow any origin.
    AllowAny,
    /// `feature=("https://a.com" "https://b.com")` — explicit allowlist.
    AllowList(Vec<String>),
}

/// Permissions-Policy header configuration.
///
/// Built via [`PermissionsPolicyConfig`](crate::app::staging::PermissionsPolicyConfig)
/// and stored on the engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    pub(crate) directives: Vec<(String, PermissionValue)>,
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        Self {
            directives: vec![
                // ── Sensors & hardware ──────────────────────────────────────
                ("accelerometer".into(), PermissionValue::Deny),
                ("ambient-light-sensor".into(), PermissionValue::Deny),
                ("bluetooth".into(), PermissionValue::Deny),
                ("camera".into(), PermissionValue::Deny),
                ("gyroscope".into(), PermissionValue::Deny),
                ("hid".into(), PermissionValue::Deny),
                ("magnetometer".into(), PermissionValue::Deny),
                ("microphone".into(), PermissionValue::Deny),
                ("midi".into(), PermissionValue::Deny),
                ("serial".into(), PermissionValue::Deny),
                ("usb".into(), PermissionValue::Deny),
                // ── Location & identity ─────────────────────────────────────
                ("geolocation".into(), PermissionValue::Deny),
                ("idle-detection".into(), PermissionValue::Deny),
                // ── Screen & media capture ──────────────────────────────────
                ("display-capture".into(), PermissionValue::Deny),
                // ── Payments ────────────────────────────────────────────────
                ("payment".into(), PermissionValue::Deny),
                // ── Privacy / fingerprinting ────────────────────────────────
                ("interest-cohort".into(), PermissionValue::Deny),
                ("local-fonts".into(), PermissionValue::Deny),
                // ── Deprecated / legacy ─────────────────────────────────────
                ("sync-xhr".into(), PermissionValue::Deny),
                // ── XR ──────────────────────────────────────────────────────
                ("xr-spatial-tracking".into(), PermissionValue::Deny),
                // ── Window management ────────────────────────────────────────
                ("window-management".into(), PermissionValue::Deny),
                // ── Allowed for same origin ──────────────────────────────────
                // WebAuthn: required for passkeys / U2F on the same origin.
                (
                    "publickey-credentials-create".into(),
                    PermissionValue::AllowSelf,
                ),
                (
                    "publickey-credentials-get".into(),
                    PermissionValue::AllowSelf,
                ),
                ("fullscreen".into(), PermissionValue::AllowSelf),
                ("picture-in-picture".into(), PermissionValue::AllowSelf),
            ],
        }
    }
}

impl PermissionsPolicy {
    pub fn to_header_value(&self) -> String {
        self.directives
            .iter()
            .map(|(feature, value)| match value {
                PermissionValue::Deny => format!("{feature}=()"),
                PermissionValue::AllowSelf => format!("{feature}=(self)"),
                PermissionValue::AllowAny => format!("{feature}=*"),
                PermissionValue::AllowList(origins) => {
                    let list = origins
                        .iter()
                        .map(|o| format!("\"{o}\""))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{feature}=({list})")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}
