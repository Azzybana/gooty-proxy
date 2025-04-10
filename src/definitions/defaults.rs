//! # Default Configuration Values
//!
//! This module provides default configuration values and constants for the proxy system.
//! These defaults serve as sensible starting points for various configuration options
//! when explicit values are not provided by the user.
//!
//! ## Categories
//!
//! * **Request Parameters** - Timeouts, retries, and delays for HTTP requests
//! * **User Agents** - A collection of browser User-Agent strings for request headers
//! * **Proxy Configuration** - Default settings for proxy validation and management
//! * **Persistence** - Settings for data storage and caching
//! * **Regular Expressions** - Patterns for extracting proxy information from text
//!
//! ## Examples
//!
//! ```
//! use spiderling_proxy::definitions::defaults;
//! use std::time::Duration;
//!
//! // Create a timeout duration from the default value
//! let timeout = Duration::from_secs(defaults::DEFAULT_REQUEST_TIMEOUT_SECS);
//!
//! // Get a random User-Agent from the default collection
//! let user_agent_list = defaults::DEFAULT_USER_AGENTS;
//! ```

/// A collection of proxy judge URLs that can be used to test proxies
///
/// These URLs provide services that return information about the request,
/// which is useful for determining the level of anonymity a proxy provides.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
///
/// // Get the first judge URL for testing
/// let judge_url = defaults::PROXY_JUDGE_URLS[0];
/// assert!(!judge_url.is_empty());
/// ```
pub const PROXY_JUDGE_URLS: &[&str] = &["http://proxyjudge.us/azenv.php", "http://azenv.net"];

/// Default User-Agent strings that can be rotated when making requests
///
/// These User-Agent strings are organized by browser type and platform.
/// Using different User-Agents helps avoid detection and blocks when
/// making many requests.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
/// use rand::seq::SliceRandom;
///
/// // Select a random User-Agent
/// let mut rng = rand::thread_rng();
/// if let Some(user_agent) = defaults::DEFAULT_USER_AGENTS.choose(&mut rng) {
///     println!("Using User-Agent: {}", user_agent);
/// }
/// ```
pub const DEFAULT_USER_AGENTS: &[&str] = &[
    // Chrome (3)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36",
    // Edge (3)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edge/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Edge/123.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edge/122.0.2365.80 Safari/537.36 Edg/122.0.2365.80",
    // Firefox (3)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/124.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:124.0) Gecko/20100101 Firefox/124.0",
    "Mozilla/5.0 (X11; Linux i686; rv:124.0) Gecko/20100101 Firefox/124.0",
    // Opera (2)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 OPR/109.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 OPR/109.0.0.0",
    // Lynx (2)
    "Lynx/2.9.0dev.11 libwww-FM/2.14 SSL-MM/1.4.1 GNUTLS/3.6.13",
    "Lynx/2.8.9rel.1 libwww-FM/2.14 SSL-MM/1.4.1 OpenSSL/1.1.1k",
    // Links (2)
    "Links (2.28; Linux x86_64; GNU C 9.3.0; text)",
    "Links (2.27; Windows; text)",
    // Safari (1)
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    // Brave (1)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 Brave/1.62.153",
    // Vivaldi (1)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36 Vivaldi/6.5.3206.63",
    // Pale Moon (1)
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.0) Gecko/20100101 Goanna/6.5 Firefox/102.0 PaleMoon/33.1.0",
];

/// Default timeout in seconds for HTTP requests
///
/// This value determines how long to wait for a response before
/// considering the request failed due to timeout.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
/// use std::time::Duration;
///
/// let timeout = Duration::from_secs(defaults::DEFAULT_REQUEST_TIMEOUT_SECS);
/// ```
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Default number of retry attempts for failed requests
///
/// When a request fails due to network issues or timeouts, this determines
/// how many additional attempts should be made before giving up.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
///
/// let retries_remaining = defaults::DEFAULT_REQUEST_RETRIES;
/// ```
pub const DEFAULT_REQUEST_RETRIES: u32 = 3;

/// Default delay between sequential requests to avoid rate limiting (in milliseconds)
///
/// This delay helps prevent triggering rate limiting when making multiple
/// requests to the same host.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
/// use std::time::Duration;
///
/// let delay = Duration::from_millis(defaults::DEFAULT_REQUEST_DELAY_MS);
/// ```
pub const DEFAULT_REQUEST_DELAY_MS: u64 = 500;

