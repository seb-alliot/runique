//! Automatic TLS certificate provisioning via Let's Encrypt (ACME HTTP-01).
//!
//! Enabled with the `acme` feature. Requires `ACME_ENABLED=true`, `ACME_DOMAIN`, `ACME_EMAIL`.
#![cfg(feature = "acme")]

use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder,
    OrderStatus, RetryPolicy,
};
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
        LetsEncrypt::Staging.url().to_owned()
    } else {
        LetsEncrypt::Production.url().to_owned()
    };

    tracing::info!(domain, "Starting ACME certificate provisioning");

    // 1. Create Let's Encrypt account
    let (account, _credentials) = Account::builder()?
        .create(
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
        .new_order(&NewOrder::new(&[Identifier::Dns(domain.to_string())]))
        .await?;

    // 3. Handle HTTP-01 challenges
    let mut authorizations = order.authorizations();
    while let Some(result) = authorizations.next().await {
        let mut authz = result?;
        match authz.status {
            AuthorizationStatus::Valid => continue,
            AuthorizationStatus::Pending => {}
            _ => return Err(format!("unexpected authorization status: {:?}", authz.status).into()),
        }

        let mut challenge = authz
            .challenge(ChallengeType::Http01)
            .ok_or("No HTTP-01 challenge available for this domain")?;

        let key_auth = challenge.key_authorization().as_str().to_string();
        let token = challenge.token.clone();

        challenge_store.write().await.insert(token, key_auth);

        challenge.set_ready().await?;
    }

    // 4. Wait for order to become ready
    let status = order.poll_ready(&RetryPolicy::default()).await?;
    if status != OrderStatus::Ready {
        return Err(format!("ACME order not ready: {status:?}").into());
    }

    // 5. Generate CSR + finalize — returns the private key PEM
    let key_pem = order.finalize().await?;

    // 6. Download the signed certificate chain
    let cert_pem = order.poll_certificate(&RetryPolicy::default()).await?;

    tracing::info!(domain, "Certificate obtained successfully");

    Ok((cert_pem.into_bytes(), key_pem.into_bytes()))
}
