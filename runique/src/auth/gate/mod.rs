//! The typed gate: every operation asks it for a proof, every guarded write
//! needs that proof — the admin's and `save_as`'s alike, with or without the
//! `admin` feature.
//!
//! Three levels, in this order; any "no" gives the same [`Refusal`]:
//! 1. Runique's floor (framework tables, secrets, superuser accounts): not configurable;
//! 2. the user's rights, by rank or groups ([`CrudAction`]);
//! 3. the developer's business rules, which can only restrict.
//!
//! A delete the database blocks because other rows still point at it is the
//! one refusal whose sentence differs: it says so, and names neither those
//! rows nor how many. The caller's rights do not change that sentence.
//!
//! Only this module builds an [`Authorized`]: its constructor is
//! `pub(in crate::auth::gate)`, so a shortcut elsewhere in Runique shows up
//! as a visibility change in review instead of a quiet call.
pub mod change;
pub mod check;
pub mod decide;
#[cfg(feature = "admin")]
pub mod membership;
pub mod operation;
pub mod proof;
#[cfg(feature = "admin")]
pub mod rank;
pub mod refusal;
pub mod rights;
#[cfg(feature = "admin")]
pub mod rights_grid;
pub mod save;
#[cfg(feature = "admin")]
pub mod transfer;
pub mod write;

#[cfg(test)]
pub(crate) mod test_support;

pub use check::{GateCtx, authorize_create, authorize_row};
pub use operation::{AdminOperation, Create, Delete, List, ResetPassword, Update, View};
pub use proof::{Authorized, Guarded, ReadableIds, WritesLinks};
pub use refusal::Refusal;
pub use rights::{Crud, CrudAction, Scope};

/// Request extension set once a request is inside the admin's constraint: its
/// panel and the routes declared `admin`. Carries the account the middleware
/// just read back, so the gate decides on it and never on `req.user`, which
/// the handler can change. Only Runique can build one.
#[derive(Clone, Debug)]
pub(crate) struct AdminConstraint(pub(crate) crate::auth::current_user::CurrentUser);

/// Value type of an entity's primary key.
pub type PkOf<E> = <<E as sea_orm::EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType;
