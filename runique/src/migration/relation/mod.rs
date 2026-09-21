//! SeaORM relation definitions — HasOne, HasMany, BelongsTo, ManyToMany.
//!
//! These structs feed the `to_model()` code generation in [`crate::migration::ModelSchema`].

/// Types of relations between two entities.
#[derive(Debug, Clone)]
pub enum RelationKind {
    HasOne,
    HasMany,
    BelongsTo { from: String, to: String },
    ManyToMany { via: String },
}

/// Definition of a SeaORM relation
#[derive(Debug, Clone)]
pub struct RelationDef {
    pub kind: RelationKind,
    pub target: String,
}

impl RelationDef {
    /// Declares a one-to-one relation to `target`.
    pub fn has_one(target: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::HasOne,
            target: target.into(),
        }
    }

    /// Declares a one-to-many relation to `target`.
    pub fn has_many(target: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::HasMany,
            target: target.into(),
        }
    }

    /// Declares a many-to-one relation to `target`, where `from` is the local
    /// foreign key column and `to` is the column it references on `target`.
    pub fn belongs_to(
        target: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        Self {
            kind: RelationKind::BelongsTo {
                from: from.into(),
                to: to.into(),
            },
            target: target.into(),
        }
    }

    /// Declares a many-to-many relation to `target` through the join table/entity `via`.
    pub fn many_to_many(target: impl Into<String>, via: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::ManyToMany { via: via.into() },
            target: target.into(),
        }
    }

    /// Display name for admin UI — no effect on migration generation.
    pub fn as_name(self, _name: impl Into<String>) -> Self {
        self
    }
}
