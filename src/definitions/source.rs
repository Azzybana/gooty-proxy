//! # Source Module
//!
//! This module defines types and functionality for managing proxy sources - locations
//! from which proxy servers can be discovered and retrieved.
//!
//! ## Overview
//!
//! The module is centered around the `Source` struct, which represents an external
//! source of proxy servers. It provides functionality for:
//!
//! - Defining sources with URLs and regex patterns for proxy extraction
//! - Fetching and parsing proxy lists from various sources
//! - Tracking source reliability and performance metrics
//! - Managing source parameters for customized requests
//!
//! Sources typically represent web pages or APIs that provide lists of proxy servers,
//! which can then be validated and used throughout the application.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::definitions::source::Source;
//! use gooty_proxy::io::http::Requestor;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a source that extracts proxies in IP:PORT format
//!     let source = Source::new(
//!         "https://example.com/proxy-list".to_string(),
//!         "Mozilla/5.0 (compatible; Gooty/1.0)".to_string(),
//!         r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{2,5})".to_string(),
//!     )?;
//!
//!     // Create an HTTP client and fetch proxies
//!     let requestor = Requestor::new()?;
//!     let proxies = source.fetch_proxies(&requestor).await?;
//!
//!     println!("Found {} proxies", proxies.len());
//!     Ok(())
//! }
//! ```

use crate::definitions::{
    enums::{AnonymityLevel, ProxyType},
    errors::{SourceError, SourceResult},
    proxy::Proxy,
};
use crate::io::http::Requestor;
use crate::utils::{self, SerializableRegex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

/// Represents a source of proxy servers.
///
/// This struct is used to define and manage sources from which proxy servers
/// can be gathered. It includes fields for the source's URL, user agent, and
/// regex pattern for extracting proxy information.
///
/// # Examples
///
/// ```
/// use gooty_proxy::definitions::Source;
///
/// let source = Source::new(
///     "http://example.com/proxies".to_string(),
///     "Mozilla/5.0 (compatible; Gooty-Proxy/0.1)".to_string(),
///     r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{2,5})".to_string(),
/// );
///
/// assert_eq!(source.url, "http://example.com/proxies");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// The URL of the proxy source.
    pub url: String,

    /// The User-Agent string to use when making requests to the source.
    pub user_agent: String,

    /// The regex pattern to use for extracting proxy information from the source.
    pub regex_pattern: String,

    /// Compiled regex object for performance
    #[serde(skip)]
    pub compiled_regex: Option<SerializableRegex>,

    /// When the source was last used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Number of times the source has been used
    pub use_count: usize,

    /// Number of times the source has failed
    pub failure_count: usize,

    /// Last failure reason
    pub last_failure_reason: Option<String>,

    /// Last failure HTTP status code if applicable
    pub last_failure_code: Option<u16>,

    /// Additional parameters for the source
    pub parameters: HashMap<String, String>,

    /// Number of proxies found from this source
    pub proxies_found: usize,
}

impl Source {
    /// Creates a new source with the required fields
    ///
    /// # Errors
    ///
    /// Returns `SourceError::InvalidUrl` if the provided URL is not valid.
    /// Returns `SourceError::InvalidRegexPattern` if the regex pattern cannot be compiled.
    pub fn new(
        url: String,
        user_agent: String,
        regex_pattern: String,
    ) -> Result<Self, SourceError> {
        // Validate the URL
        if !utils::is_valid_url(&url) {
            return Err(SourceError::InvalidUrl(url));
        }

        // Validate and compile the regex
        let compiled_regex = match utils::SerializableRegex::new(&regex_pattern) {
            Ok(regex) => Some(regex),
            Err(err) => return Err(SourceError::InvalidRegexPattern(err.to_string())),
        };

        Ok(Source {
            url,
            user_agent,
            regex_pattern,
            compiled_regex,
            last_used_at: None,
            use_count: 0,
            failure_count: 0,
            last_failure_reason: None,
            last_failure_code: None,
            parameters: HashMap::new(),
            proxies_found: 0,
        })
    }

    /// Adds a parameter to the source configuration
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// Removes a parameter from the source configuration
    pub fn remove_parameter(&mut self, key: &str) -> Option<String> {
        self.parameters.remove(key)
    }

    /// Records a successful use of the source
    pub fn record_use(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.use_count += 1;
    }

    /// Records a failure when using the source
    pub fn record_failure(&mut self, reason: String, status_code: Option<u16>) {
        self.failure_count += 1;
        self.last_failure_reason = Some(reason);
        self.last_failure_code = status_code;
    }

    /// Returns the success rate of using this source
    #[must_use]
    pub fn success_rate(&self) -> usize {
        if self.use_count == 0 {
            return 0;
        }

        let success_count = self.use_count - self.failure_count;
        (success_count) / (self.use_count)
    }

