use clap::Parser;
use derive_more::Display;
use strum_macros::EnumString;

const VAULT_ADDR: &str = "http://localhost:8200";
const VAULT_TOKEN: &str = "root";
const VAULT_PKI_MOUNT: &str = "pki";
const VAULT_PKI_CERT_ISSUER: &str = "nxlog-agent-manager-ec";
const CERTS_COUNT: usize = 100;
const VAULT_RATE_LIMIT: u32 = 0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum App {
    Gen,
    Backoff,
    Transit,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Args {
    #[arg(long, default_value = VAULT_ADDR)]
    pub vault_addr: String,
    #[arg(long, default_value = VAULT_TOKEN)]
    pub vault_token: String,
    #[arg(long, default_value = VAULT_PKI_MOUNT)]
    pub vault_pki_mount: String,
    #[arg(long, default_value = VAULT_PKI_CERT_ISSUER)]
    pub vault_pki_issuers: String,
    /// Certificates generation rate limit per second
    #[arg(long, default_value_t = VAULT_RATE_LIMIT)]
    pub vault_rate_limit: u32,
    /// Number of certificates to generate
    #[arg(long, default_value_t = CERTS_COUNT)]
    pub certs_count: usize,
    #[arg(long, default_value = "info")]
    pub logging: String,
    #[arg(long, default_value_t = App::Gen)]
    pub app: App,
}
