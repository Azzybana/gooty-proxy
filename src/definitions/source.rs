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
/// A source defines where and how to obtain proxy server information, including
/// the URL to fetch from, the user agent to use in requests, and the regex pattern
/// for extracting proxy information from the response.
///
/// The struct also tracks usage statistics such as success rates and failure counts
/// to help evaluate source reliability over time.
///
/// # Examples
///
/// ```
/// use gooty_proxy::definitions::source::Source;
///
/// let source = Source::new(
///     "https://example.com/proxy-list".to_string(),
///     "Mozilla/5.0 (compatible; Gooty/1.0)".to_string(),
///     r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{2,5})".to_string(),
/// ).unwrap();
///
/// assert_eq!(source.url, "https://example.com/proxy-list");
/// assert_eq!(source.success_rate(), 0.0); // New source with no usage yet
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
    /// Creates a new proxy source with the required fields.
    ///
    /// This constructor validates both the URL and regex pattern to ensure
    /// they're well-formed before creating the source instance.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL where proxy information can be obtained
    /// * `user_agent` - The User-Agent string to use in HTTP requests
    /// * `regex_pattern` - A regular expression pattern that extracts proxy data from responses
    ///
    /// # Returns
    ///
    /// A new `Source` instance if validation succeeds
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The URL is malformed or invalid
    /// * The regex pattern is invalid or cannot be compiled
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

    /// Adds a parameter to the source configuration.
    ///
    /// Parameters will be appended to the source URL as query parameters
    /// when making HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name
    /// * `value` - The parameter value
    ///
    /// # Examples
    ///
    /// ```
    /// # use gooty_proxy::definitions::source::Source;
    /// # let mut source = Source::new(
    /// #    "https://example.com/proxies".to_string(),
    /// #    "Mozilla/5.0".to_string(),
    /// #    r"(\d+\.\d+\.\d+\.\d+:\d+)".to_string()
    /// # ).unwrap();
    /// source.add_parameter("country".to_string(), "US".to_string());
    /// source.add_parameter("type".to_string(), "https".to_string());
    ///
    /// let url = source.get_full_url();
    /// assert!(url.contains("country=US"));
    /// assert!(url.contains("type=https"));
    /// ```
    pub fn add_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// Removes a parameter from the source configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the parameter to remove
    ///
    /// # Returns
    ///
    /// The previous value of the parameter if it was set, or `None` if it wasn't present
    ///
    /// # Examples
    ///
    /// ```
    /// # use gooty_proxy::definitions::source::Source;
    /// # let mut source = Source::new(
    /// #    "https://example.com/proxies".to_string(),
    /// #    "Mozilla/5.0".to_string(),
    /// #    r"(\d+\.\d+\.\d+\.\d+:\d+)".to_string()
    /// # ).unwrap();
    /// source.add_parameter("country".to_string(), "US".to_string());
    ///
    /// let value = source.remove_parameter("country");
    /// assert_eq!(value, Some("US".to_string()));
    /// ```
    pub fn remove_parameter(&mut self, key: &str) -> Option<String> {
        self.parameters.remove(key)
    }

    /// Records a successful use of the source.
    ///
    /// This method updates usage statistics by incrementing the use count
    /// and recording the current time as the last used timestamp.
    pub fn record_use(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.use_count += 1;
    }

    /// Records a failure when using the source.
    ///
    /// This method updates failure statistics and records the reason
    /// and optional status code for the failure.
    ///
    /// # Arguments
    ///
    /// * `reason` - A description of why the source failed
    /// * `status_code` - Optional HTTP status code if the failure was related to an HTTP response
    pub fn record_failure(&mut self, reason: String, status_code: Option<u16>) {
        self.failure_count += 1;
        self.last_failure_reason = Some(reason);
        self.last_failure_code = status_code;
    }

    /// Returns the success rate of using this source.
    ///
    /// The success rate is calculated as the ratio of successful uses
    /// to total uses. If the source has never been used, returns 0.0.
    ///
    /// # Returns
    ///
    /// A float between 0.0 and 1.0 representing the success rate
    /// where 1.0 means 100% success.
    #[must_use]
    pub fn success_rate(&self) -> usize {
        if self.use_count == 0 {
            return 0;
        }

        let success_count = self.use_count - self.failure_count;
        100 * success_count / self.use_count
    }

    /// Updates the regex pattern and recompiles it.
    ///
    /// This is useful when the pattern needs to be adjusted based on
    /// changes to the source format.
    ///
    /// # Arguments
    ///
    /// * `new_pattern` - The new regex pattern to use
    ///
    /// # Returns
    ///
    /// `Ok(())` if the pattern was valid and updated successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the new regex pattern is invalid
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

    /// Validates the source configuration.
    ///
    /// This method checks that the URL is well-formed and the
    /// regex pattern is valid.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the source is valid
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The URL is invalid
    /// * The regex pattern is invalid
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

    /// Returns a constructed URL with parameters.
    ///
    /// This method takes the base URL and appends any parameters
    /// that have been added to the source as query parameters.
    ///
    /// # Returns
    ///
    /// The complete URL including query parameters
    ///
    /// # Examples
    ///
    /// ```
    /// # use gooty_proxy::definitions::source::Source;
    /// # let mut source = Source::new(
    /// #    "https://example.com/api".to_string(),
    /// #    "Mozilla/5.0".to_string(),
    /// #    r"(\d+\.\d+\.\d+\.\d+:\d+)".to_string()
    /// # ).unwrap();
    /// source.add_parameter("token".to_string(), "abc123".to_string());
    ///
    /// let url = source.get_full_url();
    /// assert_eq!(url, "https://example.com/api?token=abc123");
    /// ```
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

    /// Fetches proxies from this source.
    ///
    /// Makes an HTTP request to the source URL and extracts proxies from
    /// the response using the defined regex pattern.
    ///
    /// # Arguments
    ///
    /// * `requestor` - The HTTP client to use for making requests
    ///
    /// # Returns
    ///
    /// A vector of `Proxy` objects extracted from the source
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The HTTP request fails
    /// * The regex pattern isn't compiled properly
    /// * The response can't be parsed
    pub async fn fetch_proxies(&self, requestor: &Requestor) -> SourceResult<Vec<Proxy>> {
        let url = self.get_full_url();

        // Make the HTTP request
        let response = requestor
            .get(&url, &self.user_agent)
            .await
            .map_err(|e| SourceError::FetchFailure(e.to_string()))?;

        // Extract proxies using regex
        let Some(regex) = &self.compiled_regex else {
            return Err(SourceError::InvalidRegexPattern(
                "Regex not compiled".to_string(),
            ));
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
                    if let Some(proxy) = Self::parse_proxy(proxy_str) {
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

    /// Fetches proxies and returns both the proxies and raw response.
    ///
    /// Similar to `fetch_proxies` but also returns the raw response text,
    /// which can be useful for debugging or further processing.
    ///
    /// # Arguments
    ///
    /// * `requestor` - The HTTP client to use for making requests
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A vector of `Proxy` objects extracted from the source
    /// * The raw response text
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The HTTP request fails
    /// * The regex pattern isn't compiled properly
    /// * The response can't be parsed
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
        let Some(regex) = &self.compiled_regex else {
            return Err(SourceError::InvalidRegexPattern(
                "Regex not compiled".to_string(),
            ));
        };

        // Parse proxies from the response
        let mut proxies = Vec::new();

        let matches_iterator = regex.find_iter(&response);

        for match_result in matches_iterator {
            match match_result {
                Ok(m) => {
                    let proxy_str = m.as_str();
                    if let Some(proxy) = Self::parse_proxy(proxy_str) {
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

    /// Parse a proxy from a string match.
    ///
    /// Attempts to parse a string like "127.0.0.1:8080" into a Proxy object.
    /// Currently handles only the simple IP:PORT format.
    ///
    /// # Arguments
    ///
    /// * `proxy_str` - String containing proxy information, expected in IP:PORT format
    ///
    /// # Returns
    ///
    /// Some(Proxy) if parsing succeeds, None otherwise
    fn parse_proxy(proxy_str: &str) -> Option<Proxy> {
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
    /// Serializes the source to a JSON string.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the Source if successful
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a source from a JSON string.
    ///
    /// This method also recompiles the regex pattern after deserialization.
    ///
    /// # Arguments
    ///
    /// * `json` - A JSON string representation of a Source
    ///
    /// # Returns
    ///
    /// A Source object if deserialization succeeds
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if deserialization fails
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut source: Source = serde_json::from_str(json)?;

        // Recompile the regex after deserialization
        if let Ok(regex) = utils::SerializableRegex::new(&source.regex_pattern) {
            source.compiled_regex = Some(regex);
        }

        Ok(source)
    }
}
