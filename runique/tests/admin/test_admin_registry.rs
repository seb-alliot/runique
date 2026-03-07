//! Tests — AdminRegistry
//! Couvre : register, get, contains, len, is_empty, keys

use std::sync::Arc;

use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use runique::admin::resource_entry::{FormBuilder, ResourceEntry};

fn make_entry(key: &'static str, title: &'static str) -> ResourceEntry {
    let meta = AdminResource::new(key, "module::Model", "module::Form", title, vec![]);
    let form_builder: FormBuilder = Arc::new(|_, _, _, _| Box::pin(async { unreachable!() }));
    ResourceEntry::new(meta, form_builder)
}

// ── État initial ──────────────────────────────────────────────────────────────

#[test]
fn test_registry_new_is_empty() {
    let registry = AdminRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ── register + get ────────────────────────────────────────────────────────────

#[test]
fn test_registry_register_single() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "Utilisateurs"));
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}

#[test]
fn test_registry_get_existing() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "Utilisateurs"));
    let resource = registry.get("users");
    assert!(resource.is_some());
    assert_eq!(resource.unwrap().meta.title, "Utilisateurs");
}

#[test]
fn test_registry_get_unknown_returns_none() {
    let registry = AdminRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_registry_get_correct_resource_by_key() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "Utilisateurs"));
    registry.register(make_entry("posts", "Articles"));
    let r = registry.get("posts").unwrap();
    assert_eq!(r.meta.key, "posts");
    assert_eq!(r.meta.title, "Articles");
}

// ── contains ─────────────────────────────────────────────────────────────────

#[test]
fn test_registry_contains_registered() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "U"));
    assert!(registry.contains("users"));
}

#[test]
fn test_registry_does_not_contain_unregistered() {
    let registry = AdminRegistry::new();
    assert!(!registry.contains("users"));
}

// ── len ───────────────────────────────────────────────────────────────────────

#[test]
fn test_registry_len_after_multiple_registrations() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "U"));
    registry.register(make_entry("posts", "P"));
    registry.register(make_entry("comments", "C"));
    assert_eq!(registry.len(), 3);
}

// ── keys ─────────────────────────────────────────────────────────────────────

#[test]
fn test_registry_keys_empty() {
    let registry = AdminRegistry::new();
    assert!(registry.keys().is_empty());
}

#[test]
fn test_registry_keys_returns_all() {
    let mut registry = AdminRegistry::new();
    registry.register(make_entry("users", "U"));
    registry.register(make_entry("posts", "P"));
    let keys = registry.keys();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"users"));
    assert!(keys.contains(&"posts"));
}
