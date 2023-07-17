use std::iter;

use anyhow::Result;
use tokio::runtime;
use vaultrs::{
    api::pki::{
        requests::GenerateCertificateRequestBuilder, responses::GenerateCertificateResponse,
    },
    client::{VaultClient, VaultClientSettingsBuilder},
    error::ClientError,
    pki::cert,
};

const VAULT_ADDR: &str = "http://localhost:8200";
const VAULT_TOKEN: &str = "root";
const VAULT_PKI_MOUNT: &str = "pki";
// todoa: remove nxlog
const VAULT_PKI_CERT_ROLE: &str = "nxlog-agent-manager-rsa";
//const VAULT_PKI_CERT_ROLE: &str = "nxlog-agent-manager-ec";
const CERTIFICATES_COUNT: usize = 1000;

// todoa: as unit tests?
fn main() -> Result<()> {
    env_logger::init();

    let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main())
}

#[cfg(unix)]
async fn async_main() -> Result<()> {
    log::error!("error test message");
    log::warn!("warn test message");
    log::info!("info test message");
    log::debug!("debug test message");
    log::trace!("trace test message");
    let settings = VaultClientSettingsBuilder::default()
        .address(VAULT_ADDR)
        .token(VAULT_TOKEN)
        .build()?;
    let client = VaultClient::new(settings)?;

    let n = CERTIFICATES_COUNT;
    let now = std::time::Instant::now();
    let futures = iter::repeat_with(|| generate_certificate(&client)).take(n);
    let results = futures::future::join_all(futures).await;
    let errors = results.into_iter().filter(Result::is_err).count();

    let time = now.elapsed().as_secs_f64();
    let speed = n as f64 / time;
    log::info!("generating {n} keys took {time:.2}s ({speed:.2} keys/s), errors: {errors}");
    assert_eq!(errors, 0);

    Ok(())
}

pub async fn generate_certificate(
    client: &VaultClient,
) -> Result<GenerateCertificateResponse, ClientError> {
    let mut builder = GenerateCertificateRequestBuilder::default();
    let &mut _ = builder
        .common_name("common_name")
        .format("pem")
        .private_key_format("pkcs8");

    let resp = cert::generate(
        client,
        VAULT_PKI_MOUNT,
        VAULT_PKI_CERT_ROLE,
        Some(&mut builder),
    )
    .await?;

    log::info!("Generated certificate, key type: {}", resp.private_key_type);

    Ok(resp)
}
