//! # Judgement Module
//!
//! This module provides functionality for judging proxies to determine their anonymity level.
//! It includes services for testing proxies against judge services and analyzing their responses.
//!
//! ## Components
//!
//! * **Judge** - A struct for determining the anonymity level of proxies
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::inspection::Judge;
//! use gooty_proxy::definitions::proxy::Proxy;
//! use gooty_proxy::definitions::enums::{AnonymityLevel, ProxyType};
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let judge = Judge::new().await?;
//!     let mut proxy = Proxy::new(
//!         ProxyType::Http,
//!         IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
//!         8080,
//!         AnonymityLevel::Anonymous,
//!     );
//!     let anonymity = judge.judge_proxy(&mut proxy).await?;
//!     println!("Proxy anonymity level: {}", anonymity);
//!     Ok(())
//! }
//! ```

use crate::definitions::{
    enums::AnonymityLevel,
    errors::{JudgementError, JudgementResult},
    proxy::Proxy,
};
use crate::io::http::Requestor;

/// Service for judging proxies to determine their anonymity level
///
/// This service provides functionality to test proxies against judge services
/// and analyze their responses to determine the level of anonymity they provide.
/// The anonymity levels are categorized as Transparent, Anonymous, or Elite,
/// based on how much information about the client the proxy reveals.
///
/// # Examples
///
/// ```no_run
/// use gooty_proxy::inspection::Judge;
/// use gooty_proxy::definitions::proxy::Proxy;
/// use gooty_proxy::definitions::enums::{AnonymityLevel, ProxyType};
/// use std::net::{IpAddr, Ipv4Addr};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a judge service
///     let judge = Judge::new().await?;
///
///     // Create a proxy to test
///     let mut proxy = Proxy::new(
///         ProxyType::Http,
///         IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
///         8080,
///         AnonymityLevel::Anonymous, // Initial assumption
///     );
///
///     // Test the proxy and determine its actual anonymity level
///     let anonymity = judge.judge_proxy(&mut proxy).await?;
///     println!("Proxy anonymity level: {}", anonymity);
///
///     Ok(())
/// }
/// ```
pub struct Judge {
    /// URLs of proxy judge services
    judge_urls: Vec<String>,

    /// Requestor for making HTTP requests
    requestor: Requestor,
}

impl Judge {
    /// Create a new judge with default configuration
    ///
    /// Initializes a Judge service with the default set of proxy judge URLs
    /// and a requestor configured with the default validation timeout.
    ///
    /// # Returns
    ///
    /// A new Judge instance with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the Requestor cannot be created
    pub async fn new() -> JudgementResult<Self> {
        let judge_urls = crate::defaults::PROXY_JUDGE_URLS
            .iter()
            .map(|url| url.to_string())
            .collect();

        let requestor = Requestor::with_timeout(crate::defaults::DEFAULT_VALIDATION_TIMEOUT_SECS)?;

        Ok(Judge {
            judge_urls,
            requestor,
        })
    }

    /// Judge a proxy to determine its anonymity level
    ///
    /// Makes a request through the provided proxy to a judge service and
    /// analyzes the response to determine the proxy's anonymity level.
    /// The proxy is also updated with latency information.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy to judge, which will be modified to record check statistics
    ///
    /// # Returns
    ///
    /// The determined anonymity level of the proxy
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * No judge URL is available
    /// * The request through the proxy fails
    /// * The response analysis fails
    pub async fn judge_proxy(&self, proxy: &mut Proxy) -> JudgementResult<AnonymityLevel> {
        // Get a judge URL to use
        let judge_url = self
            .judge_urls
            .first()
            .ok_or(JudgementError::NoJudgeUrl)?
            .to_string();

        // Use a standard user agent for consistency
        let user_agent = "Mozilla/5.0 (compatible; Gooty-Proxy/0.1)";

        // Attempt to make a request through the proxy
        let start = std::time::Instant::now();
        let response = self
            .requestor
            .get_with_proxy(&judge_url, user_agent, proxy)
            .await?;

        // Record the latency
        let latency = start.elapsed().as_millis() as u32;
        proxy.record_check(latency);

        // Analyze the response to determine anonymity level
        let anonymity = self.determine_anonymity_level(&response, proxy)?;

        Ok(anonymity)
    }

    /// Determine the anonymity level from a judge response
    ///
    /// Analyzes the response from a proxy judge service to determine
    /// the anonymity level of the proxy.
    ///
    /// # Anonymity Levels
    ///
    /// * `Transparent` - The proxy reveals the client's IP address in headers
    /// * `Anonymous` - The proxy reveals it is a proxy but hides the client's IP
    /// * `Elite` - The proxy does not reveal any proxy information or client IP
    ///
    /// # Arguments
    ///
    /// * `response` - The response from the proxy judge service
    /// * `proxy` - The proxy that was used for the request
    ///
    /// # Returns
    ///
    /// The determined anonymity level
    ///
    /// # Errors
    ///
    /// Returns an error if the response cannot be analyzed
    fn determine_anonymity_level(
        &self,
        response: &str,
        proxy: &Proxy,
    ) -> JudgementResult<AnonymityLevel> {
        // Check if our proxy IP appears in the response
        let proxy_ip = proxy.address.to_string();

        // Headers to check for proxy information
        let headers_to_check = [
            "HTTP_VIA",
            "HTTP_X_FORWARDED_FOR",
            "HTTP_FORWARDED",
            "HTTP_X_REAL_IP",
            "VIA",
            "X_FORWARDED_FOR",
            "FORWARDED",
        ];

        // Check if any headers reveal proxy information
        let mut found_proxy_headers = false;
        let mut found_ip_in_headers = false;

        // Simple parsing - in a real implementation we'd use a proper parser
        for header in &headers_to_check {
            if response.contains(header) {
                found_proxy_headers = true;

                // Check if our IP is exposed in this header
                if response.contains(&proxy_ip) {
                    found_ip_in_headers = true;
                    break;
                }
            }
        }

        // Determine anonymity level
        if found_ip_in_headers {
            // IP is visible in headers - transparent proxy
            Ok(AnonymityLevel::Transparent)
        } else if found_proxy_headers {
            // Proxy headers exist but don't reveal our IP - anonymous proxy
            Ok(AnonymityLevel::Anonymous)
        } else {
            // No proxy information revealed - elite proxy
            Ok(AnonymityLevel::Elite)
        }
    }

    /// Add a judge URL
    ///
    /// Adds a new URL to the list of judge services, if it's not already present.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the judge service to add
    pub fn add_judge_url(&mut self, url: String) {
        if !self.judge_urls.contains(&url) {
            self.judge_urls.push(url);
        }
    }

    /// Get the current judge URLs
    ///
    /// # Returns
    ///
    /// A slice containing all the judge URLs currently configured
    pub fn get_judge_urls(&self) -> &[String] {
        &self.judge_urls
    }
}
