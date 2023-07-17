mod args;
mod backoff;
mod common;
mod gen_certs;
mod transit;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, layer::SubscriberExt, reload, util::SubscriberInitExt, EnvFilter};
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

use crate::{
    args::{App, Args},
    backoff::backoff,
    gen_certs::gen_certs,
    transit::transit,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // This layer composing could be simplified after bug will be fixed https://github.com/tokio-rs/tracing/issues/1629
    let filter = EnvFilter::builder().parse(&args.logging).unwrap();
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    async_main(&args).await
}

async fn async_main(args: &Args) -> Result<()> {
    let settings = VaultClientSettingsBuilder::default()
        .address(args.vault_addr.clone())
        .token(args.vault_token.clone())
        .build()?;
    let client = VaultClient::new(settings)?;

    match args.app {
        App::Gen => {
            println!("========= Generate Certificates =========");
            gen_certs(args, client).await
        }
        App::Backoff => {
            println!("========= Backoff =========");
            backoff(args, client).await
        }
        App::Transit => {
            println!("========= Transit =========");
            transit(args, client).await
        }
    }
}
