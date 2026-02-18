/// Relation type
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
    pub fn has_one(target: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::HasOne,
            target: target.into(),
        }
    }

    pub fn has_many(target: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::HasMany,
            target: target.into(),
        }
    }

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

    pub fn many_to_many(target: impl Into<String>, via: impl Into<String>) -> Self {
        Self {
            kind: RelationKind::ManyToMany { via: via.into() },
            target: target.into(),
        }
    }
}
