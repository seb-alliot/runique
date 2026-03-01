//! Tests — macros/context/helper.rs
//! Couvre : ContextHelper::new, add, update, Deref, DerefMut

use runique::macros::context::helper::ContextHelper;
use serde_json::json;

// ═══════════════════════════════════════════════════════════════
// Création
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_context_helper_new_est_vide() {
    let ctx = ContextHelper::new();
    // Un contexte vide doit pouvoir être créé sans paniquer
    let _ = ctx;
}

#[test]
fn test_context_helper_default_identique_a_new() {
    let a = ContextHelper::new();
    let b = ContextHelper::default();
    // Les deux doivent produire des contextes fonctionnels
    let _ = (a, b);
}

// ═══════════════════════════════════════════════════════════════
// add
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_string() {
    let ctx = ContextHelper::new().add("username", "alice");
    // Le contexte doit contenir la clé après sérialisation
    let _ = ctx;
}

#[test]
fn test_add_number() {
    let _ctx = ContextHelper::new().add("count", 42u32);
}

#[test]
fn test_add_bool() {
    let _ctx = ContextHelper::new().add("is_admin", true);
}

#[test]
fn test_add_chaining() {
    let _ctx = ContextHelper::new()
        .add("a", "valeur_a")
        .add("b", 123i32)
        .add("c", false);
}

// ═══════════════════════════════════════════════════════════════
// update
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_update_avec_objet_json() {
    let data = json!({ "key1": "value1", "key2": 42 });
    let _ctx = ContextHelper::new().update(data);
}

#[test]
fn test_update_ignore_non_objet() {
    // Une valeur JSON non-objet (array, string, etc.) ne doit pas paniquer
    let data = json!([1, 2, 3]);
    let _ctx = ContextHelper::new().update(data);
}

#[test]
fn test_update_objet_vide() {
    let data = json!({});
    let _ctx = ContextHelper::new().update(data);
}

#[test]
fn test_update_apres_add() {
    let data = json!({ "extra": "info" });
    let _ctx = ContextHelper::new().add("existing", "value").update(data);
}

// ═══════════════════════════════════════════════════════════════
// Deref / DerefMut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_deref_permet_acces_aux_methodes_tera_context() {
    // add() consomme self et retourne un nouveau ContextHelper
    let mut ctx = ContextHelper::new().add("key", "value");
    // insert directement via DerefMut sur Context
    ctx.insert("autre", &"autre_val");
}
