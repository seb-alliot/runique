//! Hot-reload watcher: monitors `src/admin.rs` and `src/main.rs`, regenerates or stops based on state.
use crate::admin::daemon::{generate, parse_admin_file};
use crate::utils::trad::{t, tf};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

/// Starts monitoring of `admin_path` and `main_path`.
///
/// - Modification of `admin.rs` → regenerates admin code.
/// - Modification of `main.rs` with `.with_admin(` commented out → stops the daemon.
///
/// Blocking — runs until Ctrl+C or `.with_admin` is disabled.
pub(crate) fn watch(admin_path: &Path, main_path: &Path) -> Result<(), String> {
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

    // Initial generation at startup
    run_generation(admin_path);

    // Debounce: avoids multiple regenerations for a single save
    let mut last_event = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let debounce = Duration::from_millis(300);

    for event in rx {
        match event {
            Ok(ev) => {
                if is_write_event(&ev) {
                    let now = Instant::now();
                    if now.duration_since(last_event) > debounce {
                        last_event = now;

                        // Check if the event concerns main.rs
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
                            // main.rs modified but with_admin still active → do nothing
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

/// Checks if `.with_admin(` is still active (not commented out) in main.rs
fn admin_still_active(main_path: &Path) -> bool {
    let Ok(source) = fs::read_to_string(main_path) else {
        return false;
    };
    source.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with("//") && trimmed.contains(".with_admin(")
    })
}

/// Checks if the event is a write/modification
fn is_write_event(event: &Event) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

/// Parse + generate — display the result
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
