//! `create-superuser`: the command's entry point. Its logic (the interactive
//! wizard, the hashing and the insert) lives in `auth::create_superuser`.
use anyhow::Result;

/// Asks for the superuser's details and creates the account.
pub async fn create_superuser() -> Result<()> {
    crate::auth::create_superuser::create_superuser().await
}
