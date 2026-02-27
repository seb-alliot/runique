//! Tests — AdminRegistry
//! Couvre : register, get, contains, len, is_empty, keys

use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;

fn make_resource(key: &'static str, title: &'static str) -> AdminResource {
    AdminResource::new(key, "module::Model", "module::Form", title, vec![])
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
    registry.register(make_resource("users", "Utilisateurs"));
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
}

#[test]
fn test_registry_get_existing() {
    let mut registry = AdminRegistry::new();
    registry.register(make_resource("users", "Utilisateurs"));
    let resource = registry.get("users");
    assert!(resource.is_some());
    assert_eq!(resource.unwrap().title, "Utilisateurs");
}

#[test]
fn test_registry_get_unknown_returns_none() {
    let registry = AdminRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_registry_get_correct_resource_by_key() {
    let mut registry = AdminRegistry::new();
    registry.register(make_resource("users", "Utilisateurs"));
    registry.register(make_resource("posts", "Articles"));
    let r = registry.get("posts").unwrap();
    assert_eq!(r.key, "posts");
    assert_eq!(r.title, "Articles");
}

// ── contains ─────────────────────────────────────────────────────────────────

#[test]
fn test_registry_contains_registered() {
    let mut registry = AdminRegistry::new();
    registry.register(make_resource("users", "U"));
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
    registry.register(make_resource("users", "U"));
    registry.register(make_resource("posts", "P"));
    registry.register(make_resource("comments", "C"));
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
    registry.register(make_resource("users", "U"));
    registry.register(make_resource("posts", "P"));
    let keys = registry.keys();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"users"));
    assert!(keys.contains(&"posts"));
}

#[test]
fn test_registry_keys_order_matches_insertion() {
    let mut registry = AdminRegistry::new();
    registry.register(make_resource("aaa", "A"));
    registry.register(make_resource("bbb", "B"));
    let keys = registry.keys();
    assert_eq!(keys[0], "aaa");
    assert_eq!(keys[1], "bbb");
}
