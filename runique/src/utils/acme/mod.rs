//! Automatic TLS certificate provisioning via Let's Encrypt (ACME HTTP-01).
//!
//! Enabled with the `acme` feature. Requires `ACME_ENABLED=true`, `ACME_DOMAIN`, `ACME_EMAIL`.
#![cfg(feature = "acme")]

use instant_acme::{
    Account, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, OrderStatus,
};
use rcgen::{CertificateParams, KeyPair, SanType};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Shared store: ACME token → key authorization string.
/// Populated before challenge validation, read by the HTTP-01 route handler.
pub type ChallengeStore = Arc<RwLock<HashMap<String, String>>>;

/// Runs the full ACME HTTP-01 flow for the given domain.
///
/// Returns `(cert_chain_pem, private_key_pem)` on success.
/// The caller is responsible for serving `/.well-known/acme-challenge/{token}`
/// via the provided `challenge_store` before calling this function.
pub async fn obtain_certificate(
    domain: &str,
    email: &str,
    challenge_store: ChallengeStore,
    staging: bool,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    let url = if staging {
        LetsEncrypt::Staging.url()
    } else {
        LetsEncrypt::Production.url()
    };

    tracing::info!(domain, "Starting ACME certificate provisioning");

    // 1. Create Let's Encrypt account
    let (account, _credentials) = Account::create(
        &NewAccount {
            contact: &[&format!("mailto:{email}")],
            terms_of_service_agreed: true,
            only_return_existing: false,
        },
        url,
        None,
    )
    .await?;

    // 2. Create order for the domain
    let mut order = account
        .new_order(&NewOrder {
            identifiers: &[Identifier::Dns(domain.to_string())],
        })
        .await?;

    // 3. Handle HTTP-01 challenges
    let authorizations = order.authorizations().await?;
    for authz in &authorizations {
        let challenge = authz
            .challenges
            .iter()
            .find(|c| c.r#type == ChallengeType::Http01)
            .ok_or("No HTTP-01 challenge available for this domain")?;

        let key_auth = order.key_authorization(challenge).as_str().to_string();

        // Make the token available to the HTTP challenge route
        challenge_store
            .write()
            .await
            .insert(challenge.token.clone(), key_auth);

        // Signal Let's Encrypt to validate
        order.set_challenge_ready(&challenge.url).await?;
    }

    // 4. Wait for order to become ready
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let state = order.refresh().await?;
        match state.status {
            OrderStatus::Ready => break,
            OrderStatus::Valid => break,
            OrderStatus::Invalid => return Err("ACME order invalid — check domain DNS".into()),
            _ => {
                tracing::debug!("ACME order status: {:?}, waiting…", state.status);
            }
        }
    }

    // 5. Generate private key + CSR
    let key_pair = KeyPair::generate()?;
    let mut params = CertificateParams::default();
    params.subject_alt_names = vec![SanType::DnsName(domain.try_into()?)];
    let csr = params.serialize_request(&key_pair)?;

    // 6. Finalize the order
    order.finalize(csr.der()).await?;

    // 7. Download the signed certificate chain
    let cert_chain_pem = loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        match order.certificate().await? {
            Some(cert) => break cert,
            None => {
                tracing::debug!("Certificate not yet available, retrying…");
            }
        }
    };

    tracing::info!(domain, "Certificate obtained successfully");

    let cert_pem = cert_chain_pem.into_bytes();
    let key_pem = key_pair.serialize_pem().into_bytes();

    Ok((cert_pem, key_pem))
}
