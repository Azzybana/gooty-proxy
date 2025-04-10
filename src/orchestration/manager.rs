//! # Manager Module
//!
//! Provides the `Manager` struct and related functionality for orchestrating tasks and resources.
//!
//! ## Overview
//!
//! This module includes:
//! - Initialization and configuration of the manager
//! - Task scheduling and execution
//! - Resource monitoring and reporting
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::orchestration::manager::Manager;
//!
//! // Create a new manager
//! let manager = Manager::new();
//! assert!(manager.is_ok());
//! ```

use crate::{
    definitions::{
        enums::{AnonymityLevel, ProxyType},
        errors::{JudgementError, ManagerError, ManagerResult, SleuthError},
        proxy::Proxy,
        source::Source,
    },
    inspection::{ipinfo::Sleuth, judgement::Judge},
    io::http::Requestor,
    orchestration::processes,
};
use ahash::AHashMap;
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::Arc;

/// Statistics about proxies managed by `ProxyManager`
#[derive(Debug, Clone)]
pub struct ProxyStats {
    /// Total number of proxies
    pub total: usize,

    /// Number of working proxies (successfully judged)
    pub working: usize,

    /// Number of proxies by anonymity level
    pub by_anonymity: HashMap<AnonymityLevel, usize>,

    /// Number of proxies by type
    pub by_type: HashMap<ProxyType, usize>,

    /// Number of proxies by country
    pub by_country: HashMap<String, usize>,

    /// Average latency of working proxies
    pub avg_latency: Option<u32>,
}

/// Statistics about sources managed by `ProxyManager`
#[derive(Debug, Clone)]
pub struct SourceStats {
    /// Total number of sources
    pub total: usize,

    /// Number of active sources
    pub active: usize,

    /// Total proxies found from all sources
    pub total_proxies_found: usize,

    /// Proxies found per source
    pub proxies_by_source: HashMap<String, usize>,
}

/// Manager for proxy and source collections with testing and enrichment capabilities.
///
/// `ProxyManager` is the central component for managing proxies and sources. It provides:
/// - Storage and retrieval of proxies and sources
/// - Testing proxies for anonymity and performance
/// - Enriching proxies with metadata (country, organization, ASN)
/// - Fetching new proxies from sources
/// - Filtering and selection of proxies based on various criteria
///
/// # Examples
///
/// ```
/// use gooty_proxy::orchestration::manager::ProxyManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new manager
///     let mut manager = ProxyManager::new()?;
///
///     // Initialize services
///     manager.init_judge().await?;
///     manager.init_sleuth()?;
///
///     // Get stats about managed proxies
///     let stats = manager.get_proxy_stats();
///     println!("Managing {} proxies, {} working", stats.total, stats.working);
///
///     Ok(())
/// }
/// ```
pub struct ProxyManager {
    /// Collection of proxies keyed by their string representation
    proxies: AHashMap<String, Proxy>,

    /// Collection of sources keyed by their identifier
    sources: AHashMap<String, Source>,

    /// HTTP request client for making requests
    requestor: Requestor,

    /// Judge for checking proxy anonymity
    judge: Option<Arc<Judge>>,

    /// IP lookup tool
    sleuth: Option<Arc<Sleuth>>,

    /// Last time the manager state was updated
    last_update_time: Option<DateTime<Utc>>,
}

impl ProxyManager {
    /// Create a new proxy manager with default configuration.
    ///
    /// This initializes the manager with empty collections and a default requestor.
    /// The judge and sleuth services must be initialized separately.
    ///
    /// # Returns
    ///
    /// A new `ProxyManager` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the requestor cannot be initialized.
    pub fn new() -> ManagerResult<Self> {
        let requestor = Requestor::new().map_err(ManagerError::RequestorError)?;

        Ok(ProxyManager {
            proxies: AHashMap::new(),
            sources: AHashMap::new(),
            requestor,
            judge: None,
            sleuth: None,
            last_update_time: None,
        })
    }

