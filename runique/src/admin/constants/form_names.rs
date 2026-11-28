//! Names of the forms the admin's pages build, sent back in their hidden
//! `runique_form` field: a form sent to another page than its own is never
//! validated there.
pub const CREATE: &str = "admin_create";
pub const UPDATE: &str = "admin_update";
pub const DELETE: &str = "admin_delete";
pub const BULK: &str = "admin_bulk";
pub const RESET_PASSWORD: &str = "admin_reset_password";
pub const RANK: &str = "admin_rank";
pub const RIGHTS: &str = "admin_rights";
pub const GROUPS: &str = "admin_groups";
pub const TRANSFER: &str = "admin_transfer";
