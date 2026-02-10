// ═══════════════════════════════════════════════════════════════
// Watcher — Surveillance de src/admin.rs
// ═══════════════════════════════════════════════════════════════
//
// Surveille src/admin.rs avec notify et déclenche la génération
// à chaque modification.
//
// Flux :
//   Modification détectée
//     → parse src/admin.rs
//     → génère target/runique/admin/generated.rs
//     → affiche le résultat (✅ ou ❌)
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
pub fn watch(admin_path: &Path, output_dir: &Path) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Impossible de créer le watcher: {}", e))?;

    watcher
        .watch(admin_path, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Impossible de surveiller {}: {}", admin_path.display(), e))?;

    // Génération initiale au démarrage
    println!("⚡ Génération initiale...");
    run_generation(admin_path, output_dir);

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
                        println!("\n📝 Modification détectée → régénération...");
                        run_generation(admin_path, output_dir);
                    }
                }
            }
            Err(e) => eprintln!("⚠️  Erreur watcher: {}", e),
        }
    }

    Ok(())
}

/// Vérifie si l'événement est une écriture/modification
fn is_write_event(event: &Event) -> bool {
    matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
}

/// Parse + génère — affiche le résultat
fn run_generation(admin_path: &Path, output_dir: &Path) {
    let source = match fs::read_to_string(admin_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Lecture impossible: {}", e);
            return;
        }
    };

    match parse_admin_file(&source) {
        Err(e) => {
            eprintln!("❌ Erreur de parsing: {}", e);
        }
        Ok(parsed) => {
            if parsed.resources.is_empty() {
                println!("⚠️  Aucune ressource dans admin!{{}} — rien à générer");
                return;
            }

            match generate(&parsed.resources, output_dir) {
                Ok(()) => {
                    println!("✅ {} ressource(s) générée(s):", parsed.resources.len());
                    for r in &parsed.resources {
                        println!("   • {} ({} → {})", r.key, r.model_type, r.form_type);
                    }
                    println!("   → {}/generated.rs", output_dir.display());
                }
                Err(e) => {
                    eprintln!("❌ Erreur de génération: {}", e);
                }
            }
        }
    }
}
