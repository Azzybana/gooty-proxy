//! # Gooty Proxy
//!
//! A robust library for discovering, testing, and managing HTTP and SOCKS proxies.
//!
//! ## Overview
//!
//! Gooty Proxy provides tools for working with proxy servers, including:
//!
//! * Discovery of proxy servers from various sources
//! * Validation and testing of proxies for connectivity and anonymity
//! * Collection of metadata about proxies (location, organization, ASN)
//! * Management of proxy pools for rotation and failover
//! * Persistence of proxy data for reuse across sessions
//!
//! The library is designed to be both easy to use for simple cases and highly
//! configurable for advanced use cases.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::ProxyManager;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new proxy manager
//!     let mut manager = ProxyManager::new()?;
//!
//!     // Initialize components
//!     manager.init_judge().await?;
//!     manager.init_sleuth()?;
//!
//!     // Add a proxy to test
//!     let proxy_url = "http://example.com:8080";
//!     manager.add_proxy_from_url(proxy_url)?;
//!
//!     // Test the proxy
//!     manager.check_proxy(proxy_url).await?;
//!
//!     // Get detailed information about the proxy
//!     manager.enrich_proxy(proxy_url).await?;
//!
//!     // Get a reference to the tested proxy
//!     if let Some(proxy) = manager.get_proxy(proxy_url) {
//!         println!("Proxy type: {}", proxy.proxy_type);
//!         println!("Anonymity: {}", proxy.anonymity);
//!         if let Some(country) = &proxy.country {
//!             println!("Country: {}", country);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

//#![allow(unsafe_code)]
#![warn(missing_docs)]

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod config;
pub mod definitions;
pub mod inspection;
pub mod io;
pub mod orchestration;
pub mod utils;

// Re-export main types for easier access
pub use config::{AppConfig, ConfigLoader};
pub use definitions::{
    defaults,
    enums::{AnonymityLevel, ProxyType},
    errors::{ConfigError, ConfigResult, ProxyError, SourceError, SourceResult},
    proxy::Proxy,
    source::Source,
};
pub use inspection::{
    Cidr, IpMetadata, Judge, Location, NetworkInfo, Organization, OwnershipLookup, Sleuth,
};
pub use io::{
    filesystem::{Filestore, FilestoreConfig},
    http::Requestor,
};
pub use orchestration::manager::{ProxyManager, ProxyStats, SourceStats};
