//! # Processes Module
//!
//! Provides utilities for managing processes in the Gooty Proxy system.
//!
//! ## Overview
//!
//! This module includes functionality for:
//! - Spawning and monitoring external processes
//! - Managing process lifecycles
//! - Handling process output and errors
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::orchestration::processes;
//!
//! // Example of spawning a process
//! let output = processes::run_command("ls", &["-la"]);
//! assert!(output.is_ok());
//! ```

/// Provides process management utilities for orchestration.
///
/// This module includes functions and abstractions for managing
/// processes in the orchestration layer of the application.
///
/// # Examples
///
/// ```
/// use gooty_proxy::orchestration::processes;
///
/// processes::start_process("example_process");
/// ```
use crate::definitions::{errors::ManagerResult, proxy::Proxy};
use crate::inspection::{ipinfo::Sleuth, judgement::Judge};
use crate::io::http::Requestor;
use crate::orchestration::threading;
use futures::FutureExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, warn};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Helper function to create a progress bar with consistent styling.
///
/// # Arguments
///
/// * `total` - The total number of items to process
///
/// # Returns
///
/// A configured `ProgressBar` instance ready for tracking progress.
fn create_progress_bar(total: u64) -> ProgressBar {
    let progress = ProgressBar::new(total);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .expect("Failed to create progress bar style")
            .progress_chars("##-"),
    );
    progress
}

/// Verify a batch of proxies with the judge service.
///
/// This function tests each proxy in the batch concurrently using the provided judge service,
/// updating each proxy with its anonymity level and recording success or failure.
///
/// # Arguments
///
/// * `proxies` - A mutable slice of proxies to verify
/// * `judge` - An Arc reference to the Judge service for testing proxies
/// * `concurrency` - The maximum number of concurrent verification operations
///
/// # Returns
///
/// Returns Ok(()) if the verification process completes, regardless of individual proxy results.
///
/// # Examples
///
/// ```
/// let judge = Arc::new(Judge::new().await?);
/// let mut proxies = vec![/* proxies to verify */];
/// verify_proxies(&mut proxies, &judge, 10).await?;
/// ```
pub async fn verify_proxies(
    proxies: &mut [Proxy],
    judge: &Arc<Judge>,
    concurrency: usize,
) -> ManagerResult<()> {
    if proxies.is_empty() {
        return Ok(());
    }

    let total = proxies.len();
    info!(
        "Verifying {total} proxies with concurrency {concurrency}"
    );

    // Create a progress bar and wrap in Arc for safe sharing
    let progress = Arc::new(create_progress_bar(total as u64));

    // Make a copy of the proxies for processing
    let proxy_vec: Vec<Proxy> = proxies.to_vec();

    // Set up job function using Arc-wrapped Judge for thread safety
    // This properly clones the Arc for each future without moving ownership
    let judge = Arc::clone(judge);
    let progress_clone = Arc::clone(&progress);

    let job_fn = move |mut proxy: Proxy| -> Pin<Box<dyn Future<Output = (Proxy, bool)> + Send>> {
        // Create local clones for the async block
        let judge = Arc::clone(&judge);
        let progress = Arc::clone(&progress_clone);

        // Box::pin automatically pins the future
        async move {
            let result = judge.judge_proxy(&mut proxy).await;
            // Update progress regardless of result
            progress.inc(1);

            if let Ok(anonymity) = result {
                proxy.anonymity = anonymity;
                (proxy, true)
            } else {
                proxy.record_check_failure();
                (proxy, false)
            }
        }
        .boxed()
    };

    // Use thread utility to run concurrent batch
    let results = threading::run_concurrent_batch(proxy_vec, concurrency, &job_fn).await;

    // Update the original proxies slice with results
    let mut success_count = 0;

    for (i, (updated_proxy, success)) in results.into_iter().enumerate() {
        if i < proxies.len() {
            proxies[i] = updated_proxy;
            if success {
                success_count += 1;
            }
        }
    }

    progress.finish_with_message(format!(
        "Verified {total}/{total} ({success_count} successful)"
    ));

    info!(
        "Verified {total}/{total} proxies ({success_count} successful)"
    );

    Ok(())
}

