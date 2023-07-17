use std::{
    cell::UnsafeCell,
    iter,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use anyhow::Result;
use backoff::{Error, ExponentialBackoff};
use tokio::sync::Semaphore;
use vaultrs::client::VaultClient;

use crate::{common::generate_certificate, Args};

pub async fn backoff(args: &Args, client: VaultClient) -> Result<()> {
    let n = args.certs_count;
    let mount: &str = &args.vault_pki_mount;
    let role: &str = &args.vault_pki_issuers;
    let rate_limit = match args.vault_rate_limit {
        0 => Semaphore::MAX_PERMITS,
        rate_limit => rate_limit as usize,
    };
    let semaphore = Semaphore::new(rate_limit);
    let gen_certs_count: AtomicUsize = AtomicUsize::new(0);

    let now = Instant::now();

    let op = || async {
        let max_attempts: usize = 3;
        let attempt = AtomicUsize::new(0);
        let cert = gen_certs_count.fetch_add(1, Ordering::Release) + 1;

        let request_start_time = Instant::now();
        log::info!("attempt to grab token start for cert #{cert:?} ...");
        let permit = semaphore.acquire().await.unwrap();
        log::info!(
            "token for cert #{cert:?} granted, waited {:?}!",
            request_start_time.elapsed()
        );
        let prev_attempt_finish_time: UnsafeCell<Option<Instant>> = UnsafeCell::new(None);

        let backoff = ExponentialBackoff::default();
        // let backoff = ExponentialBackoffBuilder::new()
        //     .with_max_elapsed_time(Some(Duration::from_nanos(1)))
        //     .build();
        let res = backoff::future::retry(backoff, || async {
            let prev = attempt.fetch_add(1, Ordering::Relaxed);
            assert_eq!(prev + 1, attempt.load(Ordering::Acquire));
            unsafe {
                match *prev_attempt_finish_time.get() {
                    Some(prev_time) => {
                        let waited = Instant::now().duration_since(prev_time);
                        log::info!("generating cert #{cert:?} (attempt #{attempt:?}, waited: {waited:?})...");
                    },
                    None => {
                        log::info!("generating cert #{cert:?} (attempt #{attempt:?})...");
                    }
                }
            };
            // log::info!("generating cert #{cert:?} (attempt #{attempt:?}, waited: {:?})...", now2.elapsed());
            let res = generate_certificate(&client, mount, role).await;
            match res {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    unsafe {
                        *prev_attempt_finish_time.get() = Some(Instant::now());
                    }
                    // todoa: replace with deadline
                    if attempt.load(Ordering::Acquire) >= max_attempts {
                        log::info!("giving up generating cert #{cert:?}");
                        Err(Error::permanent(err))
                    } else {
                        log::info!("failed but will try again generating cert #{cert:?} (attempt #{attempt:?})");
                        Err(Error::transient(err))
                    }
                }
            }
        })
        .await;

        log::info!(
            "generation for cert #{cert:?} completed, full time {:?}!",
            request_start_time.elapsed()
        );
        // log::info!("dropping permit (available: {})", semaphore.available_permits());
        drop(permit);
        res
    };

    let futures = iter::repeat_with(op).take(n);
    let results = futures::future::join_all(futures).await;
    let errors = results.into_iter().filter(Result::is_err).count();

    let time = now.elapsed();
    let speed = n as f64 / time.as_secs_f64();
    log::info!("generating {n} certs took {time:.2?} ({speed:.2} certs/s), rate limit: {rate_limit} certs/s, errors: {errors}");
    assert_eq!(errors, 0);

    Ok(())
}
