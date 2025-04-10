use crate::definitions::{
    enums::{AnonymityLevel, ProxyType},
    errors::ProxyError,
};
use crate::inspection::{IpMetadata, Location, NetworkInfo, Organization};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Represents a proxy server with its connection details and metadata.
///
/// This struct is used throughout the application to manage and interact with
/// proxy servers. It includes fields for the proxy's type, address, port, and
/// anonymity level, as well as methods for managing its state and statistics.
///
/// # Examples
///
/// ```
/// use gooty_proxy::definitions::Proxy;
/// use gooty_proxy::definitions::enums::{ProxyType, AnonymityLevel};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let proxy = Proxy::new(
///     ProxyType::Http,
///     IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
///     8080,
///     AnonymityLevel::Elite,
/// );
///
/// assert_eq!(proxy.proxy_type, ProxyType::Http);
/// assert_eq!(proxy.port, 8080);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proxy {
    /// The type of the proxy (e.g., HTTP, HTTPS, SOCKS4, SOCKS5).
    pub proxy_type: ProxyType,

    /// The IP address of the proxy server.
    pub address: IpAddr,

    /// The port number of the proxy server.
    pub port: u16,

    /// Optional username for authentication.
    pub username: Option<String>,

    /// Optional password for authentication.
    pub password: Option<String>,

    /// The anonymity level of the proxy.
    pub anonymity: AnonymityLevel,

    /// The country associated with the proxy, if available.
    pub country: Option<String>,

    /// The organization associated with the proxy, if available.
    pub organization: Option<String>,

    /// The ASN (Autonomous System Number) of the proxy, if available.
    pub asn: Option<String>,

    /// The hostname of the proxy, if available.
    pub hostname: Option<String>,

    /// The latency of the proxy in milliseconds, if measured.
    pub latency_ms: Option<u32>,

    /// When the proxy was added to the system.
    pub added_at: DateTime<Utc>,

    /// When the proxy was last checked for availability.
    pub last_checked_at: Option<DateTime<Utc>>,

    /// The total number of checks performed on the proxy.
    pub check_count: usize,

    /// The number of failed checks for the proxy.
    pub check_failure_count: usize,

    /// When the proxy was last used for a connection.
    pub last_used_at: Option<DateTime<Utc>>,

    /// Number of times the proxy has been used for connections.
    pub use_count: usize,

    /// Number of times connections through this proxy have failed.
    pub use_failure_count: usize,

    /// Extended network metadata for the proxy IP address.
    pub ip_metadata: Option<IpMetadata>,

    /// CIDR notation for the network the proxy belongs to.
    pub cidr: Option<String>,

    /// Optional location information for the proxy IP address.
    pub location: Option<Location>,

    /// Optional network information for the proxy IP address.
    pub network: Option<NetworkInfo>,

    /// Optional organization information for the proxy IP address.
    pub organization_info: Option<Organization>,
}

impl Proxy {
    /// Creates a new proxy with mandatory fields and default values for statistics.
    ///
    /// # Arguments
    ///
    /// * `proxy_type` - The type of proxy protocol to use (HTTP, HTTPS, SOCKS4, SOCKS5)
    /// * `address` - The IP address of the proxy server
    /// * `port` - The port number the proxy server listens on
    /// * `anonymity` - The level of anonymity provided by the proxy
    ///
    /// # Returns
    ///
    /// A new `Proxy` instance with default values for non-specified fields
    ///
    /// # Examples
    ///
    /// ```
    /// use spiderling_proxy::definitions::{
    ///     enums::{AnonymityLevel, ProxyType},
    ///     proxy::Proxy,
    /// };
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let proxy = Proxy::new(
    ///     ProxyType::Http,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
    ///     8080,
    ///     AnonymityLevel::Anonymous,
    /// );
    /// ```
    pub fn new(
        proxy_type: ProxyType,
        address: IpAddr,
        port: u16,
        anonymity: AnonymityLevel,
    ) -> Self {
        Proxy {
            proxy_type,
            address,
            port,
            username: None,
            password: None,
            anonymity,
            country: None,
            hostname: None,
            organization: None,
            latency_ms: None,
            added_at: Utc::now(),
            last_checked_at: None,
            check_count: 0,
            check_failure_count: 0,
            last_used_at: None,
            use_count: 0,
            use_failure_count: 0,
            ip_metadata: None,
            cidr: None,
            asn: None,
            location: None,
            network: None,
            organization_info: None,
        }
    }

    /// Sets authentication credentials for the proxy.
    ///
    /// # Arguments
    ///
    /// * `username` - Username for proxy authentication
    /// * `password` - Password for proxy authentication
    ///
    /// # Returns
    ///
    /// Self with authentication credentials set
    ///
    /// # Examples
    ///
    /// ```
    /// # use spiderling_proxy::definitions::{enums::{AnonymityLevel, ProxyType}, proxy::Proxy};
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// let proxy = Proxy::new(
    ///     ProxyType::Http,
    ///     IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
    ///     8080,
    ///     AnonymityLevel::Anonymous
    /// ).with_auth("username".to_string(), "password".to_string());
    /// ```
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// Sets the country for the proxy.
    ///
    /// # Arguments
    ///
    /// * `country` - The country where the proxy server is located
    ///
    /// # Returns
    ///
    /// Self with country information set
    pub fn with_country(mut self, country: String) -> Self {
        self.country = Some(country);
        self
    }

    /// Sets the hostname for the proxy.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname of the proxy server
    ///
    /// # Returns
    ///
    /// Self with hostname information set
    pub fn with_hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }

