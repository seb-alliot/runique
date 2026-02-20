// ═══════════════════════════════════════════════════════════════
// Surveille src/admin.rs avec notify et déclenche la génération
// à chaque modification.
//
// Flux :
//   Modification détectée
//     → parse src/admin.rs
//     → génère src/admin/generated.rs
// ═══════════════════════════════════════════════════════════════

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::admin::daemon::{generate, parse_admin_file};

/// Démarre la surveillance de admin_path et régénère à chaque modification
///
/// Bloquant — tourne jusqu'à Ctrl+C.
pub fn watch(admin_path: &Path) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Unable to create watcher: {}", e))?;

    watcher
        .watch(admin_path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Unable to watch {}: {}", admin_path.display(), e))?;

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
                        println!("\n Modification detected → regeneration...");
                        run_generation(admin_path);
                    }
                }
            }
            Err(e) => eprintln!("  Watcher error: {}", e),
        }
    }

    Ok(())
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
            eprintln!(" Unable to read: {}", e);
            return;
        }
    };

    match parse_admin_file(&source) {
        Err(e) => {
            eprintln!(" Parsing error: {}", e);
        }
        Ok(parsed) => {
            if parsed.resources.is_empty() {
                println!(" No resource in admin!{{}} — nothing to generate");
                return;
            }

            match generate(&parsed.resources) {
                Ok(()) => {
                    println!(" Daemon operational → src/admin/generated.rs");
                }
                Err(e) => {
                    eprintln!(" Generation error: {}", e);
                }
            }
        }
    }
}