    /// Initialize the judge for proxy testing.
    ///
    /// The judge service is used to test proxies and determine their anonymity level.
    /// This must be called before using methods that test proxies.
    ///
    /// # Returns
    ///
    /// Ok(()) if the judge was successfully initialized.
    ///
    /// # Errors
    ///
    /// Returns an error if the judge service cannot be initialized.
    pub async fn init_judge(&mut self) -> ManagerResult<()> {
        let judge = Judge::new().await.map_err(ManagerError::JudgementError)?;
        self.judge = Some(Arc::new(judge));
        Ok(())
    }

    /// Initialize the sleuth for IP lookups.
    ///
    /// The sleuth service is used to lookup IP metadata such as country,
    /// organization, and ASN. This must be called before using methods
    /// that enrich proxy information.
    ///
    /// # Returns
    ///
    /// Ok(()) if the sleuth was successfully initialized.
    ///
    /// # Errors
    ///
    /// Returns an error if the sleuth service cannot be initialized.
    pub fn init_sleuth(&mut self) -> ManagerResult<()> {
        let sleuth = Sleuth::new();
        self.sleuth = Some(Arc::new(sleuth));
        Ok(())
    }

    /// Add a proxy to the manager.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy to add
    ///
    /// # Returns
    ///
    /// Returns true if the proxy was added, false if it already existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the proxy is invalid.
    pub fn add_proxy(&mut self, proxy: Proxy) -> ManagerResult<bool> {
        // Validate the proxy
        proxy.validate().map_err(ManagerError::ProxyError)?;

        // Use the connection string as a unique key
        let key = proxy.to_connection_string();

        // Check if this proxy already exists
        if self.proxies.contains_key(&key) {
            return Ok(false);
        }

        // Add the proxy
        self.proxies.insert(key, proxy);
        self.last_update_time = Some(Utc::now());
        Ok(true)
    }

    /// Add multiple proxies to the manager.
    ///
    /// # Arguments
    ///
    /// * `proxies` - Vector of proxies to add
    ///
    /// # Returns
    ///
    /// Returns the number of new proxies that were added.
    ///
    /// # Errors
    ///
    /// Returns an error if any proxy is invalid.
    pub fn add_proxies(&mut self, proxies: Vec<Proxy>) -> ManagerResult<usize> {
        let mut added_count = 0;

        for proxy in proxies {
            if self.add_proxy(proxy)? {
                added_count += 1;
            }
        }

        if added_count > 0 {
            self.last_update_time = Some(Utc::now());
        }

        Ok(added_count)
    }

    /// Get a proxy by its connection string.
    ///
    /// # Arguments
    ///
    /// * `id` - Connection string identifier of the proxy
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the proxy if found, or None if not found.
    #[must_use]
    pub fn get_proxy(&self, id: &str) -> Option<&Proxy> {
        self.proxies.get(id)
    }

    /// Get a mutable reference to a proxy by its connection string.
    ///
    /// # Arguments
    ///
    /// * `id` - Connection string identifier of the proxy
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the proxy if found, or None if not found.
    pub fn get_proxy_mut(&mut self, id: &str) -> Option<&mut Proxy> {
        self.proxies.get_mut(id)
    }

    /// Remove a proxy by its connection string.
    ///
    /// # Arguments
    ///
    /// * `id` - Connection string identifier of the proxy to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed proxy if found, or None if not found.
    pub fn remove_proxy(&mut self, id: &str) -> Option<Proxy> {
        let result = self.proxies.remove(id);
        if result.is_some() {
            self.last_update_time = Some(Utc::now());
        }
        result
    }

    /// Get the total number of proxies managed.
    ///
    /// # Returns
    ///
    /// The number of proxies in the manager.
    #[must_use]
    pub fn proxy_count(&self) -> usize {
        self.proxies.len()
    }

    /// Get all proxies as a vector of references.
    ///
    /// # Returns
    ///
    /// A vector containing references to all proxies.
    #[must_use]
    pub fn get_all_proxies(&self) -> Vec<&Proxy> {
        self.proxies.values().collect()
    }

    /// Get all proxies as owned values.
    ///
    /// # Returns
    ///
    /// A vector containing clones of all proxies.
    #[must_use]
    pub fn get_all_proxies_owned(&self) -> Vec<Proxy> {
        self.proxies.values().cloned().collect()
    }

