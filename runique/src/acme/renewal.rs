//! When a certificate is renewed, and where it is stored once obtained.

/// Whether the certificate at `cert_path` must be replaced: missing,
/// unreadable, not issued for `domain`, or with less than a third of its
/// lifetime left (Let's Encrypt's advice, whatever the lifetime).
pub(super) async fn cert_needs_renewal(cert_path: &std::path::Path, domain: &str) -> bool {
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
pub(super) async fn renew_if_due<F, Fut>(
    cert_path: &std::path::Path,
    domain: &str,
    renew: F,
) -> bool
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
pub(super) async fn obtain_and_store(
    domain: &str,
    email: &str,
    challenge_store: super::ChallengeStore,
    certs_dir: &str,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let (cert, key) = super::obtain_certificate(
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

#[cfg(test)]
mod tests {
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
