//! `DbWrite`: what `ValidationForm::save` and `delete` write through — the
//! connection (`ADb`), or a transaction the handler opened
//! (`request.db_atomic()`) and commits once every write of it succeeded.
use crate::db::ADb;
use sea_orm::{ConnectionTrait, DatabaseTransaction, TransactionTrait};

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::ADb {}
    impl Sealed for sea_orm::DatabaseTransaction {}
}

/// Sealed: a form on a route declared `admin` writes through the gate only
/// when `as_adb` hands it the connection — a type claiming to be the
/// connection while holding a transaction would slip a gated write into it.
pub trait DbWrite:
    ConnectionTrait + TransactionTrait<Transaction = DatabaseTransaction> + sealed::Sealed
{
    /// The connection; `None` inside a transaction.
    fn as_adb(&self) -> Option<&ADb>;
}

impl DbWrite for ADb {
    fn as_adb(&self) -> Option<&ADb> {
        Some(self)
    }
}

impl DbWrite for DatabaseTransaction {
    fn as_adb(&self) -> Option<&ADb> {
        None
    }
}
