use anyhow::Result;
use tokio::runtime;

// todoa: as unit tests?
fn main() -> Result<()> {
    // Instantiate runtime.
    let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;

    // Instantiate runtime.
    runtime.block_on(async_main())
    // nx_log::info!("Stopping Agent Manager, waiting for async tasks to stop...");
    // runtime.shutdown_timeout(Duration::from_secs(2));
    // nx_log::info!("Agent Manager stopped");
    //Ok(())
}

#[cfg(unix)]
async fn async_main() -> Result<()> {
    println!("Hello, world!");
    // // Instantiate `Server` instance.
    // let server = init::configure_server(options).await?;
    //
    // // Run the `Server` on the runtime.
    // run(server, logger).await?;

    // benches
    // let mut n_to_times = std::collections::HashMap::new();
    // let key_type = PrivateKeyType::Rsa;
    // for n in [1000] {
    //     let now = std::time::Instant::now();
    //     let futures =
    //         iter::repeat_with(|| vault_pki.generate_certificate("common_name", key_type))
    //             .take(n);
    //     let results = futures::future::join_all(futures).await;
    //     let errors_count = results.into_iter().filter(Result::is_err).count();
    //     _ = n_to_times.insert(n, (now.elapsed().as_secs_f64(), errors_count));
    // }
    // let mut n_to_times = n_to_times.into_iter().collect::<Vec<_>>();
    // n_to_times.sort_by(|l, r| l.partial_cmp(r).unwrap());
    // #[allow(clippy::cast_precision_loss)]
    // for (n, (time, errors_count)) in n_to_times {
    //     println!(
    //         "generating {n} keys took {time:.2}s ({:.2} keys/s), errors: {errors_count}",
    //         n as f64 / time
    //     );
    // }
    //
    // panic!("bench completed");
    // benches

    Ok(())
}
