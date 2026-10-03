//! Admin registry configuration, display builders and the builtin user-create
//! form — written from cargo-mutants survivors (2026-10-02).
use std::sync::Arc;

use crate::helpers::pk::pk;
use runique::admin::forms::UserAdminCreateForm;
use runique::admin::helper::resource_entry::{FormBuilder, GroupAction, ResourceEntry};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::{AdminResource, ColumnFilter, DisplayConfig};
use runique::auth::permissions::{Groupe, Permission};
use runique::auth::session::CurrentUser;
use runique::forms::field::RuniqueForm;
use runique::forms::form::Forms;

fn entry(key: &'static str) -> ResourceEntry {
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
    ResourceEntry::new(AdminResource::new(key, "M", "F", key), form_builder)
}

fn registry(keys: &[&'static str]) -> AdminRegistry {
    let mut reg = AdminRegistry::new();
    for k in keys {
        reg.register(entry(k));
    }
    reg
}

fn staff_reading(resource: &str) -> CurrentUser {
    let mut read = Permission::zeroed(resource.to_string());
    read.can_read = true;
    CurrentUser {
        id: pk(2),
        username: "staff".into(),
        is_staff: true,
        is_superuser: false,
        groupes: vec![Groupe {
            id: 1,
            nom: "g".into(),
            permissions: vec![read],
        }],
    }
}

#[test]
fn a_staff_member_sees_exactly_the_resources_they_can_read() {
    let reg = registry(&["posts", "users"]);
    let staff = staff_reading("posts");
    let keys: Vec<&str> = reg.visible_to(&staff).iter().map(|m| m.key).collect();
    assert_eq!(keys, ["posts"]);
    assert_eq!(reg.accessible_keys(&staff), ["posts"]);
}

#[test]
fn configure_and_group_actions_touch_only_the_named_resource() {
    let mut reg = registry(&["posts", "users"]);
    reg.configure("posts", DisplayConfig::new().pagination(7));
    reg.configure("ghost", DisplayConfig::new().pagination(3));
    assert_eq!(reg.get("posts").unwrap().meta.display.pagination, 7);
    assert_eq!(reg.get("users").unwrap().meta.display.pagination, 25);

    reg.configure_group_actions("users", vec![GroupAction::val("is_active", "Off", "false")]);
    assert_eq!(reg.get("users").unwrap().group_actions.len(), 1);
    assert!(reg.get("posts").unwrap().group_actions.is_empty());
}

#[test]
fn remove_and_reorder_keep_the_order_the_nav_shows() {
    let mut reg = registry(&["a", "b", "c", "d"]);
    reg.remove("b");
    assert_eq!(reg.keys(), ["a", "c", "d"]);
    reg.reorder(&["d".into(), "ghost".into(), "a".into()]);
    assert_eq!(
        reg.keys(),
        ["d", "a", "c"],
        "listed first, the rest after, unknown ignored"
    );
}

#[test]
fn display_config_builders_set_what_they_name() {
    let d = DisplayConfig::new()
        .pagination(10)
        .columns_include(vec![("title", "Titre")])
        .list_filter(vec![("status", "Statut", 5)]);
    assert_eq!(d.pagination, 10);
    assert!(
        matches!(&d.columns, ColumnFilter::Include(c) if c == &[("title".to_string(), "Titre".to_string())])
    );
    assert_eq!(
        d.list_filter,
        [("status".to_string(), "Statut".to_string(), 5)]
    );
    let ex = DisplayConfig::new().columns_exclude(vec!["password"]);
    assert!(matches!(&ex.columns, ColumnFilter::Exclude(c) if c == &["password".to_string()]));
}

#[test]
fn parent_scope_is_composite_only_with_a_local_key() {
    let composite = AdminResource::new("droits", "M", "F", "D").parent_scope(
        "groupes",
        "groupe_id",
        Some("resource_key"),
    );
    let own_pk =
        AdminResource::new("lignes", "M", "F", "L").parent_scope("commandes", "commande_id", None);
    assert!(composite.parent_scope.as_ref().unwrap().is_composite());
    assert!(!own_pk.parent_scope.as_ref().unwrap().is_composite());
}

async fn create_errors(username: &str, email: &str) -> Option<Vec<String>> {
    let mut form = Forms::new("csrf_token");
    UserAdminCreateForm::register_fields(&mut form);
    form.add_value("csrf_token", "csrf_token");
    form.add_value("username", username);
    form.add_value("email", email);
    let mut f = UserAdminCreateForm::from_form(form);
    if f.is_valid().await {
        return None;
    }
    let mut keys: Vec<String> = f.get_form().errors().into_keys().collect();
    keys.sort();
    Some(keys)
}

#[tokio::test]
async fn the_user_create_form_needs_a_three_character_username() {
    assert_eq!(
        create_errors("abc", "a@b.fr").await,
        None,
        "3 characters is enough"
    );
    assert_eq!(
        create_errors("ab", "a@b.fr").await,
        Some(vec!["username".into()])
    );
}
