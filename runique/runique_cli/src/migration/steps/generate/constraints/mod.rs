//! What the cases on keys and indexes share: a key's name, its `ForeignKey::create()`,
//! the MySQL check of a key's own index, and the guard a key change starts with.
mod fk_constraint_name;
mod fk_index_column;
mod key_changes_guard;
mod render_fk_create;
mod unless_mysql_fk;

pub(crate) use fk_constraint_name::fk_constraint_name;
pub(crate) use fk_index_column::fk_index_column;
pub(crate) use key_changes_guard::key_changes_guard;
pub(crate) use render_fk_create::render_fk_create;
pub(crate) use unless_mysql_fk::unless_mysql_fk;
