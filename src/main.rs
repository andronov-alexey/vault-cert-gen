use std::iter;

use anyhow::Result;
use clap::Parser;
use tokio::runtime;
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
// todoa: remove nxlog
const VAULT_PKI_CERT_ROLE: &str = "nxlog-agent-manager-rsa";
//const VAULT_PKI_CERT_ROLE: &str = "nxlog-agent-manager-ec";
const KEYS_COUNT: usize = 1000;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Args {
    #[arg(long, default_value = VAULT_ADDR)]
    vault_addr: String,
    #[arg(long, default_value = VAULT_TOKEN)]
    vault_token: String,
    #[arg(long, default_value = VAULT_PKI_MOUNT)]
    vault_pki_mount: String,
    #[arg(long, default_value = VAULT_PKI_CERT_ROLE)]
    vault_pki_role: String,
    /// Number of keys to generate
    #[arg(long, default_value_t = KEYS_COUNT)]
    keys_count: usize,
    #[arg(long, default_value = "info")]
    spec: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // This layer composing could be simplified after bug will be fixed https://github.com/tokio-rs/tracing/issues/1629
    let filter = EnvFilter::builder().parse(&args.spec).unwrap();
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main(args))
}

#[cfg(unix)]
async fn async_main(args: Args) -> Result<()> {
    // log::error!("error test message");
    // log::warn!("warn test message");
    // log::info!("info test message");
    // log::debug!("debug test message");
    // log::trace!("trace test message");
    let settings = VaultClientSettingsBuilder::default()
        .address(args.vault_addr)
        .token(args.vault_token)
        .build()?;
    let client = VaultClient::new(settings)?;

    let n = args.keys_count;
    let mount: &str = &args.vault_pki_mount;
    let role: &str = &args.vault_pki_role;
    let now = std::time::Instant::now();
    let futures = iter::repeat_with(|| generate_certificate(&client, mount, role)).take(n);
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
    mount: &str,
    role: &str,
) -> Result<GenerateCertificateResponse, ClientError> {
    let mut builder = GenerateCertificateRequestBuilder::default();
    let &mut _ = builder
        .common_name("common_name")
        .format("pem")
        .private_key_format("pkcs8");

    let resp = cert::generate(client, mount, role, Some(&mut builder)).await?;

    log::info!("Generated certificate, key type: {}", resp.private_key_type);

    Ok(resp)
}