    /// Get all proxies that match certain criteria.
    ///
    /// # Arguments
    ///
    /// * `filter_fn` - A function that returns true for proxies that should be included
    ///
    /// # Returns
    ///
    /// A vector of references to proxies that match the filter criteria.
    ///
    /// # Examples
    ///
    /// ```
    /// // Get all elite proxies
    /// let elite_proxies = manager.filter_proxies(|p| p.anonymity == AnonymityLevel::Elite);
    ///
    /// // Get all proxies with latency under 500ms
    /// let fast_proxies = manager.filter_proxies(|p| p.latency_ms.unwrap_or(u32::MAX) < 500);
    /// ```
    pub fn filter_proxies<F>(&self, filter_fn: F) -> Vec<&Proxy>
    where
        F: Fn(&Proxy) -> bool,
    {
        self.proxies.values().filter(|p| filter_fn(p)).collect()
    }

    /// Add a source to the manager.
    ///
    /// # Arguments
    ///
    /// * `source` - The source to add
    ///
    /// # Returns
    ///
    /// Returns true if the source was added, false if it already existed.
    ///
    /// # Errors
    ///
    /// Returns an error if the source is invalid.
    pub fn add_source(&mut self, source: Source) -> ManagerResult<bool> {
        // Use the source URL as a unique key
        let key = source.url.clone();

        // Check if this source already exists
        if self.sources.contains_key(&key) {
            return Ok(false);
        }

        // Add the source
        self.sources.insert(key, source);
        self.last_update_time = Some(Utc::now());
        Ok(true)
    }

    /// Add multiple sources to the manager.
    ///
    /// # Arguments
    ///
    /// * `sources` - Vector of sources to add
    ///
    /// # Returns
    ///
    /// Returns the number of new sources that were added.
    ///
    /// # Errors
    ///
    /// Returns an error if any source is invalid.
    pub fn add_sources(&mut self, sources: Vec<Source>) -> ManagerResult<usize> {
        let mut added_count = 0;

        for source in sources {
            if self.add_source(source)? {
                added_count += 1;
            }
        }

        if added_count > 0 {
            self.last_update_time = Some(Utc::now());
        }

        Ok(added_count)
    }

    /// Get a source by its URL.
    ///
    /// # Arguments
    ///
    /// * `url` - URL identifier of the source
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the source if found, or None if not found.
    #[must_use]
    pub fn get_source(&self, url: &str) -> Option<&Source> {
        self.sources.get(url)
    }

    /// Get a mutable reference to a source by its URL.
    ///
    /// # Arguments
    ///
    /// * `url` - URL identifier of the source
    ///
    /// # Returns
    ///
    /// An Option containing a mutable reference to the source if found, or None if not found.
    pub fn get_source_mut(&mut self, url: &str) -> Option<&mut Source> {
        self.sources.get_mut(url)
    }

    /// Remove a source by its URL.
    ///
    /// # Arguments
    ///
    /// * `url` - URL identifier of the source to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed source if found, or None if not found.
    pub fn remove_source(&mut self, url: &str) -> Option<Source> {
        let result = self.sources.remove(url);
        if result.is_some() {
            self.last_update_time = Some(Utc::now());
        }
        result
    }

    /// Get the total number of sources.
    ///
    /// # Returns
    ///
    /// The number of sources in the manager.
    #[must_use]
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Get all sources as a vector of references.
    ///
    /// # Returns
    ///
    /// A vector containing references to all sources.
    #[must_use]
    pub fn get_all_sources(&self) -> Vec<&Source> {
        self.sources.values().collect()
    }

    /// Get all sources as owned values.
    ///
    /// # Returns
    ///
    /// A vector containing clones of all sources.
    #[must_use]
    pub fn get_all_sources_owned(&self) -> Vec<Source> {
        self.sources.values().cloned().collect()
    }