    /// Updates the regex pattern and recompiles it
    pub fn update_regex_pattern(&mut self, new_pattern: String) -> Result<(), SourceError> {
        match utils::SerializableRegex::new(&new_pattern) {
            Ok(regex) => {
                self.regex_pattern = new_pattern;
                self.compiled_regex = Some(regex);
                Ok(())
            }
            Err(err) => Err(SourceError::InvalidRegexPattern(err.to_string())),
        }
    }

    /// Validates the source configuration
    pub fn validate(&self) -> Result<(), SourceError> {
        // Validate URL
        if !utils::is_valid_url(&self.url) {
            return Err(SourceError::InvalidUrl(self.url.clone()));
        }

        // Validate regex by compiling it
        match utils::SerializableRegex::new(&self.regex_pattern) {
            Ok(_) => Ok(()),
            Err(err) => Err(SourceError::InvalidRegexPattern(err.to_string())),
        }
    }

    /// Returns a constructed URL with parameters
    #[must_use]
    pub fn get_full_url(&self) -> String {
        if self.parameters.is_empty() {
            return self.url.clone();
        }

        let mut url = self.url.clone();
        let params: Vec<String> = self
            .parameters
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        if url.contains('?') {
            url.push('&');
        } else {
            url.push('?');
        }

        url.push_str(&params.join("&"));
        url
    }

    /// Fetch proxies from this source
    pub async fn fetch_proxies(&self, requestor: &Requestor) -> SourceResult<Vec<Proxy>> {
        let url = self.get_full_url();

        // Make the HTTP request
        let response = requestor
            .get(&url, &self.user_agent)
            .await
            .map_err(|e| SourceError::FetchFailure(e.to_string()))?;

        // Extract proxies using regex
        let regex = match &self.compiled_regex {
            Some(re) => re,
            None => {
                return Err(SourceError::InvalidRegexPattern(
                    "Regex not compiled".to_string(),
                ));
            }
        };

        // Parse proxies from the response
        let mut proxies = Vec::new();

        // Use the SerializableRegex's find_iter method
        let matches_iterator = regex.find_iter(&response);

        for match_result in matches_iterator {
            // Each match is a Result that needs to be handled
            match match_result {
                Ok(m) => {
                    let proxy_str = m.as_str();

                    // Try to parse the proxy string
                    if let Some(proxy) = self.parse_proxy(proxy_str) {
                        proxies.push(proxy);
                    }
                }
                Err(e) => {
                    return Err(SourceError::ParseError(e.to_string()));
                }
            }
        }

        Ok(proxies)
    }

    /// Fetch proxies and return both the proxies and raw response
    pub async fn fetch_proxies_with_response(
        &self,
        requestor: &Requestor,
    ) -> SourceResult<(Vec<Proxy>, String)> {
        let url = self.get_full_url();

        // Make the HTTP request
        let response = requestor
            .get(&url, &self.user_agent)
            .await
            .map_err(|e| SourceError::FetchFailure(e.to_string()))?;

        // Extract proxies using regex
        let regex = match &self.compiled_regex {
            Some(re) => re,
            None => {
                return Err(SourceError::InvalidRegexPattern(
                    "Regex not compiled".to_string(),
                ));
            }
        };

        // Parse proxies from the response
        let mut proxies = Vec::new();

        let matches_iterator = regex.find_iter(&response);

        for match_result in matches_iterator {
            match match_result {
                Ok(m) => {
                    let proxy_str = m.as_str();
                    if let Some(proxy) = self.parse_proxy(proxy_str) {
                        proxies.push(proxy);
                    }
                }
                Err(e) => {
                    return Err(SourceError::ParseError(e.to_string()));
                }
            }
        }

        Ok((proxies, response))
    }

    /// Parse a proxy from a string match
    fn parse_proxy(&self, proxy_str: &str) -> Option<Proxy> {
        // Simple IP:PORT parsing
        if let Some((ip_str, port_str)) = proxy_str.split_once(':') {
            if let (Ok(ip), Ok(port)) = (IpAddr::from_str(ip_str), port_str.parse::<u16>()) {
                // Default to HTTP proxy type if not specified
                return Some(Proxy::new(
                    ProxyType::Http,
                    ip,
                    port,
                    AnonymityLevel::Anonymous, // Default anonymity level, will be checked later
                ));
            }
        }

        None
    }
}

/// Functions for serialization and deserialization
impl Source {
    /// Serializes the source to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a source from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut source: Source = serde_json::from_str(json)?;

        // Recompile the regex after deserialization
        if let Ok(regex) = utils::SerializableRegex::new(&source.regex_pattern) {
            source.compiled_regex = Some(regex);
        }

        Ok(source)
    }
}
