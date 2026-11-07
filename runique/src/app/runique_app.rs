//! Built and ready-to-launch Runique application.
use axum::Router;
use std::net::SocketAddr;
use tokio::signal;

use crate::config::RuniqueConfig;
use crate::utils::aliases::AEngine;
use crate::utils::trad::{t, tf};

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
        if crate::utils::config::env::env_flag("ACME_ENABLED", false) {
            eprintln!("⚠  {}", t("server.acme_not_compiled"));
            eprintln!("   {}", t("server.acme_add_feature"));
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

        println!("   {}", t("server.operational"));
        println!(
            "      └──>  {}",
            tf("server.launched", &[format!("http://{addr}")])
        );

        #[cfg(feature = "orm")]
        {
            let moteur_db = self.engine.db.get_database_backend();
            let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
            println!(
                "          └──>  {}",
                tf("server.connected_db", &[format!("{moteur_db:?}"), db_name])
            );
            #[cfg(feature = "test-utils")]
            eprintln!("          └──>  {}", t("server.test_utils_feature_enabled"));
            println!("              └──> {}", t("server.stop_hint"));
        }

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(
            listener,
            self.router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            signal::ctrl_c().await.expect("Error signal Ctrl+C");
            println!("\n{}", t("server.shutting_down"));
        })
        .await?;

        Ok(())
    }
}

/// Whether the certificate at `cert_path` must be replaced: missing,
/// unreadable, not issued for `domain`, or with less than a third of its
/// lifetime left (Let's Encrypt's advice, whatever the lifetime).
#[cfg(feature = "acme")]
async fn cert_needs_renewal(cert_path: &std::path::Path, domain: &str) -> bool {
    let Ok(bytes) = tokio::fs::read(cert_path).await else {
        tracing::info!("Certificate for {domain} is missing or unreadable");
        return true;
    };
    let Ok((_, perm)) = x509_parser::pem::parse_x509_pem(&bytes) else {
        tracing::info!("Certificate for {domain} is not a valid PEM");
        return true;
    };
    let Ok(cert) = perm.parse_x509() else {
        tracing::info!("Certificate for {domain} is not a valid X.509 certificate");
        return true;
    };
    let covers_domain = cert
        .subject_alternative_name()
        .ok()
        .flatten()
        .is_some_and(|san| {
            san.value.general_names.iter().any(|name| {
                matches!(name, x509_parser::extensions::GeneralName::DNSName(d)
                    if d.eq_ignore_ascii_case(domain))
            })
        });
    if !covers_domain {
        tracing::info!("Certificate for {domain} does not cover the domain");
        return true;
    }
    let validity = cert.validity();
    let not_before = validity.not_before.timestamp();
    let not_after = validity.not_after.timestamp();
    let re_new_from = not_after - (not_after - not_before) / 3;
    chrono::Utc::now().timestamp() >= re_new_from
}

/// Runs `renew` when the certificate at `cert_path` must be replaced (see
/// [`cert_needs_renewal`]). Returns whether it ran.
#[cfg(feature = "acme")]
async fn renew_if_due<F, Fut>(cert_path: &std::path::Path, domain: &str, renew: F) -> bool
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    if !cert_needs_renewal(cert_path, domain).await {
        return false;
    }
    renew().await;
    true
}

/// Obtains a certificate from Let's Encrypt and stores it in `certs_dir`.
/// Its own dates decide when it is renewed ([`cert_needs_renewal`]).
#[cfg(feature = "acme")]
async fn obtain_and_store(
    domain: &str,
    email: &str,
    challenge_store: crate::utils::acme::ChallengeStore,
    certs_dir: &str,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let (cert, key) = crate::utils::acme::obtain_certificate(
        domain,
        email,
        challenge_store,
        false, // production — use true for testing
    )
    .await?;
    tokio::fs::create_dir_all(certs_dir).await?;
    tokio::fs::write(format!("{certs_dir}/cert.pem"), &cert).await?;
    crate::utils::private_file::write(std::path::Path::new(&format!("{certs_dir}/key.pem")), &key)?;
    tracing::info!(domain, "Certificate obtained and stored");
    Ok((cert, key))
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
        // Err = un provider crypto est déjà installé (idiome rustls) → bénin, pas une
        // erreur avalée : l'objectif (un provider par défaut présent) est atteint.
        let _ = rustls::crypto::ring::default_provider().install_default();
        use crate::utils::acme::ChallengeStore;
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
            axum::serve(http_listener, http_app).await.ok();
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

        println!("   {}", t("server.operational"));
        println!(
            "      └──>  {}",
            tf("server.launched", &[format!("https://{domain}")])
        );
        #[cfg(feature = "orm")]
        {
            let moteur_db = self.engine.db.get_database_backend();
            let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
            println!(
                "          └──>  {}",
                tf("server.connected_db", &[format!("{moteur_db:?}"), db_name])
            );
            #[cfg(feature = "test-utils")]
            eprintln!("          └──>  {}", t("server.test_utils_feature_enabled"));
        }
        println!("              └──> ctrl + c to stop");

        // ConnectInfo must be propagated so `trusted_proxies` sees the real peer IP.
        // Without it, conn_ip defaults to loopback (trusted) and X-Forwarded-For
        // becomes spoofable in standalone-TLS mode (no reverse proxy).
        axum_server::bind_rustls(https_addr.parse::<std::net::SocketAddr>()?, tls_config)
            .serve(
                self.router
                    .into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await?;

        Ok(())
    }
}