    /// Get statistics about the managed proxies.
    ///
    /// This method calculates counts, distributions, and performance metrics
    /// for the proxies currently in the manager.
    ///
    /// # Returns
    ///
    /// A `ProxyStats` struct containing the calculated statistics.
    #[must_use]
    pub fn get_proxy_stats(&self) -> ProxyStats {
        let total = self.proxies.len();
        let mut working = 0;
        let mut by_anonymity = HashMap::new();
        let mut by_type = HashMap::new();
        let mut by_country = HashMap::new();
        let mut latency_sum = 0;
        let mut latency_count = 0;

        for proxy in self.proxies.values() {
            // Count proxies with successful checks as working
            if proxy.check_count > 0 && proxy.check_failure_count < proxy.check_count {
                working += 1;
            }

            // Count by anonymity
            *by_anonymity.entry(proxy.anonymity).or_insert(0) += 1;

            // Count by type
            *by_type.entry(proxy.proxy_type).or_insert(0) += 1;

            // Count by country
            if let Some(country) = &proxy.country {
                *by_country.entry(country.clone()).or_insert(0) += 1;
            }

            // Calculate average latency
            if let Some(latency) = proxy.latency_ms {
                latency_sum += latency;
                latency_count += 1;
            }
        }

        // Calculate average latency
        let avg_latency = if latency_count > 0 {
            Some(latency_sum / latency_count)
        } else {
            None
        };

        ProxyStats {
            total,
            working,
            by_anonymity,
            by_type,
            by_country,
            avg_latency,
        }
    }

    /// Get statistics about the managed sources.
    ///
    /// This method calculates counts and performance metrics for the
    /// sources currently in the manager.
    ///
    /// # Returns
    ///
    /// A `SourceStats` struct containing the calculated statistics.
    #[must_use]
    pub fn get_source_stats(&self) -> SourceStats {
        let total = self.sources.len();
        let mut active = 0;
        let mut total_proxies_found: usize = 0;
        let mut proxies_by_source: HashMap<String, usize> = HashMap::new();

        for source in self.sources.values() {
            if source.last_failure_reason.is_none() || source.failure_count < source.use_count / 2 {
                active += 1;
            }

            let found = source.proxies_found;
            total_proxies_found += found;
            proxies_by_source.insert(source.url.clone(), found);
        }

        SourceStats {
            total,
            active,
            total_proxies_found,
            proxies_by_source,
        }
    }

    /// Check a proxy by testing its connectivity and anonymity.
    ///
    /// # Arguments
    ///
    /// * `proxy_id` - The connection string identifier of the proxy to check
    ///
    /// # Returns
    ///
    /// Ok(()) if the check was performed (regardless of the proxy's status).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The proxy ID is invalid
    /// * The judge service is not initialized
    /// * There's a critical failure in the checking process
    pub async fn check_proxy(&mut self, proxy_id: &str) -> ManagerResult<()> {
        let judge = self.judge.clone().ok_or_else(|| {
            ManagerError::JudgementError(JudgementError::Other("Judge not initialized".to_string()))
        })?;

        let proxy = self
            .get_proxy_mut(proxy_id)
            .ok_or_else(|| ManagerError::InvalidProxyId(proxy_id.to_string()))?;

        // Create a clone of the proxy to pass to the judge
        let mut proxy_clone = proxy.clone();

        // Try to judge the proxy
        match judge.judge_proxy(&mut proxy_clone).await {
            Ok(anonymity) => {
                // Record a successful check
                proxy.record_check(proxy_clone.latency_ms.unwrap_or(0));

                // Update proxy metadata
                proxy.update_metadata(
                    proxy_clone.country,
                    proxy_clone.organization,
                    proxy_clone.hostname,
                    Some(anonymity),
                );

                self.last_update_time = Some(Utc::now());
            }
            Err(e) => {
                // Record a failed check
                proxy.record_check_failure();
                self.last_update_time = Some(Utc::now());
                warn!("Failed to judge proxy {proxy_id}: {e}");
            }
        }

        Ok(())
    }

