//! Watcher hot-reload : surveille `src/admin.rs` et `src/main.rs`, régénère ou s'arrête selon l'état.
use crate::admin::daemon::{generate, parse_admin_file};
use crate::utils::trad::{t, tf};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

/// Démarre la surveillance de `admin_path` et `main_path`.
///
/// - Modification de `admin.rs` → régénère le code admin.
/// - Modification de `main.rs` avec `.with_admin(` commenté → arrête le daemon.
///
/// Bloquant — tourne jusqu'à Ctrl+C ou désactivation de `.with_admin`.
pub fn watch(admin_path: &Path, main_path: &Path) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Unable to create watcher: {}", e))?;

    watcher
        .watch(admin_path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Unable to watch {}: {}", admin_path.display(), e))?;

    if main_path.exists() {
        watcher
            .watch(main_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Unable to watch {}: {}", main_path.display(), e))?;
    }

    // Génération initiale au démarrage
    run_generation(admin_path);

    // Debounce : évite plusieurs régénérations pour un seul save
    let mut last_event = Instant::now() - Duration::from_secs(10);
    let debounce = Duration::from_millis(300);

    for event in rx {
        match event {
            Ok(ev) => {
                if is_write_event(&ev) {
                    let now = Instant::now();
                    if now.duration_since(last_event) > debounce {
                        last_event = now;

                        // Vérifier si l'événement concerne main.rs
                        let is_main = ev.paths.iter().any(|p| {
                            p.file_name()
                                .and_then(|n| n.to_str())
                                .map(|n| n == "main.rs")
                                .unwrap_or(false)
                        });

                        if is_main {
                            if !admin_still_active(main_path) {
                                println!("\n {}", t("daemon.with_admin_disabled"));
                                return Ok(());
                            }
                            // main.rs modifié mais with_admin toujours actif → rien à faire
                            continue;
                        }

                        println!("\n {}", t("daemon.modification_detected"));
                        run_generation(admin_path);
                    }
                }
            }
            Err(e) => eprintln!("  {}", tf("daemon.watcher_error", &[&e.to_string()])),
        }
    }

    Ok(())
}

/// Vérifie si `.with_admin(` est encore actif (non commenté) dans main.rs
fn admin_still_active(main_path: &Path) -> bool {
    let Ok(source) = fs::read_to_string(main_path) else {
        return false;
    };
    source.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with("//") && trimmed.contains(".with_admin(")
    })
}

/// Vérifie si l'événement est une écriture/modification
fn is_write_event(event: &Event) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

/// Parse + génère — affiche le résultat
fn run_generation(admin_path: &Path) {
    let source = match fs::read_to_string(admin_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(" {}", tf("daemon.unable_read", &[&e.to_string()]));
            return;
        }
    };

    match parse_admin_file(&source) {
        Err(e) => {
            eprintln!(" {}", tf("daemon.parse_error", &[&e.to_string()]));
        }
        Ok(parsed) => {
            if parsed.resources.is_empty() {
                println!(" {}", t("daemon.no_resource"));
                return;
            }

            match generate(&parsed) {
                Ok(()) => {
                    println!(" {}", t("daemon.operational"));
                }
                Err(e) => {
                    eprintln!(" {}", tf("daemon.generation_error", &[&e.to_string()]));
                }
            }
        }
    }
}
