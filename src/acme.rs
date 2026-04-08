use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, anyhow};
use axum::{Router, extract::Path, routing::get};
use instant_acme::{
    Account, AccountCredentials, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder,
    RetryPolicy,
};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, info, warn};

use crate::proto::AcmeStep;

/// Coordinates graceful hand-off of port 80 between the main HTTP server and the ACME task.
///
/// When the proxy's main server is already listening on port 80, the HTTP server loop hands this
/// permit to the ACME task.  The task awaits `ready` before it tries to bind port 80 (guaranteeing
/// the main server has fully shut down), and drops `done_tx` (or sends on it) when the temporary
/// challenge listener is closed, so the main server loop can restart.
pub struct Port80Permit {
    /// Resolves once the main server has stopped and port 80 is free.
    pub ready: oneshot::Receiver<()>,
    /// Must be consumed (sent) when the ACME listener has been dropped and port 80 is released.
    pub done_tx: oneshot::Sender<()>,
}

/// Result of a successful ACME HTTP-01 certificate issuance.
pub struct AcmeCertResult {
    pub cert_pem: String,
    pub key_pem: String,
    /// JSON-serialized `AccountCredentials` for reuse on renewal.
    pub account_credentials_json: String,
}

/// Minimal subset of the Cloudflare DoH JSON response we care about.
#[derive(Debug, Deserialize)]
struct DohResponse {
    /// DNS response code (0 = NOERROR).
    #[serde(rename = "Status")]
    status: u32,
    /// Answer records; may be absent on NXDOMAIN.
    #[serde(rename = "Answer")]
    answer: Option<Vec<serde_json::Value>>,
}

/// Performs a DNS pre-flight check for `domain` using Cloudflare's DoH endpoint.
///
/// Returns `Ok(())` if the domain resolves to at least one A or AAAA record.
/// Returns `Err(...)` if the lookup fails or the
/// domain does not resolve.
async fn check_domain_resolves(domain: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client for DoH pre-flight check")?;

    // Try both A and AAAA; succeed as long as either has an answer.
    for qtype in &["A", "AAAA"] {
        let url = format!("https://1.1.1.1/dns-query?name={domain}&type={qtype}");
        let response = client
            .get(&url)
            .header("Accept", "application/dns-json")
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => match resp.json::<DohResponse>().await {
                Ok(doh) if doh.status == 0 => {
                    let has_answers = doh.answer.as_ref().is_some_and(|a| !a.is_empty());
                    if has_answers {
                        info!("DNS pre-flight: domain '{domain}' resolved ({qtype} record found)");
                        return Ok(());
                    }
                }
                Ok(doh) => {
                    debug!(
                        "DNS pre-flight: {qtype} lookup for '{domain}' returned status {}",
                        doh.status
                    );
                }
                Err(e) => {
                    warn!(
                        "DNS pre-flight: failed to parse DoH response for '{domain}' ({qtype}): {e}"
                    );
                }
            },
            Ok(resp) => {
                warn!(
                    "DNS pre-flight: DoH request for '{domain}' ({qtype}) returned HTTP {}",
                    resp.status()
                );
            }
            Err(e) => {
                warn!("DNS pre-flight: DoH request for '{domain}' ({qtype}) failed: {e}");
            }
        }
    }

    Err(anyhow!(
        "Domain '{domain}' does not resolve to any A or AAAA record. \
         Make sure your DNS is configured to point '{domain}' to this server's public IP \
         address before obtaining a Let's Encrypt certificate."
    ))
}