    /// Fetch proxies from a source.
    ///
    /// # Arguments
    ///
    /// * `source_url` - The URL identifier of the source to fetch from
    ///
    /// # Returns
    ///
    /// A vector of proxies fetched from the source.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The source URL is invalid
    /// * The source fails to fetch proxies
    pub async fn fetch_from_source(&mut self, source_url: &str) -> ManagerResult<Vec<Proxy>> {
        let source = self
            .get_source_mut(source_url)
            .ok_or_else(|| ManagerError::InvalidSourceId(source_url.to_string()))?;

        // Create a clone of the source to work with
        let source_clone = source.clone();

        // Use the requestor directly
        let proxies = source_clone
            .fetch_proxies(&self.requestor)
            .await
            .map_err(ManagerError::SourceError)?;

        // Update source metadata in the original source
        let source = self
            .get_source_mut(source_url)
            .ok_or_else(|| ManagerError::InvalidSourceId(source_url.to_string()))?;
        source.last_used_at = Some(Utc::now());
        source.record_use();
        source.proxies_found += proxies.len();

        // Add proxies to the manager
        let added_count = self.add_proxies(proxies.clone())?;
        info!("Added {added_count} new proxies from source {source_url}");

        self.last_update_time = Some(Utc::now());
        Ok(proxies)
    }

    /// Enrich a proxy with IP metadata.
    ///
    /// # Arguments
    ///
    /// * `proxy_id` - The connection string identifier of the proxy to enrich
    ///
    /// # Returns
    ///
    /// Ok(()) if the enrichment was performed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The proxy ID is invalid
    /// * The sleuth service is not initialized
    /// * There's a failure in the enrichment process
    pub async fn enrich_proxy(&mut self, proxy_id: &str) -> ManagerResult<()> {
        let sleuth = self.sleuth.clone().ok_or_else(|| {
            ManagerError::SleuthError(SleuthError::ApiError("Sleuth not initialized".into()))
        })?;

        let proxy = self
            .get_proxy_mut(proxy_id)
            .ok_or_else(|| ManagerError::InvalidProxyId(proxy_id.to_string()))?;

        // Look up IP metadata
        match sleuth.lookup_ip_metadata(&proxy.address).await {
            Ok(metadata) => {
                // Update proxy with IP metadata
                proxy.update_with_ip_metadata(metadata);
                self.last_update_time = Some(Utc::now());
                debug!("Enriched proxy {proxy_id} with IP metadata");
            }
            Err(e) => {
                warn!("Failed to enrich proxy {proxy_id} with IP metadata: {e}");
                return Err(ManagerError::SleuthError(e));
            }
        }

        Ok(())
    }

    /// Get the last update time of the manager state.
    ///
    /// # Returns
    ///
    /// An Option containing the `DateTime` of the last update, or None if never updated.
    #[must_use]
    pub fn get_last_update_time(&self) -> Option<DateTime<Utc>> {
        self.last_update_time
    }

    /// Clear all proxies from the manager.
    ///
    /// This removes all proxies from the manager but keeps the sources.
    pub fn clear_proxies(&mut self) {
        if !self.proxies.is_empty() {
            self.proxies.clear();
            self.last_update_time = Some(Utc::now());
        }
    }

    /// Clear all sources from the manager.
    ///
    /// This removes all sources from the manager but keeps the proxies.
    pub fn clear_sources(&mut self) {
        if !self.sources.is_empty() {
            self.sources.clear();
            self.last_update_time = Some(Utc::now());
        }
    }

    /// Check all proxies in parallel.
    ///
    /// This method is useful for bulk verification of proxies, using
    /// concurrent processing for efficiency.
    ///
    /// # Arguments
    ///
    /// * `proxies` - A mutable slice of proxies to verify
    /// * `concurrency` - The maximum number of concurrent verification operations
    ///
    /// # Returns
    ///
    /// Ok(()) if the verification process completes.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a critical failure in the verification process.
    pub async fn check_all_proxies(
        &mut self,
        proxies: &mut [Proxy],
        concurrency: usize,
    ) -> ManagerResult<()> {
        // Ensure judge is initialized
        if self.judge.is_none() {
            self.init_judge().await?;
        }

        let judge = self.judge.clone().unwrap();

        if proxies.is_empty() {
            return Ok(());
        }

        // Use the processes module to verify proxies with progress
        processes::verify_proxies(proxies, &judge, concurrency).await?;

        self.last_update_time = Some(Utc::now());
        Ok(())
    }

