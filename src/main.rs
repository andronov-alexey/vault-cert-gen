use std::{
    iter,
    num::NonZeroU32,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use clap::Parser;
use governor::{Jitter, Quota, RateLimiter};
use nonzero_ext::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, EnvFilter};
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
const VAULT_PKI_CERT_ISSUER: &str = "nxlog-agent-manager-ec";
const CERTS_COUNT: usize = 100;
const VAULT_RATE_LIMIT: u32 = 0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Args {
    #[arg(long, default_value = VAULT_ADDR)]
    vault_addr: String,
    #[arg(long, default_value = VAULT_TOKEN)]
    vault_token: String,
    #[arg(long, default_value = VAULT_PKI_MOUNT)]
    vault_pki_mount: String,
    #[arg(long, default_value = VAULT_PKI_CERT_ISSUER)]
    vault_pki_issuers: String,
    /// Certificates generation rate limit per second
    #[arg(long, default_value_t = VAULT_RATE_LIMIT)]
    vault_rate_limit: u32,
    /// Number of certificates to generate
    #[arg(long, default_value_t = CERTS_COUNT)]
    certs_count: usize,
    #[arg(long, default_value = "info")]
    spec: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // This layer composing could be simplified after bug will be fixed https://github.com/tokio-rs/tracing/issues/1629
    let filter = EnvFilter::builder().parse(&args.spec).unwrap();
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    async_main(args).await
}

async fn async_main(args: Args) -> Result<()> {
    let settings = VaultClientSettingsBuilder::default()
        .address(args.vault_addr)
        .token(args.vault_token)
        .build()?;
    let client = VaultClient::new(settings)?;

    let n = args.certs_count;
    let mount: &str = &args.vault_pki_mount;
    let role: &str = &args.vault_pki_issuers;
    let rate_limit = match args.vault_rate_limit {
        0 => nonzero!(u32::MAX),
        rate_limit => NonZeroU32::new(rate_limit).unwrap_or(nonzero!(u32::MAX)),
    };

    let gen_certs_count: AtomicUsize = AtomicUsize::new(0);
    let quota = Quota::per_second(rate_limit);
    let rate_limiter = RateLimiter::direct(quota);
    let min = quota.replenish_interval();
    let jitter = Jitter::new(min, 3 * min);

    let now = std::time::Instant::now();
    let futures = iter::repeat_with(|| async {
        rate_limiter.until_ready_with_jitter(jitter).await;
        let prev = gen_certs_count.fetch_add(1, Ordering::Release);
        log::info!("generating cert #{} ...", prev + 1);
        generate_certificate(&client, mount, role).await
    })
    .take(n);
    let results = futures::future::join_all(futures).await;
    let errors = results.into_iter().filter(Result::is_err).count();

    let time = now.elapsed();
    let speed = n as f64 / time.as_secs_f64();
    log::info!("generating {n} certs took {time:.2?} ({speed:.2} certs/s), rate limit: {rate_limit} certs/s, errors: {errors}");
    assert_eq!(errors, 0);

    Ok(())
}

pub async fn generate_certificate(
    client: &VaultClient,
    mount: &str,
    role: &str,
) -> Result<GenerateCertificateResponse, ClientError> {
    let mut builder = GenerateCertificateRequestBuilder::default();
    let &mut _ = builder
        .common_name("common_name")
        .format("pem")
        .private_key_format("pkcs8");

    cert::generate(client, mount, role, Some(&mut builder)).await
}
