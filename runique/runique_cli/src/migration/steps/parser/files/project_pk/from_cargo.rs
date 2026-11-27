use super::*;

/// Asks cargo for the project of the current folder.
pub(crate) fn from_cargo() -> Result<PkType> {
    pk_of(&cargo_metadata()?)
}