    /// Enrich all proxies with IP metadata in parallel.
    ///
    /// This method is useful for bulk enrichment of proxies, using
    /// concurrent processing for efficiency.
    ///
    /// # Arguments
    ///
    /// * `proxies` - A mutable slice of proxies to enrich
    /// * `concurrency` - The maximum number of concurrent enrichment operations
    ///
    /// # Returns
    ///
    /// Ok(()) if the enrichment process completes.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a critical failure in the enrichment process.
    pub async fn enrich_all_proxies(
        &mut self,
        proxies: &mut [Proxy],
        concurrency: usize,
    ) -> ManagerResult<()> {
        // Only proceed if sleuth is initialized
        if self.sleuth.is_none() {
            self.init_sleuth()?;
        }

        let sleuth = self.sleuth.clone().unwrap();

        if proxies.is_empty() {
            return Ok(());
        }

        // Use the processes module to enrich proxies with progress
        processes::enrich_proxies(proxies, &sleuth, concurrency).await?;

        self.last_update_time = Some(Utc::now());
        Ok(())
    }

    /// Fetch proxies from all active sources in parallel.
    ///
    /// This method scrapes proxies from all active sources concurrently,
    /// handles errors gracefully, and filters out inactive or blacklisted sources.
    ///
    /// # Arguments
    ///
    /// * `concurrency` - The maximum number of concurrent fetch operations
    ///
    /// # Returns
    ///
    /// Ok(()) if the fetch process completes.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a critical failure in the fetch process.
    pub async fn fetch_from_all_sources(&mut self, concurrency: usize) -> ManagerResult<()> {
        let active_sources: Vec<Source> = self
            .sources
            .values()
            .filter(|s| s.last_failure_reason.is_none() || s.failure_count < s.use_count / 2)
            .cloned()
            .collect();

        if active_sources.is_empty() {
            info!("No active sources to fetch from");
            return Ok(());
        }

        // Use the processes module to fetch from sources
        let new_proxies =
            processes::fetch_from_sources(&active_sources, &self.requestor, concurrency).await?;

        // Add new proxies to the manager
        let added = self.add_proxies(new_proxies)?;

        // Update source metadata in the manager
        for source in active_sources {
            if let Some(s) = self.sources.get_mut(&source.url) {
                s.last_used_at = source.last_used_at;
                s.use_count = source.use_count;
                s.proxies_found = source.proxies_found;
            }
        }

        info!("Added {added} unique proxies from all sources");
        self.last_update_time = Some(Utc::now());
        Ok(())
    }

    /// Get the best proxies based on latency and success rate.
    ///
    /// This method selects the most reliable proxies based on their
    /// success rate and latency. It's useful for getting a set of
    /// high-quality proxies for critical tasks.
    ///
    /// # Arguments
    ///
    /// * `count` - The maximum number of proxies to return
    ///
    /// # Returns
    ///
    /// A vector containing references to the best proxies, ordered by quality.
    ///
    /// # Examples
    ///
    /// ```
    /// // Get the 5 best proxies for an important task
    /// let best_proxies = manager.get_best_proxies(5);
    /// ```
    #[must_use]
    pub fn get_best_proxies(&self, count: usize) -> Vec<&Proxy> {
        let mut proxies: Vec<&Proxy> = self
            .proxies
            .values()
            .filter(|p| p.check_count > 0 && p.check_success_rate() > 50)
            .collect();

        // Sort by success rate and latency
        proxies.sort_by(|a, b| {
            let a_success = a.check_success_rate();
            let b_success = b.check_success_rate();

            // Compare success rates first (higher is better)
            if a_success - b_success > 0 {
                return b_success
                    .partial_cmp(&a_success)
                    .unwrap_or(std::cmp::Ordering::Equal);
            }

            // If success rates are similar, compare latency (lower is better)
            match (a.latency_ms, b.latency_ms) {
                (Some(a_lat), Some(b_lat)) => a_lat.cmp(&b_lat),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });

        // Take the requested number of proxies
        proxies.truncate(count);
        proxies
    }
}
