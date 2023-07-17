use std::{
    iter,
    num::NonZeroU32,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use governor::{Jitter, Quota, RateLimiter};
use nonzero_ext::*;
use vaultrs::client::VaultClient;

use crate::{
    common::{sign, verify},
    Args,
};

pub async fn transit(args: &Args, client: VaultClient) -> Result<()> {
    let n = args.certs_count;
    let rate_limit = match args.vault_rate_limit {
        0 => nonzero!(u32::MAX),
        rate_limit => NonZeroU32::new(rate_limit).unwrap_or(nonzero!(u32::MAX)),
    };

    let data = "sdfk803fjsdf".as_bytes();
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
        log::info!("signing data chunk #{} ...", prev + 1);
        let res = sign(&client, data).await;
        match res {
            Ok(signature) => {
                log::info!("verifying data chunk #{} ...", prev + 1);
                Ok(verify(&client, data, &signature).await?)
            }
            Err(err) => Err(err),
        }
    })
    .take(n);
    let results = futures::future::join_all(futures).await;
    let errors = results.into_iter().filter(Result::is_err).count();

    let time = now.elapsed();
    let speed = n as f64 / time.as_secs_f64();
    log::info!("sign + verify {n} chunks took {time:.2?} ({speed:.2} chunks/s), rate limit: {rate_limit} chunks/s, errors: {errors}");
    assert_eq!(errors, 0);

    Ok(())
}
