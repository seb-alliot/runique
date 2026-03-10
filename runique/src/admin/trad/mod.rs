use crate::utils::{trad::t, ADMIN_MESSAGE_KEYS};
use tera::Context;

pub fn insert_admin_messages(context: &mut Context, section: &str) {
    let prefix = format!("admin.{}.", section);
    for key in ADMIN_MESSAGE_KEYS.iter().filter(|k| k.starts_with(&prefix)) {
        let var_name = key.replace('.', "_");
        context.insert(var_name, &t(key));
    }
}