/// Default number of proxies to validate in parallel
///
/// This controls how many parallel validation operations can run simultaneously
/// when testing multiple proxies.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
///
/// let parallelism = defaults::DEFAULT_PARALLEL_VALIDATIONS;
/// ```
pub const DEFAULT_PARALLEL_VALIDATIONS: usize = 10;

/// Default threshold for considering a proxy "alive" (in milliseconds)
///
/// If a proxy's latency exceeds this value, it might be considered too slow for use.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
///
/// let max_acceptable_latency = defaults::DEFAULT_MAX_ACCEPTABLE_LATENCY_MS;
/// let is_proxy_fast_enough = measured_latency <= max_acceptable_latency;
/// ```
pub const DEFAULT_MAX_ACCEPTABLE_LATENCY_MS: u32 = 3000;

/// Default validation timeout (shorter than general request timeout)
///
/// Specific timeout value used during proxy validation operations, typically
/// shorter than the general request timeout.
///
/// # Examples
///
/// ```
/// use spiderling_proxy::definitions::defaults;
/// use std::time::Duration;
///
/// let validation_timeout = Duration::from_secs(defaults::DEFAULT_VALIDATION_TIMEOUT_SECS);
/// ```
pub const DEFAULT_VALIDATION_TIMEOUT_SECS: u64 = 10;

/// Default proxy rotation settings
///
/// Contains constants related to when and how proxies should be rotated
/// during operation.
pub mod rotation {
    /// Minimum success rate threshold for including a proxy in rotation
    ///
    /// Proxies with a success rate below this threshold may be excluded from rotation.
    pub const MIN_SUCCESS_RATE: f64 = 0.7;

    /// Maximum consecutive failures before removing proxy from rotation
    ///
    /// After this many consecutive failures, a proxy will be temporarily removed
    /// from the rotation pool.
    pub const MAX_CONSECUTIVE_FAILURES: u32 = 3;

    /// Time until a failed proxy is eligible for retesting (in seconds)
    ///
    /// Determines how long to wait before attempting to use a failed proxy again.
    pub const FAILURE_COOLDOWN_SECS: u64 = 300; // 5 minutes
}

/// Regex patterns for extracting proxies from text sources
///
/// This module provides regular expression patterns that can be used to extract
/// proxy information from various text formats.
pub mod regex_patterns {
    /// Basic IP:PORT pattern
    ///
    /// Matches simple IP:PORT format like "127.0.0.1:8080"
    pub const IP_PORT: &str = r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):(\d{2,5})";

    /// Pattern with proxy type (http|https|socks4|socks5)://ip:port
    ///
    /// Matches protocol-specified proxies like "http://127.0.0.1:8080"
    pub const TYPED_PROXY: &str =
        r"(https?|socks[45])://(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):(\d{2,5})";

    /// Pattern with optional authentication user:pass@ip:port
    ///
    /// Matches authenticated proxies like "user:pass@127.0.0.1:8080"
    pub const AUTH_PROXY: &str =
        r"(?:([^:@]+):([^@]+)@)?(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):(\d{2,5})";
}

/// Default persistence settings
///
/// Contains constants related to data storage, backup, and retention.
pub mod persistence {
    /// How often to automatically save proxy data (in seconds)
    ///
    /// Determines the interval for auto-saving proxy data to persistent storage.
    pub const AUTO_SAVE_INTERVAL_SECS: u64 = 300; // 5 minutes

    /// Maximum age for a proxy before requiring revalidation (in seconds)
    ///
    /// Proxies older than this value will need to be retested before use.
    pub const MAX_PROXY_AGE_SECS: u64 = 86400; // 24 hours
}

/// Default ports for different proxy types
///
/// Standard port numbers commonly used for different proxy protocols.
pub mod default_ports {
    /// Default port for HTTP proxies
    pub const HTTP: u16 = 8080;

    /// Default port for HTTPS proxies
    pub const HTTPS: u16 = 8443;

    /// Default port for SOCKS4 proxies
    pub const SOCKS4: u16 = 1080;

    /// Default port for SOCKS5 proxies
    pub const SOCKS5: u16 = 1080;
}