#[cfg(all(test, feature = "acme"))]
mod cert_renewal_tests {
    use super::{cert_needs_renewal, renew_if_due};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const DAY: time::Duration = time::Duration::days(1);

    /// A self-signed certificate for `domain`, valid from `before` ago to
    /// `after` from now, written to `path`.
    fn write_cert(
        path: &std::path::Path,
        domain: &str,
        before: time::Duration,
        after: time::Duration,
    ) {
        let mut params = rcgen::CertificateParams::new(vec![domain.to_string()]).unwrap();
        let now = time::OffsetDateTime::now_utc();
        params.not_before = now - before;
        params.not_after = now + after;
        let key = rcgen::KeyPair::generate().unwrap();
        std::fs::write(path, params.self_signed(&key).unwrap().pem()).unwrap();
    }

    fn temp_cert() -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("rq_acme_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cert.pem");
        (dir, path)
    }

    #[tokio::test]
    async fn a_certificate_missing_unreadable_or_for_another_domain_is_renewed() {
        let (dir, path) = temp_cert();
        assert!(cert_needs_renewal(&path, "example.test").await, "missing");
        std::fs::write(&path, "not a certificate").unwrap();
        assert!(
            cert_needs_renewal(&path, "example.test").await,
            "unreadable"
        );
        write_cert(&path, "other.test", 10 * DAY, 80 * DAY);
        assert!(
            cert_needs_renewal(&path, "example.test").await,
            "another domain"
        );
        write_cert(&path, "example.test", 10 * DAY, 80 * DAY);
        assert!(
            !cert_needs_renewal(&path, "example.test").await,
            "its domain"
        );
        assert!(
            !cert_needs_renewal(&path, "EXAMPLE.test").await,
            "DNS names ignore case"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn a_certificate_is_renewed_once_a_third_of_its_life_is_left() {
        let (dir, path) = temp_cert();
        // 90 days: renewed from 30 days left.
        write_cert(
            &path,
            "example.test",
            60 * DAY,
            30 * DAY + time::Duration::minutes(2),
        );
        assert!(
            !cert_needs_renewal(&path, "example.test").await,
            "just over a third left"
        );
        write_cert(&path, "example.test", 60 * DAY, 30 * DAY);
        assert!(
            cert_needs_renewal(&path, "example.test").await,
            "a third left"
        );
        // 45 days: the rule follows the lifetime, renewed from 15 days left.
        write_cert(&path, "example.test", 20 * DAY, 25 * DAY);
        assert!(
            !cert_needs_renewal(&path, "example.test").await,
            "45 days, 25 left"
        );
        write_cert(&path, "example.test", 35 * DAY, 10 * DAY);
        assert!(
            cert_needs_renewal(&path, "example.test").await,
            "45 days, 10 left"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn the_renewal_runs_once_due_and_never_before() {
        let (dir, path) = temp_cert();
        let runs = AtomicUsize::new(0);
        let renew = || async {
            runs.fetch_add(1, Ordering::SeqCst);
        };

        write_cert(&path, "example.test", 10 * DAY, 80 * DAY);
        assert!(!renew_if_due(&path, "example.test", renew).await);
        assert_eq!(runs.load(Ordering::SeqCst), 0, "not due yet");

        write_cert(&path, "example.test", 80 * DAY, 10 * DAY);
        assert!(renew_if_due(&path, "example.test", renew).await);
        assert_eq!(runs.load(Ordering::SeqCst), 1, "due: renewed once");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
