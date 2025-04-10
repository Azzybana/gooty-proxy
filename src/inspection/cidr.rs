use crate::definitions::errors::{CidrError, CidrResult};
use std::net::IpAddr;

/// Represents a CIDR (Classless Inter-Domain Routing) block.
///
/// This struct provides functionality for working with IP subnet ranges,
/// including checking if IPs are contained within a block.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::Cidr;
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let cidr = Cidr::to_cidr("192.168.1.0/24").unwrap();
/// let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
/// assert!(cidr.contains(&ip));
/// ```
#[derive(Debug, Clone)]
pub struct Cidr {
    /// Base IP address of the network
    pub network_address: IpAddr,

    /// Network prefix length (subnet mask bits)
    pub prefix_length: u8,

    /// String representation of the CIDR
    pub cidr_string: String,
}

impl Cidr {
    /// Creates a new CIDR from a string representation (e.g., "192.168.0.0/24").
    ///
    /// # Arguments
    ///
    /// * `cidr_str` - A string in CIDR notation format like "192.168.0.0/24"
    ///
    /// # Returns
    ///
    /// A `CidrResult<Cidr>` which is Ok if parsing succeeded or an error
    /// if the CIDR notation is invalid.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The format is invalid (not in format "ip/prefix")
    /// * The IP address part cannot be parsed
    /// * The prefix length is invalid (>32 for IPv4 or >128 for IPv6)
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::inspection::Cidr;
    ///
    /// let cidr = Cidr::to_cidr("10.0.0.0/8").unwrap();
    /// assert_eq!(cidr.get_prefix_length(), 8);
    /// ```
    pub fn to_cidr(cidr_str: &str) -> CidrResult<Self> {
        let parts: Vec<&str> = cidr_str.split('/').collect();
        if parts.len() != 2 {
            return Err(CidrError::InvalidFormat(cidr_str.to_string()));
        }

        let ip_str = parts[0];
        let prefix_str = parts[1];

        let network_address = ip_str
            .parse::<IpAddr>()
            .map_err(|_| CidrError::InvalidIpAddress(ip_str.to_string()))?;

        let prefix_length = prefix_str
            .parse::<u8>()
            .map_err(|_| CidrError::InvalidPrefixLength(prefix_str.to_string()))?;

        // Validate prefix length based on IP version
        match network_address {
            IpAddr::V4(_) if prefix_length > 32 => {
                return Err(CidrError::InvalidPrefixLength(format!(
                    "IPv4 prefix length must be <= 32, got {}",
                    prefix_length
                )));
            }
            IpAddr::V6(_) if prefix_length > 128 => {
                return Err(CidrError::InvalidPrefixLength(format!(
                    "IPv6 prefix length must be <= 128, got {}",
                    prefix_length
                )));
            }
            _ => {}
        }

        Ok(Cidr {
            network_address,
            prefix_length,
            cidr_string: cidr_str.to_string(),
        })
    }

    /// Checks if an IP address is contained within this CIDR block.
    ///
    /// This method compares the network bits of the provided IP with the network bits
    /// of this CIDR block to determine if the IP is within the subnet.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to check
    ///
    /// # Returns
    ///
    /// `true` if the IP is within this CIDR block, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::inspection::Cidr;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let cidr = Cidr::to_cidr("192.168.1.0/24").unwrap();
    /// let ip_in = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
    /// let ip_out = IpAddr::V4(Ipv4Addr::new(192, 168, 2, 10));
    ///
    /// assert!(cidr.contains(&ip_in));
    /// assert!(!cidr.contains(&ip_out));
    /// ```
    pub fn contains(&self, ip: &IpAddr) -> bool {
        // Ensure IP versions match
        match (ip, &self.network_address) {
            (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_)) => {}
            _ => return false, // Different IP versions can't be in the same network
        }

