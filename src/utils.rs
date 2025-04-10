//! # Utility Module
//!
//! This module provides common utility functions used throughout the gooty proxy system.
//! It includes validation functions, string manipulation, and other helpers.
//!
//! ## Components
//!
//! * **URL utilities** - Functions for validating and working with URLs
//! * **Regex utilities** - Functions for validating and working with regular expressions
//! * **Random generators** - Functions for generating random values
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::utils;
//!
//! let url = "https://example.com";
//! assert!(utils::is_valid_url(url));
//!
//! let regex_pattern = r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):\d+";
//! assert!(utils::validate_regex(regex_pattern).is_ok());
//! ```

use crate::definitions::{
    defaults,
    errors::{UtilError, UtilResult},
};
use fancy_regex::Regex;
use rand::prelude::*;
use serde::{self};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use url::Url;

/// A wrapper type for `fancy_regex::Regex` that implements Serialize, Deserialize, `PartialEq`, Eq
///
/// This wrapper allows storing and serializing regular expressions by storing
/// the pattern string alongside the compiled regex object.
///
/// # Examples
///
/// ```
/// use gooty_proxy::utils::SerializableRegex;
///
/// let regex = SerializableRegex::new(r"\d{3}").unwrap();
/// assert!(regex.is_match("123").unwrap());
///
/// // Serialize and deserialize with serde
/// let serialized = serde_json::to_string(&regex).unwrap();
/// let deserialized: SerializableRegex = serde_json::from_str(&serialized).unwrap();
///
/// assert_eq!(regex, deserialized);
/// ```
#[derive(Clone, Debug, serde::Serialize)]
pub struct SerializableRegex {
    /// The pattern string used to create the regex
    pattern: String,

    /// The compiled regex object
    #[serde(skip_serializing, skip_deserializing)]
    regex: Regex,
}

impl SerializableRegex {
    /// Creates a new `SerializableRegex` from a pattern string
    ///
    /// # Arguments
    ///
    /// * `pattern` - The regex pattern to compile
    ///
    /// # Returns
    ///
    /// A Result containing the `SerializableRegex` if valid
    ///
    /// # Errors
    ///
    /// Returns a `UtilError::InvalidRegex` if the pattern is invalid
    pub fn new(pattern: &str) -> UtilResult<Self> {
        let regex = validate_regex(pattern)?;
        Ok(SerializableRegex {
            pattern: pattern.to_string(),
            regex,
        })
    }

    /// Gets the underlying regex pattern
    ///
    /// # Returns
    ///
    /// The pattern string used to create this regex
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Gets a reference to the compiled regex
    ///
    /// # Returns
    ///
    /// A reference to the compiled Regex object
    #[must_use]
    pub fn regex(&self) -> &Regex {
        &self.regex
    }

    /// Checks if the given text matches this regex
    ///
    /// # Arguments
    ///
    /// * `text` - The text to check
    ///
    /// # Returns
    ///
    /// A Result containing a boolean indicating whether the pattern matches
    pub fn is_match(&self, text: &str) -> Result<bool, Box<fancy_regex::Error>> {
        self.regex.is_match(text).map_err(Box::new)
    }

    /// Finds all matches in the given text
    ///
    /// # Arguments
    ///
    /// * `text` - The text to search
    ///
    /// # Returns
    ///
    /// An iterator over all matches in the text
    #[must_use]
    pub fn find_iter<'r, 't>(&'r self, text: &'t str) -> fancy_regex::Matches<'r, 't> {
        self.regex.find_iter(text)
    }
}

impl PartialEq for SerializableRegex {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for SerializableRegex {}

impl Hash for SerializableRegex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state);
    }
}

impl fmt::Display for SerializableRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

impl FromStr for SerializableRegex {
    type Err = UtilError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SerializableRegex::new(s)
    }
}

/// Validates whether a given string is a valid URL
///
/// # Arguments
///
/// * `url` - The URL string to validate
///
/// # Returns
///
/// `true` if the URL is valid, `false` otherwise
#[must_use]
pub fn is_valid_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(parsed) => parsed.scheme() == "http" || parsed.scheme() == "https",
        Err(_) => false,
    }
}

/// Validates and compiles a regex pattern
///
/// # Arguments
///
/// * `pattern` - The regex pattern to validate
///
/// # Returns
///
/// A compiled Regex if valid, or an error if the pattern is invalid
pub fn validate_regex(pattern: &str) -> UtilResult<Regex> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex),
        Err(e) => Err(UtilError::InvalidRegex(e.to_string())),
    }
}

/// Returns a random User-Agent string from the default list
///
/// # Returns
///
/// A random User-Agent string
#[must_use]
pub fn get_random_user_agent() -> &'static str {
    let mut rng = rand::rng();
    defaults::DEFAULT_USER_AGENTS
        .choose(&mut rng)
        .unwrap_or(&"Mozilla/5.0 (compatible; Gooty-Proxy/0.1)")
}

/// Sanitizes a URL to be used as part of a filename
///
/// Removes protocol, replaces special characters, and shortens if necessary
///
/// # Arguments
///
/// * `url` - The URL to sanitize
///
/// # Returns
///
/// A string that can be safely used as part of a filename
#[must_use]
pub fn sanitize_url_for_filename(url: &str) -> String {
    // Parse the URL to extract just the host and path
    let parsed = match Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => return "invalid-url".to_string(),
    };

    // Get just the hostname
    let hostname = parsed.host_str().unwrap_or("unknown");

    // Replace special characters with hyphens
    let sanitized = hostname.replace(['.', '/', ':', '?', '&', '=', ' '], "-");

    // Limit the length to avoid excessively long filenames
    if sanitized.len() > 50 {
        sanitized[0..50].to_string()
    } else {
        sanitized
    }
}

/// Checks if a string is a valid IPv4 or IPv6 address
///
/// # Arguments
///
/// * `ip_str` - The IP address string to validate
///
/// # Returns
///
/// `true` if the string is a valid IP address, `false` otherwise
#[must_use]
pub fn is_valid_ip(ip_str: &str) -> bool {
    ip_str.parse::<std::net::IpAddr>().is_ok()
}

/// Checks if a number is a valid port number (1-65535)
///
/// # Arguments
///
/// * `port` - The port number to validate
///
/// # Returns
///
/// `true` if the port number is valid, `false` otherwise
#[must_use]
pub fn is_valid_port(port: u16) -> bool {
    port > 0
}

/// Formats bytes as human-readable sizes
///
/// # Arguments
///
/// * `bytes` - Number of bytes
///
/// # Returns
///
/// A human-readable string representing the size
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{bytes} bytes")
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}
