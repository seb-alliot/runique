//! Tests — nested (scoped child) resources at the registry level.
//! Couvre : un enfant scopé est masqué du nav (`visible_to`) mais reste
//! auditable (`accessible_keys`).

use std::sync::Arc;

use runique::admin::helper::resource_entry::{FormBuilder, ResourceEntry};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use runique::auth::session::CurrentUser;

fn form_builder() -> FormBuilder {
    Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }))
}

fn flat_entry(key: &'static str, title: &'static str) -> ResourceEntry {
    ResourceEntry::new(
        AdminResource::new(key, "M", "F", title, vec![]),
        form_builder(),
    )
}

fn scoped_child_entry() -> ResourceEntry {
    let meta = AdminResource::new("droits", "M", "F", "Droits", vec![]).parent_scope(
        "groupes",
        "groupe_id",
        Some("resource_key"),
    );
    ResourceEntry::new(meta, form_builder())
}

fn registry_with_child() -> AdminRegistry {
    let mut reg = AdminRegistry::new();
    reg.register(flat_entry("groupes", "Groupes"));
    reg.register(scoped_child_entry());
    reg
}

fn superuser() -> CurrentUser {
    CurrentUser {
        id: 1,
        username: "root".into(),
        is_staff: true,
        is_superuser: true,
        groupes: vec![],
    }
}

#[test]
fn scoped_child_visible_in_nav() {
    // A scoped child stays in the nav: it is reachable both at the top level and
    // inline under its parent (the two paths coexist by design).
    let reg = registry_with_child();
    let keys: Vec<&str> = reg.visible_to(&superuser()).iter().map(|m| m.key).collect();
    assert!(keys.contains(&"groupes"));
    assert!(keys.contains(&"droits"));
}

#[test]
fn scoped_child_accessible() {
    let reg = registry_with_child();
    let keys = reg.accessible_keys(&superuser());
    assert!(keys.contains(&"droits".to_string()));
    assert!(keys.contains(&"groupes".to_string()));
}
