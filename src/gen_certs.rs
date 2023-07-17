use std::{
    iter,
    num::NonZeroU32,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use governor::{Jitter, Quota, RateLimiter};
use nonzero_ext::*;
use vaultrs::client::VaultClient;

use crate::{common::generate_certificate, Args};

pub async fn gen_certs(args: &Args, client: VaultClient) -> Result<()> {
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
        let now2 = std::time::Instant::now();
        log::info!("attempt to grab token start ...");
        rate_limiter.until_ready_with_jitter(jitter).await;
        log::info!("token granted, waited {:?}!", now2.elapsed());

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
