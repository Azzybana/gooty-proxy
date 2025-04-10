//! # Ownership Module
//!
//! This module provides functionality for determining ownership information
//! for IP addresses and proxy servers, including organization details and
//! Autonomous System Numbers (ASNs).
//!
//! ## Overview
//!
//! The module contains types and functions for:
//!
//! - Looking up IP address ownership through various data sources
//! - Retrieving Autonomous System Numbers (ASNs) and network information
//! - Obtaining organization details associated with IP addresses
//! - Accessing network-level metadata about IP ranges
//!
//! This information helps classify proxies by their operators, detect
//! datacenter vs residential proxies, and identify potentially malicious
//! sources.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::inspection::{OwnershipLookup, Organization};
//! use std::net::IpAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new ownership lookup service
//! let lookup = OwnershipLookup::new();
//!
//! // Look up an IP address
//! let ip: IpAddr = "8.8.8.8".parse()?;
//! let network_info = lookup.lookup(ip).await?;
//!
//! // Access organization information
//! if let Some(org) = &network_info.organization {
//!     println!("Organization: {}", org.name.as_deref().unwrap_or("Unknown"));
//!     println!("ASN: {}", org.asn.as_deref().unwrap_or("Unknown"));
//! }
//! # Ok(())
//! # }
//! ```

use crate::definitions::errors::{OwnershipError, OwnershipResult};
use crate::inspection::Location;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Represents the ownership information of an organization.
///
/// This structure contains details about organizations that own or operate
/// IP address blocks, including their name, ASN, and parent organization
/// if available.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::Organization;
///
/// let org = Organization::new(
///     Some("Google LLC".to_string()),
///     Some("15169".to_string())
/// );
///
/// assert_eq!(org.name.as_deref(), Some("Google LLC"));
/// assert_eq!(org.asn.as_deref(), Some("15169"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Organization {
    /// The name of the organization
    pub name: Option<String>,

    /// The ASN (Autonomous System Number) of the organization
    pub asn: Option<String>,

    /// The parent organization if available
    pub parent: Option<Box<Organization>>,
}

/// Network information associated with an IP address
///
/// Contains details about the network an IP address belongs to,
/// including CIDR notation, organization ownership, and geographic location.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::{NetworkInfo, Organization, Location};
///
/// // Create network info with CIDR and organization
/// let org = Organization::new(Some("Example ISP".to_string()), Some("12345".to_string()));
/// let network = NetworkInfo {
///     cidr: Some("192.168.0.0/24".to_string()),
///     organization: Some(org),
///     location: None,
/// };
///
/// assert_eq!(network.cidr.as_deref(), Some("192.168.0.0/24"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NetworkInfo {
    /// CIDR notation for the network (e.g., 192.168.1.0/24)
    pub cidr: Option<String>,

    /// Organization that owns this network
    pub organization: Option<Organization>,

    /// Location associated with this network
    pub location: Option<Location>,
}

impl Organization {
    /// Create a new Organization with the given name and ASN
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the organization
    /// * `asn` - The ASN (Autonomous System Number) of the organization
    ///
    /// # Returns
    ///
    /// A new Organization instance with the specified name and ASN
    #[must_use]
    pub fn new(name: Option<String>, asn: Option<String>) -> Self {
        Organization {
            name,
            asn,
            parent: None,
        }
    }

    /// Set the parent organization
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent organization
    ///
    /// # Returns
    ///
    /// Self with the parent organization set
    #[must_use]
    pub fn with_parent(mut self, parent: Organization) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }

    /// Check if this organization has a parent
    ///
    /// # Returns
    ///
    /// `true` if this organization has a parent, `false` otherwise
    #[must_use]
    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    /// Get the ASN as a number if it exists
    ///
    /// # Returns
    ///
    /// The ASN as a u32 if it exists and can be parsed as a number,
    /// or None if the ASN is not set or cannot be parsed
    #[must_use]
    pub fn get_asn_number(&self) -> Option<u32> {
        self.asn.as_ref().and_then(|asn| asn.parse::<u32>().ok())
    }
}

