//! Built and ready-to-launch Runique application.
use axum::Router;
use std::net::SocketAddr;
use tokio::signal;

use crate::{
    config::RuniqueConfig,
    translation::{t, tf},
    utils::aliases::AEngine,
};

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
    /// Keeps the non-blocking log file writers alive for the app's lifetime.
    /// Dropping these flushes and stops the background writer threads.
    pub _log_guards: Vec<tracing_appender::non_blocking::WorkerGuard>,
}

/// The startup line naming the database: its engine and, when the environment
/// tells it, its name. A connection built another way shows the engine alone
/// rather than a name that may not be its own.
#[cfg(feature = "orm")]
pub(crate) fn connected_db_message(backend: sea_orm::DbBackend) -> String {
    let engine = format!("{backend:?}");
    match crate::db::DatabaseConfig::displayed_name() {
        Some(name) => tf(
            runique_i18n::keys::server::info::CONNECTED_DB,
            &[engine, name],
        ),
        None => tf(
            runique_i18n::keys::server::info::CONNECTED_ENGINE,
            &[engine],
        ),
    }
}

/// Resolves when the server is asked to stop: Ctrl+C, or SIGTERM on Unix —
/// what `systemctl stop` and `docker stop` send. Without the second, those
/// cut the requests in flight (and a container, deaf to SIGTERM as PID 1,
/// is killed after its grace period).
pub(crate) async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = signal::ctrl_c().await {
            tracing::warn!(%error, "{}", crate::translation::t(runique_i18n::keys::server::log::CTRL_C_NOT_INSTALLED));
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::warn!(%error, "{}", crate::translation::t(runique_i18n::keys::server::log::SIGTERM_NOT_INSTALLED));
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    println!("\n{}", t(runique_i18n::keys::server::info::SHUTTING_DOWN));
}

impl RuniqueApp {
    /// Creates a new builder to configure the application.
    ///
    /// Shortcut to [`RuniqueAppBuilder::new`].
    pub fn builder(config: RuniqueConfig) -> RuniqueAppBuilder {
        RuniqueAppBuilder::new(config)
    }

    /// Launches the server.
    /// - If `ACME_ENABLED=true`: provisions TLS via Let's Encrypt and serves HTTPS on port 443.
    /// - Otherwise: serves HTTP on the configured port.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(feature = "acme"))]
        if crate::config::env::env_flag("ACME_ENABLED", false) {
            eprintln!(
                "⚠  {}",
                t(runique_i18n::keys::server::error::ACME_NOT_COMPILED)
            );
            eprintln!(
                "   {}",
                t(runique_i18n::keys::server::info::ACME_ADD_FEATURE)
            );
        }

        #[cfg(feature = "acme")]
        if self.engine.config.security.acme_enabled {
            return self.run_with_acme().await;
        }

        self.run_http().await
    }

    /// Standard HTTP server (no TLS).
    async fn run_http(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!(
            "{}:{}",
            self.engine.config.server.ip_server, self.engine.config.server.port
        );

        println!("   {}", t(runique_i18n::keys::server::info::OPERATIONAL));
        println!(
            "      └──>  {}",
            tf(
                runique_i18n::keys::server::info::LAUNCHED,
                &[format!("http://{addr}")]
            )
        );

        #[cfg(feature = "orm")]
        {
            println!(
                "          └──>  {}",
                connected_db_message(self.engine.db.get_database_backend())
            );
            #[cfg(feature = "test-utils")]
            eprintln!(
                "          └──>  {}",
                t(runique_i18n::keys::server::info::TEST_UTILS_FEATURE_ENABLED)
            );
            println!(
                "              └──> {}",
                t(runique_i18n::keys::server::hint::STOP)
            );
        }

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        Ok(())
    }
}
