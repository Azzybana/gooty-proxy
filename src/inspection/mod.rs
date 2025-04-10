//! # Inspection Module
//!
//! This module provides tools for inspecting and analyzing proxies.
//! It includes utilities for IP information, CIDR block handling,
//! and proxy ownership determination.
//!
//! ## Components
//!
//! * **IP Info** - Fetches and processes IP-related data
//! * **CIDR** - Handles CIDR block operations
//! * **Ownership** - Determines proxy ownership and related metadata
//!
//! ## Overview
//!
//! This module contains components for:
//! - Analyzing IP addresses and networks using CIDR notation
//! - Gathering metadata about IP addresses and proxies
//! - Making judgements about proxy quality and anonymity
//! - Determining geographic location of proxies
//! - Looking up network ownership and autonomous system information
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::inspection::{Sleuth, Judge};
//! use std::net::IpAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a sleuth for IP metadata lookup
//! let sleuth = Sleuth::new();
//!
//! // Look up information about an IP address
//! let ip: IpAddr = "93.184.216.34".parse()?;
//! let metadata = sleuth.lookup(ip).await?;
//!
//! // Create a judge for proxy validation
//! let mut judge = Judge::new();
//! # Ok(())
//! # }
//! ```

pub mod cidr;
pub mod ipinfo;
pub mod judgement;
pub mod location;
pub mod ownership;

// Re-exports from modules
pub use cidr::Cidr;
pub use ipinfo::{IpMetadata, Sleuth};
pub use judgement::Judge;
pub use location::Location;
pub use ownership::{AutonomousSystem, NetworkInfo, Organization, OwnershipLookup};