/// ASN (Autonomous System Number) information
///
/// Contains detailed information about an Autonomous System,
/// including its identifier number, owning organization, and location.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::AutonomousSystem;
///
/// // Create a new AutonomousSystem
/// let asn = AutonomousSystem {
///     number: 15169,
///     organization: Some("Google LLC".to_string()),
///     country: Some("US".to_string()),
///     description: Some("Google Global LLC".to_string()),
/// };
///
/// assert_eq!(asn.number, 15169);
/// assert_eq!(asn.organization.as_deref(), Some("Google LLC"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AutonomousSystem {
    /// The ASN number
    pub number: u32,

    /// The organization that owns this ASN
    pub organization: Option<String>,

    /// The country where this ASN is registered
    pub country: Option<String>,

    /// The description or name of the ASN
    pub description: Option<String>,
}

/// Service for looking up ASN and organization information
///
/// This service provides methods for retrieving ownership information
/// for IP addresses, including the organization, ASN, and network details.
/// It uses IP geolocation and ASN lookup services to gather this data.
///
/// # Examples
///
/// ```no_run
/// use std::net::{IpAddr, Ipv4Addr};
/// use gooty_proxy::inspection::OwnershipLookup;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let lookup = OwnershipLookup::new();
///
///     // Lookup ASN for an IP
///     let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
///     let asn = lookup.lookup_asn(&ip).await?;
///
///     println!("ASN: {:?}", asn);
///
///     // Lookup organization information
///     let org = lookup.lookup_organization(&ip).await?;
///     if let Some(org) = org {
///         println!("Organization: {:?}", org.name);
///         println!("ASN: {:?}", org.asn);
///     }
///
///     Ok(())
/// }
/// ```
pub struct OwnershipLookup {
    client: Client,
}

