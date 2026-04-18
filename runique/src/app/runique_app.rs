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

    /// Launches the server.
    /// - If `ACME_ENABLED=true`: provisions TLS via Let's Encrypt and serves HTTPS on port 443.
    /// - Otherwise: serves HTTP on the configured port.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(feature = "acme"))]
        if std::env::var("ACME_ENABLED").as_deref() == Ok("true") {
            eprintln!("⚠  ACME_ENABLED=true but the `acme` feature is not compiled in.");
            eprintln!("   Add features = [\"acme\"] to your Cargo.toml.");
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

    /// Returns true if the certificate should be renewed (missing or past renewal date).
    #[cfg(feature = "acme")]
    async fn _cert_needs_renewal_placeholder() {}
}

#[cfg(feature = "acme")]
async fn cert_needs_renewal(expires_path: &std::path::Path) -> bool {
    let Ok(content) = tokio::fs::read_to_string(expires_path).await else {
        return true;
    };
    let Ok(renew_after) = content.trim().parse::<chrono::DateTime<chrono::Utc>>() else {
        return true;
    };
    chrono::Utc::now() >= renew_after
}

impl RuniqueApp {
    /// HTTPS server with automatic Let's Encrypt certificate (ACME HTTP-01).
    ///
    /// Flow:
    /// 1. Start HTTP server on port 80 (ACME challenge + redirect to HTTPS)
    /// 2. Obtain certificate from Let's Encrypt (or load existing from ./certs/)
    /// 3. Start HTTPS server on port 443
    #[cfg(feature = "acme")]
    async fn run_with_acme(self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        use crate::utils::acme::{ChallengeStore, obtain_certificate};
        use axum::{
            extract::{Path, State},
            response::Redirect,
            routing::get,
        };
        use std::{collections::HashMap, path::Path as FsPath, sync::Arc};
        use tokio::sync::RwLock;

        let domain = self
            .engine
            .config
            .security
            .acme_domain
            .as_deref()
            .ok_or("ACME_ENABLED=true but ACME_DOMAIN is not set")?
            .to_string();
        let email = self
            .engine
            .config
            .security
            .acme_email
            .as_deref()
            .ok_or("ACME_ENABLED=true but ACME_EMAIL is not set")?
            .to_string();
        let ip = self.engine.config.server.ip_server.clone();

        // Shared store for ACME HTTP-01 tokens
        let challenge_store: ChallengeStore = Arc::new(RwLock::new(HashMap::new()));

        // HTTP server on port 80: serves ACME challenges + redirects to HTTPS
        let store_clone = challenge_store.clone();
        let domain_clone = domain.clone();
        let http_app = Router::new()
            .route(
                "/.well-known/acme-challenge/{token}",
                get(
                    |Path(token): Path<String>, State(store): State<ChallengeStore>| async move {
                        let map = store.read().await;
                        match map.get(&token) {
                            Some(key_auth) => key_auth.clone(),
                            None => String::from("not found"),
                        }
                    },
                ),
            )
            .fallback(move || {
                let domain = domain_clone.clone();
                async move { Redirect::permanent(&format!("https://{domain}")) }
            })
            .with_state(store_clone);

        let http_addr = format!("{ip}:80");
        let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
        tokio::spawn(async move {
            axum::serve(http_listener, http_app).await.ok();
        });

        // Load existing cert or run ACME flow
        let cert_path = FsPath::new("./certs/cert.pem");
        let key_path = FsPath::new("./certs/key.pem");
        let expires_path = FsPath::new("./certs/expires.txt");

        let needs_renewal =
            !cert_path.exists() || !key_path.exists() || cert_needs_renewal(expires_path).await;

        let (cert_pem, key_pem) = if !needs_renewal {
            tracing::info!("Loading existing TLS certificate from ./certs/");
            (
                tokio::fs::read(cert_path).await?,
                tokio::fs::read(key_path).await?,
            )
        } else {
            let (cert, key) = obtain_certificate(
                &domain,
                &email,
                challenge_store,
                false, // production — use true for testing
            )
            .await?;

            // Persist to disk for future restarts
            tokio::fs::create_dir_all("./certs").await?;
            tokio::fs::write(cert_path, &cert).await?;
            tokio::fs::write(key_path, &key).await?;

            // Store renewal date: Let's Encrypt certs last 90 days, renew after 60
            let renew_after = chrono::Utc::now() + chrono::Duration::days(60);
            tokio::fs::write(expires_path, renew_after.to_rfc3339()).await?;
            tracing::info!(
                "Certificate will be renewed after {}",
                renew_after.format("%Y-%m-%d")
            );

            (cert, key)
        };

        // Start HTTPS server on port 443
        let https_addr = format!("{ip}:443");
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem(cert_pem, key_pem).await?;

        println!("   Runique is operational");
        println!("      └──>  Server launched on https://{}", domain);
        #[cfg(feature = "orm")]
        {
            let moteur_db = self.engine.db.get_database_backend();
            let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
            println!(
                "          └──>  Connected to database {:?} -> {}",
                moteur_db, db_name
            );
        }
        println!("              └──> ctrl + c to stop");

        axum_server::bind_rustls(https_addr.parse()?, tls_config)
            .serve(self.router.into_make_service())
            .await?;

        Ok(())
    }
}
