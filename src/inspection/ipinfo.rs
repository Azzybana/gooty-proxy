//! # IP Information Module
//!
//! This module provides functionality for gathering metadata about IP addresses.
//! It includes services for retrieving hostname, network, location, and ownership information.
//!
//! ## Components
//!
//! * **Sleuth** - A struct for performing IP lookups
//! * **`IpMetadata`** - A struct for storing comprehensive IP metadata
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::inspection::Sleuth;
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let sleuth = Sleuth::new();
//!     let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
//!     let metadata = sleuth.lookup_ip_metadata(&ip).await?;
//!     println!("Hostname: {:?}", metadata.hostname);
//!     Ok(())
//! }
//! ```

use crate::definitions::errors::{SleuthError, SleuthResult};
use crate::inspection::{
    cidr,
    location::Location,
    ownership::{NetworkInfo, Organization, OwnershipLookup},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Full IP address metadata gathered by Sleuth
///
/// This struct contains comprehensive information about an IP address,
/// including its network details, geographical location, and organizational ownership.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::{IpMetadata, Sleuth};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let sleuth = Sleuth::new();
///     let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
///
///     if let Ok(metadata) = sleuth.lookup_ip_metadata(&ip).await {
///         println!("Hostname: {:?}", metadata.hostname);
///         println!("Network: {:?}", metadata.network);
///         println!("ASN: {:?}", metadata.asn);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpMetadata {
    /// IP address being analyzed
    pub ip: IpAddr,

    /// Hostname associated with the IP address, if available
    pub hostname: Option<String>,

    /// Network information for the IP address
    pub network: Option<NetworkInfo>,

    /// ASN (Autonomous System Number) specifically for the IP
    pub asn: Option<String>,
}

impl Default for IpMetadata {
    /// Creates a default `IpMetadata` instance
    ///
    /// The default instance uses 0.0.0.0 as the IP address with all other fields set to None.
    fn default() -> Self {
        IpMetadata {
            ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            hostname: None,
            network: None,
            asn: None,
        }
    }
}

/// Main Sleuth struct for performing IP lookups
///
/// The Sleuth service provides comprehensive IP intelligence by querying
/// various data sources to gather information about IP addresses. It can
/// retrieve hostname, network, location, and ownership information.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::Sleuth;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new Sleuth instance
///     let sleuth = Sleuth::new();
///
///     // Look up information for Google's DNS server
///     let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
///     let hostname = sleuth.lookup_hostname(&ip).await?;
///
///     println!("Hostname: {:?}", hostname);
///
///     Ok(())
/// }
/// ```
pub struct Sleuth {
    /// HTTP client for making API requests
    client: Client,

    /// Ownership lookup service for retrieving ASN and organization information
    ownership_lookup: OwnershipLookup,
}

impl Default for Sleuth {
    /// Creates a default Sleuth instance with standard configuration
    fn default() -> Self {
        Self::new()
    }
}

