//! Commande `start` — génère le code entrypoint `main.rs` et `admin.rs` depuis les templates CLI.
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
    // Lancer le daemon en thread séparé
    let admin_path = admin_path.to_string();
    std::thread::spawn(move || {
        if let Err(e) = start_admin_daemon(&admin_path) {
            eprintln!("{}", tf("cli.daemon_error", &[&e.to_string()]));
        }
    });

    // Lancer le serveur applicatif après le daemon
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

// Détection de .with_admin() dans main.rs

/// Vérifie si `src/main.rs` contient un appel à `.with_admin(...)`
fn has_admin(source: &str) -> bool {
    source.contains(".with_admin(")
}

// Daemon AdminPanel

fn start_admin_daemon(admin_path: &str) -> Result<()> {
    use crate::admin::daemon::watch;

    let admin_file = Path::new(admin_path);

    if !admin_file.exists() {
        anyhow::bail!("{}", tf("cli.admin_not_found", &[&admin_path]));
    }

    watch(admin_file)
        .map_err(|e| anyhow::anyhow!("{}", tf("cli.daemon_error", std::slice::from_ref(&e))))?;

    Ok(())
}
