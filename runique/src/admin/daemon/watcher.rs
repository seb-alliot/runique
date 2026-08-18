//! One-shot generation: parses `src/admin.rs` and generates the admin code.
use crate::admin::daemon::{generate, parse_admin_file};
use crate::utils::trad::{t, tf};
use std::{fs, path::Path};

/// Parse + generate, once.
///
/// Returns the failure instead of only printing it: `runique start` must not
/// hand over to `cargo` when generation failed — it would compile the previous
/// `src/admins/` and the developer would chase a bug in stale code.
pub(crate) fn generate_once(admin_path: &Path) -> Result<(), String> {
    let source =
        fs::read_to_string(admin_path).map_err(|e| tf("daemon.unable_read", &[&e.to_string()]))?;

    let parsed =
        parse_admin_file(&source).map_err(|e| tf("daemon.parse_error", &[&e.to_string()]))?;

    // No resource is not an error: an `admin!{}` block may legitimately be empty
    // while the developer is still writing it.
    if parsed.resources.is_empty() {
        println!(" {}", t("daemon.no_resource"));
        return Ok(());
    }

    generate(&parsed).map_err(|e| tf("daemon.generation_error", &[&e.to_string()]))?;
    println!(" {}", t("daemon.operational"));
    Ok(())
}