impl Sleuth {
    /// Create a new Sleuth instance with default configuration
    ///
    /// Initializes a Sleuth instance with a default HTTP client that has
    /// a 10-second timeout.
    ///
    /// # Returns
    ///
    /// A new Sleuth instance configured with default settings
    #[must_use] pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Sleuth {
            client: client.clone(),
            ownership_lookup: OwnershipLookup::with_client(client),
        }
    }

    /// Create a new Sleuth instance with a custom HTTP client
    ///
    /// Allows for custom configuration of the underlying HTTP client
    /// used for making API requests.
    ///
    /// # Arguments
    ///
    /// * `client` - Custom reqwest HTTP client
    ///
    /// # Returns
    ///
    /// A new Sleuth instance configured with the provided HTTP client
    #[must_use] pub fn with_client(client: Client) -> Self {
        Sleuth {
            client: client.clone(),
            ownership_lookup: OwnershipLookup::with_client(client),
        }
    }

    /// Lookup hostname for an IP address using DNS reverse lookup
    ///
    /// Retrieves the hostname associated with an IP address by querying
    /// the ipinfo.io API.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// The hostname if available, otherwise None
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The network request fails
    /// * The API returns an error response
    /// * The response cannot be parsed
    pub async fn lookup_hostname(&self, ip: &IpAddr) -> SleuthResult<Option<String>> {
        // Use ipinfo.io's free API to get hostname information
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SleuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(SleuthError::NotFound(ip.to_string())),
                429 => Err(SleuthError::RateLimited),
                _ => Err(SleuthError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SleuthError::ParseError(e.to_string()))?;

        let hostname = data
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(hostname)
    }

    /// Lookup CIDR range for an IP address
    ///
    /// Retrieves the CIDR (Classless Inter-Domain Routing) notation
    /// for the network block containing the specified IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// The CIDR notation if available, otherwise None
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The network request fails
    /// * The API returns an error response
    /// * The response cannot be parsed
    pub async fn lookup_cidr(&self, ip: &IpAddr) -> SleuthResult<Option<String>> {
        // Use ipinfo.io's free API to get network information
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SleuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(SleuthError::NotFound(ip.to_string())),
                429 => Err(SleuthError::RateLimited),
                _ => Err(SleuthError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SleuthError::ParseError(e.to_string()))?;

        let cidr = data.get("cidr").and_then(|v| v.as_str()).map(String::from);

        Ok(cidr)
    }

    /// Lookup ASN information for an IP address
    ///
    /// Retrieves the Autonomous System Number (ASN) associated with the IP address.
    /// This information indicates which network operator is responsible for routing
    /// traffic to this IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// The ASN if available, otherwise None
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup operation fails
    pub async fn lookup_asn(&self, ip: &IpAddr) -> SleuthResult<Option<String>> {
        self.ownership_lookup
            .lookup_asn(ip)
            .await
            .map_err(SleuthError::from)
    }

    /// Lookup organization information for an IP address
    ///
    /// Retrieves information about the organization that owns or operates
    /// the network containing the specified IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// Organization information if available, otherwise None
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup operation fails
    pub async fn lookup_organization(&self, ip: &IpAddr) -> SleuthResult<Option<Organization>> {
        self.ownership_lookup
            .lookup_organization(ip)
            .await
            .map_err(SleuthError::from)
    }

    /// Lookup location information for an IP address
    ///
    /// Retrieves geographical location information for the specified IP address,
    /// including city, region, postal code, and country.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// Location information if available, otherwise None
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The network request fails
    /// * The API returns an error response
    /// * The response cannot be parsed
    pub async fn lookup_location(&self, ip: &IpAddr) -> SleuthResult<Option<Location>> {
        // Use ipinfo.io's free API to get location information
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SleuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(SleuthError::NotFound(ip.to_string())),
                429 => Err(SleuthError::RateLimited),
                _ => Err(SleuthError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SleuthError::ParseError(e.to_string()))?;

        let city = data.get("city").and_then(|v| v.as_str()).map(String::from);
        let region = data
            .get("region")
            .and_then(|v| v.as_str())
            .map(String::from);
        let postal = data
            .get("postal")
            .and_then(|v| v.as_str())
            .map(String::from);
        let country = data
            .get("country")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Only create a location if we have at least one piece of information
        if city.is_some() || region.is_some() || postal.is_some() || country.is_some() {
            let location = Location {
                city,
                state: region,
                postal_code: postal,
                country,
                facility_name: None, // Not available from ipinfo.io free API
            };

            Ok(Some(location))
        } else {
            Ok(None)
        }
    }

    /// Get comprehensive metadata about an IP address
    ///
    /// Performs a single API call to retrieve all available information about an IP address
    /// and combines it with data from other lookups to create a comprehensive profile.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// Complete IP metadata including network, location, and ownership information
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The network request fails
    /// * The API returns an error response
    /// * The response cannot be parsed
    pub async fn lookup_ip_metadata(&self, ip: &IpAddr) -> SleuthResult<IpMetadata> {
        // Use ipinfo.io's free API to get all information in one request
        let url = format!("https://ipinfo.io/{ip}/json");

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| SleuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return match response.status().as_u16() {
                404 => Err(SleuthError::NotFound(ip.to_string())),
                429 => Err(SleuthError::RateLimited),
                _ => Err(SleuthError::ApiError(format!(
                    "Status {}",
                    response.status()
                ))),
            };
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SleuthError::ParseError(e.to_string()))?;

        let hostname = data
            .get("hostname")
            .and_then(|v| v.as_str())
            .map(String::from);

        let cidr = data.get("cidr").and_then(|v| v.as_str()).map(String::from);

        // Use the ownership lookup for organization information
        let organization = (self.lookup_organization(ip).await).unwrap_or_default();

        // Parse location information
        let city = data.get("city").and_then(|v| v.as_str()).map(String::from);
        let region = data
            .get("region")
            .and_then(|v| v.as_str())
            .map(String::from);
        let postal = data
            .get("postal")
            .and_then(|v| v.as_str())
            .map(String::from);
        let country = data
            .get("country")
            .and_then(|v| v.as_str())
            .map(String::from);

        let location =
            if city.is_some() || region.is_some() || postal.is_some() || country.is_some() {
                Some(Location {
                    city,
                    state: region,
                    postal_code: postal,
                    country,
                    facility_name: None,
                })
            } else {
                None
            };

        // Extract ASN from org field
        let asn = (self.lookup_asn(ip).await).unwrap_or_default();

        // Create network info if we have any relevant data
        let network = if cidr.is_some() || organization.is_some() || location.is_some() {
            Some(NetworkInfo {
                cidr,
                organization,
                location,
            })
        } else {
            None
        };

        Ok(IpMetadata {
            ip: *ip,
            hostname,
            network,
            asn,
        })
    }

    /// Try to find parent organizations and ownership chain
    ///
    /// Attempts to determine the ownership hierarchy for the network
    /// containing the specified IP address, potentially identifying
    /// parent organizations and subsidiaries.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to look up
    ///
    /// # Returns
    ///
    /// A vector of organizations in the ownership chain, if available
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup operation fails
    pub async fn lookup_ownership_chain(&self, ip: &IpAddr) -> SleuthResult<Vec<Organization>> {
        self.ownership_lookup
            .lookup_ownership_chain(ip)
            .await
            .map_err(SleuthError::from)
    }

    /// Check if an IP address is within a given CIDR range
    ///
    /// Determines whether an IP address is contained within a specific
    /// network range expressed in CIDR notation.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to check
    /// * `cidr_str` - The CIDR range to check against
    ///
    /// # Returns
    ///
    /// `true` if the IP is in the CIDR range, `false` otherwise
    #[must_use] pub fn is_ip_in_cidr(&self, ip: &IpAddr, cidr_str: &str) -> bool {
        cidr::helpers::is_ip_in_cidr(ip, cidr_str)
    }
}
