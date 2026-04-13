//! Built and ready-to-launch Runique application.
use axum::Router;
use tokio::signal;

use crate::config::RuniqueConfig;
use crate::utils::aliases::AEngine;

use super::builder::RuniqueAppBuilder;

// ═══════════════════════════════════════════════════════════════
// RuniqueApp — Built application, ready to be launched
// ═══════════════════════════════════════════════════════════════

/// Compiled application: engine + router ready to serve HTTP requests.
pub struct RuniqueApp {
    /// Shared engine containing config, Tera, DB, and security policies.
    pub engine: AEngine,
    /// Axum router with all attached middlewares.
    pub router: Router,
}

impl RuniqueApp {
    /// Creates a new builder to configure the application.
    ///
    /// Shortcut to [`RuniqueAppBuilder::new`].
    pub fn builder(config: RuniqueConfig) -> RuniqueAppBuilder {
        RuniqueAppBuilder::new(config)
    }

    /// Launches the HTTP server with graceful shutdown (Ctrl+C).
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
