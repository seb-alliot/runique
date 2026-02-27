//! Tests — FlashMessage & MessageLevel
//! Couvre : constructeurs, level CSS classes, contenu

use runique::flash::{FlashMessage, MessageLevel};

// ── Constructeurs ─────────────────────────────────────────────────────────────

#[test]
fn test_flash_message_success() {
    let msg = FlashMessage::success("Opération réussie");
    assert_eq!(msg.content, "Opération réussie");
    assert!(matches!(msg.level, MessageLevel::Success));
}

#[test]
fn test_flash_message_error() {
    let msg = FlashMessage::error("Une erreur est survenue");
    assert_eq!(msg.content, "Une erreur est survenue");
    assert!(matches!(msg.level, MessageLevel::Error));
}

#[test]
fn test_flash_message_info() {
    let msg = FlashMessage::info("Vérifiez votre email");
    assert_eq!(msg.content, "Vérifiez votre email");
    assert!(matches!(msg.level, MessageLevel::Info));
}

#[test]
fn test_flash_message_warning() {
    let msg = FlashMessage::warning("Action irréversible");
    assert_eq!(msg.content, "Action irréversible");
    assert!(matches!(msg.level, MessageLevel::Warning));
}

#[test]
fn test_flash_message_new_generic() {
    let msg = FlashMessage::new("Message personnalisé", MessageLevel::Info);
    assert_eq!(msg.content, "Message personnalisé");
    assert!(matches!(msg.level, MessageLevel::Info));
}

#[test]
fn test_flash_message_accepts_string_owned() {
    let content = format!("Bienvenue, {}!", "Alice");
    let msg = FlashMessage::success(content);
    assert_eq!(msg.content, "Bienvenue, Alice!");
}

// ── CSS classes ───────────────────────────────────────────────────────────────

#[test]
fn test_message_level_success_css_class() {
    assert_eq!(MessageLevel::Success.as_css_class(), "success-message");
}

#[test]
fn test_message_level_error_css_class() {
    assert_eq!(MessageLevel::Error.as_css_class(), "error-message");
}

#[test]
fn test_message_level_info_css_class() {
    assert_eq!(MessageLevel::Info.as_css_class(), "info-message");
}

#[test]
fn test_message_level_warning_css_class() {
    assert_eq!(MessageLevel::Warning.as_css_class(), "warning-message");
}

// ── flash_now! macro ──────────────────────────────────────────────────────────

#[test]
fn test_flash_now_macro_single() {
    let msgs = runique::flash_now!(error => "Formulaire invalide");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "Formulaire invalide");
    assert!(matches!(msgs[0].level, MessageLevel::Error));
}

#[test]
fn test_flash_now_macro_multiple() {
    let msgs = runique::flash_now!(warning => "Champ A manquant", "Champ B manquant");
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "Champ A manquant");
    assert_eq!(msgs[1].content, "Champ B manquant");
    assert!(matches!(msgs[0].level, MessageLevel::Warning));
    assert!(matches!(msgs[1].level, MessageLevel::Warning));
}

#[test]
fn test_flash_now_macro_success() {
    let msgs = runique::flash_now!(success => "Tout va bien");
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0].level, MessageLevel::Success));
}

#[test]
fn test_flash_now_macro_info() {
    let msgs = runique::flash_now!(info => "Info A", "Info B", "Info C");
    assert_eq!(msgs.len(), 3);
}
