//! i18n injection for admin templates — `insert_admin_messages` by section.
use crate::utils::{ADMIN_MESSAGE_KEYS, trad::t};
use tera::Context;

/// Injects every translated message of the `admin.{section}.*` keys listed
/// in `ADMIN_MESSAGE_KEYS` into the Tera context, each under the template
/// variable the list gives it (`{{ admin_section_key }}`).
pub fn insert_admin_messages(context: &mut Context, section: &str) {
    let prefix = format!("admin.{section}.");
    for (variable, key) in ADMIN_MESSAGE_KEYS
        .iter()
        .filter(|(_, key)| key.starts_with(&prefix))
    {
        context.insert(*variable, &t(key));
    }
}

/// Injects the `admin_prefix` variable into the Tera context.
///
/// Must be called in every admin handler so that templates can build
/// URLs dynamically instead of relying on the hardcoded `/admin/` path.
pub fn inject_admin_prefix(context: &mut Context, prefix: &str) {
    context.insert("admin_prefix", prefix.trim_end_matches('/'));
}
