//! i18n injection for admin templates — `insert_admin_messages` by section.
use crate::utils::{ADMIN_MESSAGE_KEYS, trad::t};
use tera::Context;

pub fn insert_admin_messages(context: &mut Context, section: &str) {
    let prefix = format!("admin.{section}.");
    for key in ADMIN_MESSAGE_KEYS.iter().filter(|k| k.starts_with(&prefix)) {
        let var_name = key.replace('.', "_");
        context.insert(var_name, &t(key));
    }
}

/// Injects the `admin_prefix` variable into the Tera context.
///
/// Must be called in every admin handler so that templates can build
/// URLs dynamically instead of relying on the hardcoded `/admin/` path.
pub fn inject_admin_prefix(context: &mut Context, prefix: &str) {
    context.insert("admin_prefix", prefix.trim_end_matches('/'));
}
