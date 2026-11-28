//! The server Runique runs when it holds the certificate itself: port 80 for
//! the ACME challenge and the redirect, port 443 for the application.
use super::renewal::{cert_needs_renewal, obtain_and_store, renew_if_due};
#[cfg(feature = "orm")]
use crate::app::runique_app::connected_db_message;
use crate::{
    app::runique_app::{RuniqueApp, shutdown_signal},
    translation::{t, tf},
};
use axum::Router;

impl RuniqueApp {
    /// HTTPS server with automatic Let's Encrypt certificate (ACME HTTP-01).
    ///
    /// Flow:
    /// 1. Start HTTP server on port 80 (ACME challenge + redirect to HTTPS)
    /// 2. Obtain certificate from Let's Encrypt (or load existing from ./certs/)
    /// 3. Start HTTPS server on port 443
    pub(crate) async fn run_with_acme(self) -> Result<(), Box<dyn std::error::Error>> {
        // Err = a crypto provider is already installed (rustls idiom) → harmless, not a
        // swallowed error: the goal (a default provider is in place) is met.
        let _ = rustls::crypto::ring::default_provider().install_default();
        use super::ChallengeStore;
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
            // Same path and query in HTTPS; the host comes from the config,
            // never from the request's `Host` header.
            .fallback(move |uri: axum::http::Uri| {
                let domain = domain_clone.clone();
                async move {
                    let path = uri.path_and_query().map_or("/", |pq| pq.as_str());
                    Redirect::permanent(&format!("https://{domain}{path}"))
                }
            })
            .with_state(store_clone);

        let http_addr = format!("{ip}:80");
        let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
        tokio::spawn(async move {
            // Never silent: without it the ACME challenge can't be answered, and
            // the next renewal fails weeks later with nothing to explain it.
            use crate::log::TraceResult;
            axum::serve(http_listener, http_app).await.trace_or(
                crate::log::log_level!(middleware.https),
                tracing::Level::WARN,
                runique_i18n::keys::server::log::HTTPS_PORT80_STOPPED,
            );
        });

        // Load existing cert or run ACME flow
        let certs_dir = &self.engine.config.security.acme_certs_dir;
        let cert_str = format!("{certs_dir}/cert.pem");
        let key_str = format!("{certs_dir}/key.pem");
        let cert_path = FsPath::new(&cert_str);
        let key_path = FsPath::new(&key_str);

        let needs_renewal = !key_path.exists() || cert_needs_renewal(cert_path, &domain).await;

        let (cert_pem, key_pem) = if !needs_renewal {
            tracing::info!(certs_dir, "Loading existing TLS certificate");
            (
                tokio::fs::read(cert_path).await?,
                tokio::fs::read(key_path).await?,
            )
        } else {
            match obtain_and_store(&domain, &email, challenge_store.clone(), certs_dir).await {
                Ok(pems) => pems,
                Err(e) => {
                    // ACME failed (rate-limit, network issue, etc.)
                    // Fall back to the existing cert on disk if available
                    if cert_path.exists() && key_path.exists() {
                        tracing::warn!(
                            error = %e,
                            "ACME renewal failed — using existing certificate from ./certs/"
                        );
                        (
                            tokio::fs::read(cert_path).await?,
                            tokio::fs::read(key_path).await?,
                        )
                    } else {
                        return Err(e);
                    }
                }
            }
        };

        // Start HTTPS server on port 443
        let https_addr = format!("{ip}:443");
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem(cert_pem, key_pem).await?;

        // The certificate is checked again every 12 hours: a process running
        // past its renewal date gets the new one without a restart.
        let renewal = (
            domain.clone(),
            email,
            challenge_store,
            certs_dir.clone(),
            tls_config.clone(),
        );
        tokio::spawn(async move {
            let (domain, email, store, certs_dir, tls) = renewal;
            let cert_path = std::path::PathBuf::from(format!("{certs_dir}/cert.pem"));
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(12 * 3600));
            ticker.tick().await;
            loop {
                ticker.tick().await;
                renew_if_due(&cert_path, &domain, || async {
                    let obtained = obtain_and_store(&domain, &email, store.clone(), &certs_dir)
                        .await
                        .map_err(|e| e.to_string());
                    match obtained {
                        Ok((cert, key)) => match tls.reload_from_pem(cert, key).await {
                            Ok(()) => tracing::info!("TLS certificate renewed and reloaded"),
                            Err(e) => tracing::warn!(error = %e, "Renewed certificate not loaded — keeping the current one"),
                        },
                        Err(e) => {
                            tracing::warn!(error = %e, "ACME renewal failed — keeping the current certificate");
                        }
                    }
                })
                .await;
            }
        });

        println!("   {}", t(runique_i18n::keys::server::info::OPERATIONAL));
        println!(
            "      └──>  {}",
            tf(
                runique_i18n::keys::server::info::LAUNCHED,
                &[format!("https://{domain}")]
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
        }
        println!("              └──> ctrl + c to stop");

        // ConnectInfo must be propagated so `trusted_proxies` sees the real peer IP.
        // Without it, conn_ip defaults to loopback (trusted) and X-Forwarded-For
        // becomes spoofable in standalone-TLS mode (no reverse proxy).
        // Requests in flight get this long to finish once the stop is asked.
        let handle = axum_server::Handle::<std::net::SocketAddr>::new();
        let stop = handle.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            stop.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });
        axum_server::bind_rustls(https_addr.parse::<std::net::SocketAddr>()?, tls_config)
            .handle(handle)
            .serve(
                self.router
                    .into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await?;

        Ok(())
    }
}
