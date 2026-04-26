// Tests pour RelationDef
// RelationKind (Debug) est déjà couvert par test_relation_kind.rs

use runique::migration::relation::{RelationDef, RelationKind};

// ═══════════════════════════════════════════════════════════════
// has_one
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_relation_has_one_target() {
    let rel = RelationDef::has_one("profile");
    assert_eq!(rel.target, "profile");
    assert!(matches!(rel.kind, RelationKind::HasOne));
}

// ═══════════════════════════════════════════════════════════════
// has_many
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_relation_has_many_target() {
    let rel = RelationDef::has_many("posts");
    assert_eq!(rel.target, "posts");
    assert!(matches!(rel.kind, RelationKind::HasMany));
}

// ═══════════════════════════════════════════════════════════════
// belongs_to
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_relation_belongs_to_target() {
    let rel = RelationDef::belongs_to("users", "user_id", "id");
    assert_eq!(rel.target, "users");
    assert!(matches!(rel.kind, RelationKind::BelongsTo { .. }));
}

#[test]
fn test_relation_belongs_to_from_to() {
    let rel = RelationDef::belongs_to("categories", "category_id", "uid");
    if let RelationKind::BelongsTo { from, to } = rel.kind {
        assert_eq!(from, "category_id");
        assert_eq!(to, "uid");
    } else {
        panic!("expected BelongsTo");
    }
}

// ═══════════════════════════════════════════════════════════════
// many_to_many
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_relation_many_to_many_target() {
    let rel = RelationDef::many_to_many("tags", "article_tags");
    assert_eq!(rel.target, "tags");
}

#[test]
fn test_relation_many_to_many_via() {
    let rel = RelationDef::many_to_many("tags", "article_tags");
    if let RelationKind::ManyToMany { via } = rel.kind {
        assert_eq!(via, "article_tags");
    } else {
        panic!("expected ManyToMany");
    }
}

// ═══════════════════════════════════════════════════════════════
// Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_relation_clone_has_one() {
    let rel = RelationDef::has_one("profile");
    let cloned = rel.clone();
    assert_eq!(cloned.target, "profile");
    assert!(matches!(cloned.kind, RelationKind::HasOne));
}

#[test]
fn test_relation_clone_belongs_to() {
    let rel = RelationDef::belongs_to("users", "user_id", "id");
    let cloned = rel.clone();
    assert_eq!(cloned.target, "users");
    if let RelationKind::BelongsTo { from, to } = cloned.kind {
        assert_eq!(from, "user_id");
        assert_eq!(to, "id");
    } else {
        panic!("expected BelongsTo");
    }
}

#[test]
fn test_relation_clone_many_to_many() {
    let rel = RelationDef::many_to_many("tags", "article_tags");
    let cloned = rel.clone();
    assert_eq!(cloned.target, "tags");
}
