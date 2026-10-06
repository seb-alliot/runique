use runique::prelude::*;
use std::path::PathBuf;

fn find_seed_sql() -> Option<PathBuf> {
    // Priorité : fichier ciblé (sans tables gérées par d'autres seeds)
    let candidates = [
        "seed.sql",
        "demo-app/seed.sql",
        "../seed.sql",
        "seed.sql",
        "demo-app/seed.sql",
        "../seed.sql",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

async fn psql_file(db_url: &str, path: &PathBuf) {
    let output = tokio::process::Command::new("psql")
        .arg(db_url)
        .arg("-f")
        .arg(path)
        .output()
        .await;

    if let Ok(out) = output {
        let stderr = String::from_utf8_lossy(&out.stderr);
        for line in stderr.lines() {
            if line.contains("ERREUR") || line.contains("ERROR") {
                tracing::warn!("demo_seed: {}", line);
            }
        }
    }
}

pub async fn seed_demo() {
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::warn!("demo_seed: DATABASE_URL manquant");
            return;
        }
    };

    let sql_path = match find_seed_sql() {
        Some(p) => p,
        None => {
            tracing::warn!("demo_seed: seed.sql introuvable, seed ignoré");
            return;
        }
    };

    // Re-seed à chaque démarrage. seed.sql vide lui-même les tables dans la
    // même transaction que le rechargement : un seed en échec garde le contenu.
    tracing::info!("demo_seed: re-seed depuis {:?}", sql_path);
    psql_file(&db_url, &sql_path).await;

    tracing::info!("demo_seed: terminé");
}
