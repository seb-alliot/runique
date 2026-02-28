// Tests pour IndexDef

use runique::migration::index::IndexDef;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_index_new_une_colonne() {
    let idx = IndexDef::new(vec!["email"]);
    assert_eq!(idx.columns, vec!["email"]);
    assert!(!idx.unique, "unique doit être false par défaut");
    assert!(idx.name.is_none(), "name doit être None par défaut");
}

#[test]
fn test_index_new_multi_colonnes() {
    let idx = IndexDef::new(vec!["last_name", "first_name"]);
    assert_eq!(idx.columns.len(), 2);
    assert_eq!(idx.columns[0], "last_name");
    assert_eq!(idx.columns[1], "first_name");
}

// ═══════════════════════════════════════════════════════════════
// Builder — unique, name
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_index_unique() {
    let idx = IndexDef::new(vec!["email"]).unique();
    assert!(idx.unique);
}

#[test]
fn test_index_name_explicite() {
    let idx = IndexDef::new(vec!["email"]).name("idx_users_email");
    assert_eq!(idx.name.as_deref(), Some("idx_users_email"));
}

#[test]
fn test_index_unique_et_name() {
    let idx = IndexDef::new(vec!["slug"])
        .unique()
        .name("idx_articles_slug");
    assert!(idx.unique);
    assert_eq!(idx.name.as_deref(), Some("idx_articles_slug"));
}

// ═══════════════════════════════════════════════════════════════
// Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_index_clone() {
    let idx = IndexDef::new(vec!["email"]).unique().name("my_index");
    let cloned = idx.clone();
    assert_eq!(cloned.columns, vec!["email"]);
    assert!(cloned.unique);
    assert_eq!(cloned.name.as_deref(), Some("my_index"));
}

// ═══════════════════════════════════════════════════════════════
// to_sea_index — ne panique pas
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_index_to_sea_index_sans_nom() {
    // Nom auto-généré : idx_{table}_{colonnes}
    let idx = IndexDef::new(vec!["email"]);
    let _ = idx.to_sea_index("users");
}

#[test]
fn test_index_to_sea_index_avec_nom_explicite() {
    let idx = IndexDef::new(vec!["email"]).name("idx_custom");
    let _ = idx.to_sea_index("users");
}

#[test]
fn test_index_to_sea_index_multi_colonnes() {
    let idx = IndexDef::new(vec!["last_name", "first_name"]);
    let _ = idx.to_sea_index("contacts");
}

#[test]
fn test_index_to_sea_index_unique() {
    let idx = IndexDef::new(vec!["email"]).unique();
    let _ = idx.to_sea_index("users");
}