    /// Sets the organization for the proxy.
    ///
    /// # Arguments
    ///
    /// * `organization` - The organization or ISP operating the proxy
    ///
    /// # Returns
    ///
    /// Self with organization information set
    pub fn with_organization(mut self, organization: String) -> Self {
        self.organization = Some(organization);
        self
    }

    /// Validates that the proxy configuration is correct.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the proxy configuration is valid
    /// * `Err(ProxyError)` - If the proxy configuration is invalid
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The port is set to 0
    /// * Authentication is missing required fields (e.g., password is missing when username is provided for SOCKS5)
    pub fn validate(&self) -> Result<(), ProxyError> {
        // Validate port range (though u16 already ensures this)
        if self.port == 0 {
            return Err(ProxyError::InvalidPort(self.port));
        }

        // Check if authentication is provided when required
        if matches!(self.proxy_type, ProxyType::Socks5)
            && self.username.is_some()
            && self.password.is_none()
        {
            return Err(ProxyError::MissingAuthentication);
        }

        Ok(())
    }

    /// Records a successful check of the proxy
    pub fn record_check(&mut self, latency: u32) {
        self.last_checked_at = Some(Utc::now());
        self.check_count += 1;
        self.latency_ms = Some(latency);
    }

    /// Records a failed check of the proxy
    pub fn record_check_failure(&mut self) {
        self.last_checked_at = Some(Utc::now());
        self.check_count += 1;
        self.check_failure_count += 1;
    }

    /// Records a successful use of the proxy
    pub fn record_use(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.use_count += 1;
    }

    /// Records a failed use of the proxy
    pub fn record_use_failure(&mut self) {
        self.use_failure_count += 1;
    }

    /// Calculates the success rate of the proxy based on check history
    pub fn check_success_rate(&self) -> f64 {
        if self.check_count == 0 {
            return 0.0;
        }

        let success_count = self.check_count - self.check_failure_count;
        (success_count as f64) / (self.check_count as f64)
    }

    /// Calculates the success rate of the proxy based on usage history
    pub fn use_success_rate(&self) -> f64 {
        if self.use_count == 0 {
            return 0.0;
        }

        let success_count = self.use_count - self.use_failure_count;
        (success_count as f64) / (self.use_count as f64)
    }

    /// Returns a connection string representation of the proxy
    pub fn to_connection_string(&self) -> String {
        let auth_part = match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("{}:{}@", u, p),
            _ => String::new(),
        };

        format!(
            "{}://{}{}:{}",
            self.proxy_type.to_string().to_lowercase(),
            auth_part,
            self.address,
            self.port
        )
    }

    /// Updates the proxy with new information from a check
    pub fn update_metadata(
        &mut self,
        country: Option<String>,
        organization: Option<String>,
        hostname: Option<String>,
        anonymity: Option<AnonymityLevel>,
    ) {
        if let Some(c) = country {
            self.country = Some(c);
        }

        if let Some(o) = organization {
            self.organization = Some(o);
        }

        if let Some(h) = hostname {
            self.hostname = Some(h);
        }

        if let Some(a) = anonymity {
            self.anonymity = a;
        }
    }

    /// Updates the proxy with network metadata from a sleuth lookup
    pub fn update_with_ip_metadata(&mut self, metadata: IpMetadata) {
        // Update the hostname if not already set
        if self.hostname.is_none() {
            self.hostname = metadata.hostname.clone();
        }

        // Update CIDR information
        if let Some(network) = &metadata.network {
            self.cidr = network.cidr.clone();

            // Update organization name
            if let Some(org) = &network.organization {
                if let Some(name) = &org.name {
                    self.organization = Some(name.clone());
                }

                // Update ASN
                self.asn = org.asn.clone();
            }

            // Update location-based information
            if let Some(location) = &network.location {
                if let Some(country) = &location.country {
                    self.country = Some(country.clone());
                }
            }
        }

        // Store the full metadata structure
        self.ip_metadata = Some(metadata);
    }

    /// Gets the full IP metadata if available
    pub fn get_ip_metadata(&self) -> Option<&IpMetadata> {
        self.ip_metadata.as_ref()
    }
}

/// Helper functions for serialization and deserialization
impl Proxy {
    /// Serializes the proxy to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a proxy from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
