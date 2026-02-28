// Tests pour SessionConfig et SessionBackend

use runique::middleware::{SessionBackend, SessionConfig};
use tower_sessions::cookie::time::Duration;

// ═══════════════════════════════════════════════════════════════
// Tests — valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_session_config_default_duration_est_86400() {
    let cfg = SessionConfig::default();
    assert_eq!(cfg.duration, Duration::seconds(86400));
}

#[test]
fn test_session_config_default_backend_est_memory() {
    let cfg = SessionConfig::default();
    assert!(matches!(cfg.session, SessionBackend::Memory));
}

// ═══════════════════════════════════════════════════════════════
// Tests — with_duration (builder)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_session_config_with_duration_modifie_duree() {
    let cfg = SessionConfig::default().with_duration(Duration::seconds(3600));
    assert_eq!(cfg.duration, Duration::seconds(3600));
}

#[test]
fn test_session_config_with_duration_ne_change_pas_le_backend() {
    let cfg = SessionConfig::default().with_duration(Duration::seconds(7200));
    assert!(matches!(cfg.session, SessionBackend::Memory));
}

#[test]
fn test_session_config_with_duration_zero() {
    let cfg = SessionConfig::default().with_duration(Duration::seconds(0));
    assert_eq!(cfg.duration, Duration::seconds(0));
}

#[test]
fn test_session_config_with_duration_chaining() {
    // Le dernier appel gagne
    let cfg = SessionConfig::default()
        .with_duration(Duration::seconds(100))
        .with_duration(Duration::seconds(999));
    assert_eq!(cfg.duration, Duration::seconds(999));
}
