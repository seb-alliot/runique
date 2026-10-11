//! What the gate HAS authorized.
use super::PkOf;
use sea_orm::{Condition, EntityTrait};
use std::marker::PhantomData;

/// Permission to perform the operation `O` on one row of `E` (no row for a
/// create). Built only by the gate; neither `Clone` nor `Copy`, so a write
/// consumes it and it can't be reused.
///
/// A write asks for the proof of ITS operation, so a create path can't reach
/// an update (the 3.0.3 `bulk_create` flaw):
///
/// ```compile_fail
/// use runique::admin::gate::{Authorized, Create, Update};
/// fn update<E: sea_orm::EntityTrait>(_proof: Authorized<Update, E>) {}
/// fn create_handler<E: sea_orm::EntityTrait>(proof: Authorized<Create, E>) {
///     update(proof);
/// }
/// ```
pub struct Authorized<O, E: EntityTrait> {
    /// The write takes the id HERE, never from a separate argument: a proof
    /// for one row can't be used on another.
    id: Option<PkOf<E>>,
    /// What the gate checked (owner, parent, not a superuser, business
    /// filter), replayed by the write in its `WHERE`.
    conditions: Condition,
    _op: PhantomData<O>,
}

impl<O, E: EntityTrait> Authorized<O, E> {
    pub(in crate::admin::gate) fn new(id: Option<PkOf<E>>, conditions: Condition) -> Self {
        Self {
            id,
            conditions,
            _op: PhantomData,
        }
    }

    pub fn id(&self) -> Option<&PkOf<E>> {
        self.id.as_ref()
    }

    /// Consumed by the write: the row and the conditions to replay.
    pub(in crate::admin::gate) fn into_parts(self) -> (Option<PkOf<E>>, Condition) {
        (self.id, self.conditions)
    }
}

/// Conditions checked by the gate, replayed by the write: a guarded update
/// or delete that touches no row is refused (the row changed in between).
pub trait Guarded {
    fn conditions(&self) -> Condition;
}

impl<O, E: EntityTrait> Guarded for Authorized<O, E> {
    fn conditions(&self) -> Condition {
        self.conditions.clone()
    }
}

/// Ids sent by the client (foreign key, many-to-many targets) that the user
/// can read. One id refused refuses the whole field.
pub struct ReadableIds<E: EntityTrait>(Vec<PkOf<E>>);

impl<E: EntityTrait> ReadableIds<E> {
    pub(in crate::admin::gate) fn new(ids: Vec<PkOf<E>>) -> Self {
        Self(ids)
    }

    pub fn ids(&self) -> &[PkOf<E>] {
        &self.0
    }
}

/// Proofs allowed to write a parent's many-to-many links: create and update.
/// A deletion relies on the cascade declared in the database.
pub trait WritesLinks<E: EntityTrait> {}
impl<E: EntityTrait> WritesLinks<E> for Authorized<super::operation::Create, E> {}
impl<E: EntityTrait> WritesLinks<E> for Authorized<super::operation::Update, E> {}