/// Enrich a batch of proxies with IP metadata.
///
/// This function adds metadata to each proxy in the batch concurrently using the provided
/// Sleuth service, updating proxies with their country, ASN, organization, and other details.
///
/// # Arguments
///
/// * `proxies` - A mutable slice of proxies to enrich with metadata
/// * `sleuth` - An Arc reference to the Sleuth service for IP lookups
/// * `concurrency` - The maximum number of concurrent enrichment operations
///
/// # Returns
///
/// Returns Ok(()) if the enrichment process completes, regardless of individual results.
///
/// # Examples
///
/// ```
/// let sleuth = Arc::new(Sleuth::new());
/// let mut proxies = vec![/* proxies to enrich */];
/// enrich_proxies(&mut proxies, &sleuth, 10).await?;
/// ```
pub async fn enrich_proxies(
    proxies: &mut [Proxy],
    sleuth: &Arc<Sleuth>,
    concurrency: usize,
) -> ManagerResult<()> {
    if proxies.is_empty() {
        return Ok(());
    }

    let total = proxies.len();
    info!(
        "Enriching {total} proxies with concurrency {concurrency}"
    );

    // Create a progress bar and wrap in Arc for safe sharing
    let progress = Arc::new(create_progress_bar(total as u64));

    // Make a copy of the proxies for processing
    let proxy_vec: Vec<Proxy> = proxies.to_vec();
    let progress_clone = Arc::clone(&progress);

    // Set up job function using Arc-wrapped Sleuth for thread safety
    // This properly clones the Arc for each future without moving ownership
    let sleuth = Arc::clone(sleuth);
    let job_fn = move |mut proxy: Proxy| -> Pin<Box<dyn Future<Output = (Proxy, bool)> + Send>> {
        // Create local clones for the async block
        let sleuth = Arc::clone(&sleuth);
        let progress = Arc::clone(&progress_clone);

        // Box::pin automatically pins the future
        async move {
            let result = sleuth.lookup_ip_metadata(&proxy.address).await;
            // Update progress regardless of result
            progress.inc(1);

            match result {
                Ok(metadata) => {
                    proxy.update_with_ip_metadata(metadata);
                    (proxy, true)
                }
                Err(_) => {
                    // No need to record failure for enrichment
                    (proxy, false)
                }
            }
        }
        .boxed()
    };

    // Use thread utility to run concurrent batch
    let results = threading::run_concurrent_batch(proxy_vec, concurrency, &job_fn).await;

    // Update the original proxies slice with results
    let mut success_count = 0;

    for (i, (updated_proxy, success)) in results.into_iter().enumerate() {
        if i < proxies.len() {
            proxies[i] = updated_proxy;
            if success {
                success_count += 1;
            }
        }
    }

    progress.finish_with_message(format!(
        "Enriched {total}/{total} ({success_count} successful)"
    ));

    info!(
        "Enriched {total}/{total} proxies ({success_count} successful)"
    );

    Ok(())
}

/// Fetch proxies from multiple sources concurrently.
///
/// This function scrapes proxies from all provided sources in parallel,
/// applying rate limiting and error handling.
///
/// # Arguments
///
/// * `sources` - Slice of Source objects to fetch proxies from
/// * `requestor` - The Requestor instance to use for HTTP requests
/// * `concurrency` - Maximum number of concurrent fetch operations
///
/// # Returns
///
/// A vector of unique proxies fetched from all sources.
///
/// # Errors
///
/// Returns an error if there's a critical failure in the fetch process.
/// Individual source failures are logged but don't cause the entire operation to fail.
pub async fn fetch_from_sources(
    sources: &[crate::definitions::source::Source],
    requestor: &Requestor,
    concurrency: usize,
) -> ManagerResult<Vec<Proxy>> {
    if sources.is_empty() {
        return Ok(Vec::new());
    }

    let total = sources.len();
    info!(
        "Fetching from {total} sources with concurrency {concurrency}"
    );

    // Create a progress bar and wrap in Arc for safe sharing
    let progress = Arc::new(create_progress_bar(total as u64));

    // Make a copy of sources for processing
    let source_vec: Vec<crate::definitions::source::Source> = sources.to_vec();

    // Arc-wrap the requestor for thread safety
    let requestor = Arc::new(requestor.clone());
    let progress_clone = Arc::clone(&progress);

    // Set up job function with proper captures
    let job_fn = move |source: crate::definitions::source::Source| -> Pin<Box<dyn Future<Output = (Vec<Proxy>, bool)> + Send>> {
        // Create local clones for the async block
        let requestor = Arc::clone(&requestor);
        let progress = Arc::clone(&progress_clone);

        // Box::pin automatically pins the future
        async move {
            let result = source.fetch_proxies(&requestor).await;
            // Update progress regardless of result
            progress.inc(1);

            match result {
                Ok(proxies) => {
                    debug!("Found {} proxies from {}", proxies.len(), source.url);
                    (proxies, true)
                }
                Err(e) => {
                    warn!("Failed to fetch from {}: {}", source.url, e);
                    (Vec::new(), false)
                }
            }
        }.boxed()
    };

    // Use thread utility to run concurrent batch
    let results = threading::run_concurrent_batch(source_vec, concurrency, &job_fn).await;

    // Collect unique proxies
    let mut all_proxies = Vec::new();
    let mut success_count = 0;
    let mut proxy_count = 0;

    for (proxies, success) in results {
        if success {
            success_count += 1;
        }
        proxy_count += proxies.len();
        all_proxies.extend(proxies);
    }

    // Remove duplicates (this is a simple approach - in a real system we'd use a more
    // efficient method like a HashSet with custom hash implementation for Proxy)
    let mut unique_proxies = Vec::new();
    for proxy in all_proxies {
        if !unique_proxies.iter().any(|p: &Proxy| {
            p.address == proxy.address && p.port == proxy.port && p.proxy_type == proxy.proxy_type
        }) {
            unique_proxies.push(proxy);
        }
    }

    progress.finish_with_message(format!(
        "Fetched from {}/{} sources ({} proxies, {} unique)",
        success_count,
        total,
        proxy_count,
        unique_proxies.len()
    ));

    info!(
        "Fetched from {}/{} sources ({} proxies, {} unique)",
        success_count,
        total,
        proxy_count,
        unique_proxies.len()
    );

    Ok(unique_proxies)
}
