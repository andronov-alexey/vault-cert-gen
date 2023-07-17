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

// todoa: maybe cmd args? [too much]
const VAULT_ADDR: &str = "http://localhost:8200";
const VAULT_TOKEN: &str = "root";
const VAULT_PKI_MOUNT: &str = "pki";
// todoa: remove nxlog
const VAULT_PKI_CERT_ROLE: &str = "nxlog-agent-manager-rsa";

// todoa: as unit tests?
fn main() -> Result<()> {
    // Instantiate runtime.
    let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;

    // Instantiate runtime.
    runtime.block_on(async_main())
}

#[cfg(unix)]
async fn async_main() -> Result<()> {
    // todoa: include logging lib
    // log::warn!("[root] warn");
    // log::info!("[root] info");
    // log::debug!("[root] debug");

    let settings = VaultClientSettingsBuilder::default()
        .address(VAULT_ADDR)
        .token(VAULT_TOKEN)
        .build()?;

    let client = VaultClient::new(settings)?;
    let mut n_to_times = std::collections::HashMap::new();
    for n in [1000] {
        let now = std::time::Instant::now();
        let futures = iter::repeat_with(|| generate_certificate(&client)).take(n);
        let results = futures::future::join_all(futures).await;
        let errors_count = results.into_iter().filter(Result::is_err).count();
        _ = n_to_times.insert(n, (now.elapsed().as_secs_f64(), errors_count));
    }
    let mut n_to_times = n_to_times.into_iter().collect::<Vec<_>>();
    n_to_times.sort_by(|l, r| l.partial_cmp(r).unwrap());
    #[allow(clippy::cast_precision_loss)]
    for (n, (time, errors_count)) in n_to_times {
        println!(
            "generating {n} keys took {time:.2}s ({:.2} keys/s), errors: {errors_count}",
            n as f64 / time
        );
        assert_eq!(errors_count, 0);
    }

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
    .await;

    let resp = resp.map_err(|err| {
        println!("Generate certificate error: {err}");
        err
    })?;

    println!("Generated certificate, key type: {}", resp.private_key_type);

    Ok(resp)
}
