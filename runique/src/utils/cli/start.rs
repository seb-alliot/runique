//! `start` command — generates the entrypoint code `main.rs` and `admin.rs` from CLI templates.
use crate::utils::trad::{t, tf};
use anyhow::Result;

use std::{fs, path::Path, process::Command};

pub fn runique_start(main_path: &str, admin_path: &str) -> Result<()> {
    let main_file = Path::new(main_path);

    if !main_file.exists() {
        anyhow::bail!("{}", tf("cli.file_not_found", &[&main_path]));
    }

    let main_source = fs::read_to_string(main_file)?;

    if !has_admin(&main_source) {
        println!("  {}", t("cli.add_admin_hint"));
        return Ok(());
    }

    println!("{}", t("cli.admin_detected"));

    // Generate BEFORE handing over to cargo, and on this very thread.
    //
    // The generation rewrites `src/admins/`, which is exactly what cargo reads to
    // compile. Running both at once (the former `thread::spawn`) was a race: the
    // build could pick up a half-written `admin.rs`, and it failed only sometimes
    // — the worst kind of bug, never reproducible, never blamed on its cause.
    // Sequencing removes it without a lock: nothing compiles while we write.
    generate_admin(admin_path)?;
    format_workspace();

    // Start the application server once the admin code is on disk
    let status = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .status()
        .unwrap_or_else(|_| panic!("{}", t("cli.cargo_run_expect")));
    if !status.success() {
        anyhow::bail!("{}", t("cli.cargo_run_failed"));
    }
    Ok(())
}

// Detection of .with_admin() in main.rs

/// Checks if `src/main.rs` contains an active call to `.with_admin(...)`
/// (ignores lines commented with `//`)
fn has_admin(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with("//") && trimmed.contains(".with_admin(")
    })
}

// Formatting

/// Runs `cargo fmt --all` between generation and build.
///
/// The generated `admin.rs` is not rustfmt-canonical, and the CI enforces
/// `cargo fmt --all -- --check` — without this step every regeneration turns the
/// pipeline red until someone formats by hand.
///
/// Never fatal: formatting is cosmetic, so a missing `rustfmt` must not stop the
/// server from starting. The failure is reported rather than swallowed — a step
/// that silently does nothing is worse than one that says why.
fn format_workspace() {
    match Command::new("cargo").arg("fmt").arg("--all").status() {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("  {}", tf("cli.fmt_failed", &[&status.to_string()])),
        Err(e) => eprintln!("  {}", tf("cli.fmt_failed", &[&e.to_string()])),
    }
}

// AdminPanel generation

/// Generates `src/admins/` from `src/admin.rs`, and propagates any failure so the
/// caller stops before invoking cargo.
fn generate_admin(admin_path: &str) -> Result<()> {
    use crate::admin::daemon::generate_once;

    let admin_file = Path::new(admin_path);

    if !admin_file.exists() {
        anyhow::bail!("{}", tf("cli.admin_not_found", &[&admin_path]));
    }

    generate_once(admin_file)
        .map_err(|e| anyhow::anyhow!("{}", tf("cli.daemon_error", std::slice::from_ref(&e))))?;

    Ok(())
}
