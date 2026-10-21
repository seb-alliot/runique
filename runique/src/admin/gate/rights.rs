//! What a user CAN do on a resource, read from their groups on every request.
//! The rights themselves live in `auth::permissions::rights`, shared with the developer's views.
pub use crate::auth::permissions::rights::{Crud, CrudAction, ReadableColumns};
use crate::utils::pk::Pk;
use sea_orm::EntityTrait;

/// Which rows a query on several rows (list, group action) may reach.
pub enum Scope<E: EntityTrait> {
    /// Full right: every row (still narrowed by the resource's business filter).
    All,
    /// `_own` right only: the query is restricted to `column = user`.
    Own(E::Column, Pk),
}
