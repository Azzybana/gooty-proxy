//! # HTTP Module
//!
//! This module provides functionality for making HTTP requests with optional proxy support.
//! It includes features for handling timeouts, error conversions, and response validation.
//!
//! ## Components
//!
//! * **Requestor** - A struct for making HTTP requests with or without proxy support
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::io::http::Requestor;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let requestor = Requestor::new()?;
//!     let response = requestor.get("https://example.com", "Mozilla/5.0").await?;
//!     println!("Response: {}", response);
//!     Ok(())
//! }
//! ```

use crate::definitions::{
    errors::{RequestResult, RequestorError},
    proxy::Proxy,
};
use reqwest::{Client, Proxy as ReqwestProxy};
use std::time::{Duration, Instant};

/// Simple HTTP requestor with optional proxy support.
///
/// The Requestor provides methods to make HTTP requests with configurable
/// timeout settings and optional proxy support. It handles error conversions,
/// response validation, and timeouts in a consistent way.
///
/// # Examples
///
/// ```
/// use gooty_proxy::io::requestor::Requestor;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a new requestor with default timeout (30 seconds)
///     let requestor = Requestor::new()?;
///
///     // Make a GET request
///     let response = requestor.get(
///         "https://example.com",
///         "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"
///     ).await?;
///
///     println!("Response: {}", response);
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Requestor {
    /// The HTTP client for making requests
    client: Client,

    /// Request timeout duration
    timeout: Duration,
}

impl Requestor {
    /// Creates a new requestor with a default timeout of 30 seconds.
    ///
    /// # Returns
    ///
    /// A new Requestor instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new() -> Result<Self, RequestorError> {
        Self::with_timeout(30)
    }

    /// Creates a new requestor with a custom timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - The timeout duration in seconds
    ///
    /// # Returns
    ///
    /// A new Requestor instance with the specified timeout.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_timeout(timeout_secs: u64) -> Result<Self, RequestorError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        Ok(Requestor {
            client,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// Makes a GET request to the specified URL with the provided user agent.
    ///
    /// This method makes a direct GET request without using a proxy.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `user_agent` - The User-Agent header value to use
    ///
    /// # Returns
    ///
    /// The response body as a String if successful.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The request fails to send
    /// * The response has a non-success status code
    /// * The response body cannot be read as text
    /// * The request times out
    pub async fn get(&self, url: &str, user_agent: &str) -> RequestResult<String> {
        let start_time = Instant::now();

        let response = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .send()
            .await?;

        if start_time.elapsed() >= self.timeout {
            return Err(RequestorError::Timeout(self.timeout.as_secs()));
        }

        let status = response.status();
        if !status.is_success() {
            return Err(RequestorError::StatusError(status, status.to_string()));
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Makes a GET request using a proxy.
    ///
    /// This method creates a new client configured to use the specified proxy,
    /// then makes a GET request through that proxy.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `user_agent` - The User-Agent header value to use
    /// * `proxy` - The proxy to use for the request
    ///
    /// # Returns
    ///
    /// The response body as a String if successful.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The proxy configuration is invalid
    /// * The request fails to send
    /// * The response has a non-success status code
    /// * The response body cannot be read as text
    /// * The request times out
    /// * There's a proxy connection error
    pub async fn get_with_proxy(
        &self,
        url: &str,
        user_agent: &str,
        proxy: &Proxy,
    ) -> RequestResult<String> {
        // Build a client with the proxy configuration
        let proxy_url = proxy.to_connection_string();
        let mut proxy_builder = ReqwestProxy::all(&proxy_url)?;

        // Add authentication if provided
        if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
            proxy_builder = proxy_builder.basic_auth(username, password);
        }

        // Build a new client with the proxy
        let client = Client::builder()
            .proxy(proxy_builder)
            .timeout(self.timeout)
            .build()?;

        let start_time = Instant::now();

        let response = client
            .get(url)
            .header(reqwest::header::USER_AGENT, user_agent)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RequestorError::Timeout(self.timeout.as_secs())
                } else if e.is_connect() {
                    RequestorError::ProxyError(e.to_string())
                } else {
                    RequestorError::RequestError(e)
                }
            })?;

        if start_time.elapsed() >= self.timeout {
            return Err(RequestorError::Timeout(self.timeout.as_secs()));
        }

        let status = response.status();
        if !status.is_success() {
            return Err(RequestorError::StatusError(status, status.to_string()));
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Measures the latency to a URL in milliseconds.
    ///
    /// This method makes a lightweight HEAD request to the specified URL
    /// and measures how long it takes to get a response.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to measure latency to
    ///
    /// # Returns
    ///
    /// The latency in milliseconds.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to send or times out.
    pub async fn measure_latency(&self, url: &str) -> RequestResult<u32> {
        let start = Instant::now();

        // Make a HEAD request to minimize data transfer
        let _ = self.client.head(url).send().await?;

        let elapsed = start.elapsed();
        Ok(elapsed.as_millis() as u32)
    }
}
