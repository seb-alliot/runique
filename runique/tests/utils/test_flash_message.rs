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

// `message.html` builds the class from the serialized level
// (`message-{{ message.level }}`): both must give the same lowercase class.
#[test]
fn test_css_class_matches_what_the_template_renders() {
    for (level, class) in [
        (MessageLevel::Success, "message-success"),
        (MessageLevel::Error, "message-error"),
        (MessageLevel::Info, "message-info"),
        (MessageLevel::Warning, "message-warning"),
    ] {
        let serialized = serde_json::to_value(&level).unwrap();
        assert_eq!(format!("message-{}", serialized.as_str().unwrap()), class);
        assert_eq!(level.as_css_class(), class);
    }
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
