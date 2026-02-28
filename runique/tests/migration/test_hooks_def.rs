// Tests pour HooksDef, Hook et HookType

use runique::migration::hooks::{HookType, HooksDef};

// ═══════════════════════════════════════════════════════════════
// HookType — PartialEq / Eq
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hook_type_eq() {
    assert_eq!(HookType::BeforeSave, HookType::BeforeSave);
    assert_eq!(HookType::AfterSave, HookType::AfterSave);
    assert_eq!(HookType::BeforeDelete, HookType::BeforeDelete);
    assert_eq!(HookType::AfterDelete, HookType::AfterDelete);
}

#[test]
fn test_hook_type_ne() {
    assert_ne!(HookType::BeforeSave, HookType::AfterSave);
    assert_ne!(HookType::BeforeDelete, HookType::AfterDelete);
    assert_ne!(HookType::BeforeSave, HookType::BeforeDelete);
}

// ═══════════════════════════════════════════════════════════════
// HooksDef::new() — valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hooks_new_vide() {
    let h = HooksDef::new();
    assert!(h.hooks.is_empty());
    assert!(h.file_path.is_none());
}

#[test]
fn test_hooks_default_vide() {
    let h = HooksDef::default();
    assert!(h.hooks.is_empty());
    assert!(h.file_path.is_none());
}

// ═══════════════════════════════════════════════════════════════
// HooksDef::from_file()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hooks_from_file() {
    let h = HooksDef::from_file("src/hooks/article.rs");
    assert_eq!(h.file_path.as_deref(), Some("src/hooks/article.rs"));
    assert!(h.hooks.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// add() — ajout de hooks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hooks_add_simple() {
    let h = HooksDef::new().add(HookType::BeforeSave, 1, "handle_before_save");
    assert_eq!(h.hooks.len(), 1);
    assert_eq!(h.hooks[0].hook_type, HookType::BeforeSave);
    assert_eq!(h.hooks[0].slot, 1);
    assert_eq!(h.hooks[0].handler_path, "handle_before_save");
}

#[test]
fn test_hooks_add_trie_par_slot() {
    // Ajout dans le mauvais ordre → doit être trié par slot
    let h = HooksDef::new()
        .add(HookType::AfterSave, 3, "h3")
        .add(HookType::BeforeSave, 1, "h1")
        .add(HookType::AfterSave, 2, "h2");
    assert_eq!(h.hooks[0].slot, 1);
    assert_eq!(h.hooks[1].slot, 2);
    assert_eq!(h.hooks[2].slot, 3);
}

#[test]
fn test_hooks_add_slot_zero() {
    let h = HooksDef::new().add(HookType::BeforeDelete, 0, "first_handler");
    assert_eq!(h.hooks[0].slot, 0);
}

// ═══════════════════════════════════════════════════════════════
// Méthodes raccourcis
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hooks_before_save() {
    let h = HooksDef::new().before_save(0, "my_before_save");
    assert_eq!(h.hooks.len(), 1);
    assert_eq!(h.hooks[0].hook_type, HookType::BeforeSave);
    assert_eq!(h.hooks[0].handler_path, "my_before_save");
}

#[test]
fn test_hooks_after_save() {
    let h = HooksDef::new().after_save(1, "my_after_save");
    assert_eq!(h.hooks[0].hook_type, HookType::AfterSave);
    assert_eq!(h.hooks[0].slot, 1);
}

#[test]
fn test_hooks_before_delete() {
    let h = HooksDef::new().before_delete(2, "my_before_delete");
    assert_eq!(h.hooks[0].hook_type, HookType::BeforeDelete);
}

#[test]
fn test_hooks_after_delete() {
    let h = HooksDef::new().after_delete(3, "my_after_delete");
    assert_eq!(h.hooks[0].hook_type, HookType::AfterDelete);
}

// ═══════════════════════════════════════════════════════════════
// Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_hooks_clone() {
    let h = HooksDef::new().before_save(1, "handler");
    let cloned = h.clone();
    assert_eq!(cloned.hooks.len(), 1);
    assert_eq!(cloned.hooks[0].handler_path, "handler");
    assert_eq!(cloned.hooks[0].hook_type, HookType::BeforeSave);
}
