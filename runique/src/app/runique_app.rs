use axum::Router;
use tokio::signal;

use crate::utils::aliases::AEngine;

// ═══════════════════════════════════════════════════════════════
// RuniqueApp — Application construite, prête à être lancée
// ═══════════════════════════════════════════════════════════════

pub struct RuniqueApp {
    pub engine: AEngine,
    pub router: Router,
}

impl RuniqueApp {
    /// Lance le serveur HTTP avec graceful shutdown (Ctrl+C).
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!(
            "{}:{}",
            self.engine.config.server.ip_server, self.engine.config.server.port
        );

        println!("   Runique is operational");
        println!("      └──>  Server launched on http://{}", addr);

        #[cfg(feature = "orm")]
        {
            let moteur_db = self.engine.db.get_database_backend();
            let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
            println!(
                "          └──>  Connected to database {:?} -> {}",
                moteur_db, db_name
            );
            println!("              └──> ctrl + c to stop");
        }

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(listener, self.router)
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.expect("Error signal Ctrl+C");
                println!("\nShutting down Runique server...");
            })
            .await?;

        Ok(())
    }
}