        // For a proper implementation, we need to compare the network bits
        // This requires converting IPs to their binary representation
        match (ip, &self.network_address) {
            (IpAddr::V4(check_ip), IpAddr::V4(network)) => {
                let mask = !0u32 << (32 - self.prefix_length);
                let network_bits = u32::from(*network) & mask;
                let check_bits = u32::from(*check_ip) & mask;
                network_bits == check_bits
            }
            (IpAddr::V6(check_ip), IpAddr::V6(network)) => {
                // For IPv6, we need to work with the segments
                let segments_network = network.segments();
                let segments_check = check_ip.segments();

                // Calculate how many full 16-bit segments are covered by the prefix
                let full_segments = self.prefix_length / 16;
                let remainder_bits = self.prefix_length % 16;

                // Check full segments first
                for i in 0..full_segments as usize {
                    if segments_network[i] != segments_check[i] {
                        return false;
                    }
                }

                // Check remaining bits in the partial segment, if any
                if remainder_bits > 0 {
                    let segment_idx = full_segments as usize;
                    let mask = !0u16 << (16 - remainder_bits);
                    let network_bits = segments_network[segment_idx] & mask;
                    let check_bits = segments_check[segment_idx] & mask;
                    if network_bits != check_bits {
                        return false;
                    }
                }

                true
            }
            // This should never happen due to the earlier check
            _ => false,
        }
    }

    /// Returns the network address of the CIDR block.
    ///
    /// # Returns
    ///
    /// A reference to the network IP address.
    pub fn get_network_address(&self) -> &IpAddr {
        &self.network_address
    }

    /// Returns the prefix length of the CIDR block.
    ///
    /// # Returns
    ///
    /// The prefix length as a u8 value (e.g., 24 for a /24 network).
    pub fn get_prefix_length(&self) -> u8 {
        self.prefix_length
    }

    /// Returns the string representation of the CIDR block.
    ///
    /// # Returns
    ///
    /// The CIDR in string format like "192.168.1.0/24".
    pub fn to_string(&self) -> &str {
        &self.cidr_string
    }
}

/// Helper functions for working with CIDR notations.
///
/// This module provides utility functions for parsing and working with CIDR
/// notation strings without needing to create full CIDR objects.
pub mod helpers {
    use super::*;

    /// Extracts the network part of a CIDR notation.
    ///
    /// # Arguments
    ///
    /// * `cidr` - A string in CIDR notation format (e.g., "192.168.1.0/24")
    ///
    /// # Returns
    ///
    /// `Some(String)` containing the IP address part if valid, or `None` if invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::inspection::cidr::helpers;
    ///
    /// let ip = helpers::extract_network_from_cidr("192.168.1.0/24").unwrap();
    /// assert_eq!(ip, "192.168.1.0");
    /// ```
    pub fn extract_network_from_cidr(cidr: &str) -> Option<String> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            Some(parts[0].to_string())
        } else {
            None
        }
    }

    /// Extracts the prefix length from a CIDR notation.
    ///
    /// # Arguments
    ///
    /// * `cidr` - A string in CIDR notation format (e.g., "192.168.1.0/24")
    ///
    /// # Returns
    ///
    /// `Some(u8)` containing the prefix length if valid, or `None` if invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::inspection::cidr::helpers;
    ///
    /// let prefix = helpers::extract_prefix_from_cidr("192.168.1.0/24").unwrap();
    /// assert_eq!(prefix, 24);
    /// ```
    pub fn extract_prefix_from_cidr(cidr: &str) -> Option<u8> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            parts[1].parse().ok()
        } else {
            None
        }
    }

    /// Checks if an IP address is within a CIDR range.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address to check
    /// * `cidr` - A string in CIDR notation format (e.g., "192.168.1.0/24")
    ///
    /// # Returns
    ///
    /// `true` if the IP is within the CIDR range, `false` otherwise or if the CIDR is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use gooty_proxy::inspection::cidr::helpers;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
    /// assert!(helpers::is_ip_in_cidr(&ip, "192.168.1.0/24"));
    /// ```
    pub fn is_ip_in_cidr(ip: &IpAddr, cidr: &str) -> bool {
        match Cidr::to_cidr(cidr) {
            Ok(cidr_block) => cidr_block.contains(ip),
            Err(_) => false,
        }
    }
}