/// Run a full ACME HTTP-01 certificate issuance for the given domain.
///
/// - If `existing_credentials_json` is non-empty, the ACME account is restored from it.
/// - Otherwise a fresh account is created.
/// - A temporary axum server is spun up on port 80 to serve the challenge.
/// - If the proxy's main server is already on port 80, pass `port80_permit` so the function
///   waits until the main server has vacated the port before binding.
/// - Progress steps are sent on `progress_tx` as they happen; send errors are silently ignored
/// - On success, returns the certificate chain PEM, private key PEM, and
///   the (potentially refreshed) account credentials JSON.
pub async fn run_acme_http01(
    domain: String,
    existing_credentials_json: String,
    use_staging: bool,
    port80_permit: Option<Port80Permit>,
    progress_tx: mpsc::UnboundedSender<AcmeStep>,
) -> anyhow::Result<AcmeCertResult> {
    info!("Starting ACME HTTP-01 certificate issuance for domain: {domain}");
    let dir_url = if use_staging {
        info!("Using Let's Encrypt staging environment");
        LetsEncrypt::Staging.url().to_owned()
    } else {
        info!("Using Let's Encrypt production environment");
        LetsEncrypt::Production.url().to_owned()
    };

    // DNS pre-flight: verify the domain resolves before attempting ACME.
    let _ = progress_tx.send(AcmeStep::CheckingDomain);
    info!("DNS pre-flight check for domain: {domain}");
    check_domain_resolves(&domain).await?;

    let _ = progress_tx.send(AcmeStep::Connecting);

    // Restore or create account.
    let (account, credentials) = if existing_credentials_json.is_empty() {
        info!("No stored ACME account found; creating a new one with Let's Encrypt");
        let builder = Account::builder().context("Failed to create ACME account builder")?;
        info!("Registering account at ACME directory: {dir_url}");
        let (account, credentials) = builder
            .create(
                &NewAccount {
                    terms_of_service_agreed: true,
                    contact: &[],
                    only_return_existing: false,
                },
                dir_url.clone(),
                None,
            )
            .await
            .context("Failed to create ACME account")?;
        info!("ACME account registered successfully");
        (account, credentials)
    } else {
        info!("Restoring existing ACME account from stored credentials");
        let creds: AccountCredentials = serde_json::from_str(&existing_credentials_json)
            .context("Failed to deserialize stored ACME account credentials")?;
        let builder = Account::builder().context("Failed to create ACME account builder")?;
        let account = builder
            .from_credentials(creds)
            .await
            .context("Failed to restore ACME account from credentials")?;
        info!("ACME account restored successfully");
        // After restoring there are no new credentials returned - re-serialize the same ones.
        let restored_creds: AccountCredentials =
            serde_json::from_str(&existing_credentials_json)
                .context("Failed to re-deserialize ACME credentials for storage")?;
        (account, restored_creds)
    };

    let account_credentials_json =
        serde_json::to_string(&credentials).context("Failed to serialize ACME credentials")?;

    let mut order = account
        .new_order(&NewOrder::new(&[Identifier::Dns(domain.clone())]))
        .await
        .context("Failed to create ACME order")?;
    info!("ACME order placed for domain: {domain}");

    // Collect all (token, key_authorization) pairs we need to serve.
    let challenge_map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // Spin up temporary HTTP server on port 80 to serve challenges BEFORE calling
    // set_ready(), so the server is already accepting requests when LE attempts validation.
    let map_for_server = Arc::clone(&challenge_map);
    let app = Router::new().route(
        "/.well-known/acme-challenge/{token}",
        get(move |Path(token): Path<String>| {
            let map = Arc::clone(&map_for_server);
            async move {
                let map = map.lock().unwrap();
                if let Some(key_auth) = map.get(&token) {
                    debug!("Serving ACME challenge for token: {token}");
                    (
                        axum::http::StatusCode::OK,
                        [(axum::http::header::CONTENT_TYPE, "text/plain")],
                        key_auth.clone(),
                    )
                } else {
                    error!("Unknown ACME challenge token: {token}");
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [(axum::http::header::CONTENT_TYPE, "text/plain")],
                        String::new(),
                    )
                }
            }
        }),
    );

    // If the main HTTP server is on port 80, wait for it to vacate before binding.
    // We destructure the permit here so `ready` (a oneshot::Receiver) can be consumed.
    let (listener, port80_permit) = if let Some(permit) = port80_permit {
        info!("Waiting for main HTTP server to release port 80 before ACME challenge bind");
        let _ = permit.ready.await;
        info!("Port 80 released by main HTTP server; binding for ACME challenge");
        let listener = TcpListener::bind("0.0.0.0:80")
            .await
            .context("Failed to bind port 80 for ACME HTTP-01 challenge server")?;
        (listener, Some(permit.done_tx))
    } else {
        let listener = TcpListener::bind("0.0.0.0:80")
            .await
            .context("Failed to bind port 80 for ACME HTTP-01 challenge server")?;
        (listener, None::<tokio::sync::oneshot::Sender<()>>)
    };
    info!("ACME challenge server listening on port 80");

    let server_handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            error!("ACME challenge server error: {err}");
        }
    });

    // Now populate the challenge map and notify LE - server is already up.
    let mut authorizations = order.authorizations();

    while let Some(result) = authorizations.next().await {
        let mut authz = result.context("Failed to retrieve ACME authorization")?;
        let mut challenge = authz
            .challenge(ChallengeType::Http01)
            .ok_or_else(|| anyhow!("ACME server did not offer HTTP-01 challenge"))?;

        let token = challenge.token.clone();
        let key_auth = challenge.key_authorization().as_str().to_owned();

        info!("Preparing HTTP-01 challenge for domain: {domain} (token: {token})");

        {
            let mut map = challenge_map.lock().unwrap();
            map.insert(token, key_auth);
        }

        challenge
            .set_ready()
            .await
            .context("Failed to signal ACME challenge as ready")?;
        info!("HTTP-01 challenge signalled as ready; waiting for Let's Encrypt to validate");
    }

    // LE will now attempt HTTP-01 validation against our challenge server.
    let _ = progress_tx.send(AcmeStep::ValidatingDomain);
    info!("Polling Let's Encrypt for domain validation result...");

    // Wait for the order to become ready for finalization.
    let status = order
        .poll_ready(&RetryPolicy::default())
        .await
        .context("ACME order did not become ready")?;
    info!("Domain validation complete, order status: {status:?}");

    server_handle.abort();
    info!("ACME challenge server shut down; port 80 released");

    if let Some(done_tx) = port80_permit {
        let _ = done_tx.send(());
    }

    // Domain validated; finalizing order and retrieving the certificate.
    let _ = progress_tx.send(AcmeStep::IssuingCertificate);
    info!("Finalizing ACME order and requesting certificate issuance...");

    let key_pem = order
        .finalize()
        .await
        .context("Failed to finalize ACME order")?;
    info!("ACME order finalized; polling for certificate...");

    // Poll until the certificate is issued.
    let cert_pem = order
        .poll_certificate(&RetryPolicy::default())
        .await
        .context("Failed to retrieve ACME certificate")?;

    info!("ACME certificate issued successfully for domain: {domain}");

    Ok(AcmeCertResult {
        cert_pem,
        key_pem,
        account_credentials_json,
    })
}
