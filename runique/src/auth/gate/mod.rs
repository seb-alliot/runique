//! The typed gate: every operation asks it for a proof, every guarded write
//! needs that proof — the admin's and `save_as`'s alike, with or without the
//! `admin` feature.
//!
//! Three levels, in this order; any "no" gives the same [`Refusal`]:
//! 1. Runique's floor (framework tables, secrets, superuser accounts): not configurable;
//! 2. the user's rights, by rank or groups ([`CrudAction`]);
//! 3. the developer's business rules, which can only restrict.
//!
//! Only this module builds an [`Authorized`]: its constructor is
//! `pub(in crate::auth::gate)`, so a shortcut elsewhere in Runique shows up
//! as a visibility change in review instead of a quiet call.
pub mod check;
pub mod decide;
pub mod operation;
pub mod proof;
pub mod refusal;
pub mod rights;
pub mod write;

#[cfg(test)]
pub(crate) mod test_support;

pub use check::{GateCtx, authorize_create, authorize_row};
pub use operation::{AdminOperation, Create, Delete, List, ResetPassword, Update, View};
pub use proof::{Authorized, Guarded, ReadableIds, WritesLinks};
pub use refusal::Refusal;
pub use rights::{Crud, CrudAction, Scope};

/// Request extension set once a request is inside the admin's constraint: its
/// panel and the routes declared `admin`. Only Runique can build one, so no
/// middleware of the app can put a form under or out of the gate.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AdminConstraint;

/// Value type of an entity's primary key.
pub type PkOf<E> = <<E as sea_orm::EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType;