impl Default for OwnershipLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnershipLookup {
    /// Create a new ownership lookup service with default configuration
    ///
    /// Creates a new instance with a default HTTP client configuration,
    /// including a 10-second timeout.
    ///
    /// # Returns
    ///
    /// A new `OwnershipLookup` instance
    #[must_use]
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        OwnershipLookup { client }
    }

    /// Create a new ownership lookup service with a custom HTTP client
    ///
    /// # Arguments
    ///
    /// * `client` - A pre-configured HTTP client
    ///
    /// # Returns
    ///
    /// A new `OwnershipLookup` instance with the specified client
    #[must_use]
    pub fn with_client(client: Client) -> Self {
        OwnershipLookup { client }
    }

    /// Lookup ASN information for an IP address
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to lookup
    ///
    /// # Returns
    ///
    /// The ASN as a string if found, or None if not available
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The request to the ASN lookup service fails
    /// * The response cannot be parsed
    /// * The service returns an error status code
    pub async fn lookup_asn(&self, ip: &IpAddr) -> OwnershipResult<Option<String>> {
        // Use ipinfo.io's free API to get ASN information
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OwnershipError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(OwnershipError::NotFound(ip.to_string())),
                429 => Err(OwnershipError::RateLimited),
                _ => Err(OwnershipError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OwnershipError::ParseError(e.to_string()))?;

        let asn = data.get("org").and_then(|v| v.as_str()).and_then(|org| {
            // ASN is often prefixed in the org field like "AS15169 Google LLC"
            let parts: Vec<&str> = org.split_whitespace().collect();
            if !parts.is_empty() && parts[0].starts_with("AS") {
                Some(parts[0].trim_start_matches("AS").to_string())
            } else {
                None
            }
        });

        Ok(asn)
    }

    /// Lookup organization information for an IP address
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to lookup
    ///
    /// # Returns
    ///
    /// An Organization if information is available, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The request to the organization lookup service fails
    /// * The response cannot be parsed
    /// * The service returns an error status code
    pub async fn lookup_organization(&self, ip: &IpAddr) -> OwnershipResult<Option<Organization>> {
        // Use ipinfo.io's free API to get organization information
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OwnershipError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(OwnershipError::NotFound(ip.to_string())),
                429 => Err(OwnershipError::RateLimited),
                _ => Err(OwnershipError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OwnershipError::ParseError(e.to_string()))?;

        let org_str = data.get("org").and_then(|v| v.as_str());

        if let Some(org_str) = org_str {
            // Parse organization string like "AS15169 Google LLC"
            let parts: Vec<&str> = org_str.splitn(2, ' ').collect();
            let (asn, name) = if parts.len() == 2 && parts[0].starts_with("AS") {
                (
                    Some(parts[0].trim_start_matches("AS").to_string()),
                    Some(parts[1].to_string()),
                )
            } else {
                (None, Some(org_str.to_string()))
            };

            let org = Organization {
                name,
                asn,
                parent: None,
            };

            Ok(Some(org))
        } else {
            Ok(None)
        }
    }

    /// Try to find parent organizations and ownership chain
    ///
    /// Attempts to build a chain of organization ownership for the IP address.
    /// In this simplified implementation, it returns just the immediate organization.
    /// A more comprehensive implementation would follow ownership chains through
    /// multiple data sources.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to lookup
    ///
    /// # Returns
    ///
    /// A vector of Organizations representing the ownership chain,
    /// from direct owner to ultimate parent
    ///
    /// # Errors
    ///
    /// Returns an error if the organization lookup fails
    ///
    /// # Note
    ///
    /// This requires multiple API calls and might hit rate limits with free APIs
    pub async fn lookup_ownership_chain(&self, ip: &IpAddr) -> OwnershipResult<Vec<Organization>> {
        // This is a simplified implementation as full ownership chain lookup
        // would require premium API access or multiple data sources.
        // For now, we'll just return the immediate organization.

        let org = self.lookup_organization(ip).await?;

        match org {
            Some(org) => Ok(vec![org]),
            None => Ok(vec![]),
        }
    }

    /// Lookup detailed information about an ASN
    ///
    /// # Arguments
    ///
    /// * `asn` - The ASN to lookup, with or without the "AS" prefix
    ///
    /// # Returns
    ///
    /// Detailed information about the ASN if available, or None if not found
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The ASN is not a valid number
    /// * The request to the ASN lookup service fails
    /// * The response cannot be parsed
    /// * The service returns an error status code
    pub async fn lookup_asn_details(&self, asn: &str) -> OwnershipResult<Option<AutonomousSystem>> {
        // Remove "AS" prefix if present
        let asn_number = asn.trim_start_matches("AS");

        // Ensure it's a valid number
        let Ok(asn_num) = asn_number.parse::<u32>() else {
            return Err(OwnershipError::ParseError(format!("Invalid ASN: {asn}")));
        };

        // Use ipinfo.io's free API to get ASN information
        // Note: This is a simplified implementation as detailed ASN lookup
        // typically requires a paid API or more specific data source
        let url = format!("https://ipinfo.io/AS{asn_num}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OwnershipError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(OwnershipError::NotFound(asn.to_string())),
                429 => Err(OwnershipError::RateLimited),
                _ => Err(OwnershipError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OwnershipError::ParseError(e.to_string()))?;

        let org = data.get("name").and_then(|v| v.as_str()).map(String::from);
        let country = data
            .get("country")
            .and_then(|v| v.as_str())
            .map(String::from);
        let description = data
            .get("domain")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Only create an ASN if we have at least some information
        if org.is_some() || country.is_some() || description.is_some() {
            let asn_details = AutonomousSystem {
                number: asn_num,
                organization: org,
                country,
                description,
            };

            Ok(Some(asn_details))
        } else {
            Ok(None)
        }
    }
}
