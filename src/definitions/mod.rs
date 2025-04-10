//! # Definitions Module
//!
//! This module contains core definitions used throughout the gooty proxy system.
//! It includes types, constants, and error definitions that are shared across
//! various components of the application.
//!
//! ## Components
//!
//! * **Source** - Represents a proxy source configuration
//! * **Proxy** - Defines the structure and behavior of proxies
//! * **Errors** - Custom error types for the system
//! * **Defaults** - Default values for configuration and runtime behavior
//! * **Enums** - Enumerations used across the system

//! # Core Definitions
//!
//! Fundamental data structures and type definitions for the Gooty Proxy system.
//!
//! ## Overview
//!
//! This module contains the core types, enums, and structures that define the data model
//! for the Gooty Proxy system, including:
//!
//! * **Proxy** - Core representation of proxy servers with their connection details and metadata
//! * **Source** - Proxy source definitions and interfaces
//! * **Enums** - Type definitions for proxy protocols, anonymity levels, and other categorizations
//! * **Errors** - Error types specific to different components of the system
//! * **Defaults** - Default configuration values and constants
//!
//! These definitions are used throughout the library to provide a consistent data model
//! and type-safe interfaces.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::definitions::{
//!     Proxy,
//!     enums::{ProxyType, AnonymityLevel},
//! };
//! use std::net::{IpAddr, Ipv4Addr};
//!
//! // Create a new proxy definition
//! let proxy = Proxy::new(
//!     ProxyType::Http,
//!     IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
//!     8080,
//!     AnonymityLevel::Elite
//! );
//!
//! assert_eq!(proxy.proxy_type, ProxyType::Http);
//! assert_eq!(proxy.port, 8080);
//! ```

pub mod defaults;
pub mod enums;
pub mod errors;
pub mod proxy;
pub mod source;

// Re-exports for backward compatibility
pub use defaults::{
    DEFAULT_MAX_ACCEPTABLE_LATENCY_MS, DEFAULT_PARALLEL_VALIDATIONS, DEFAULT_REQUEST_DELAY_MS,
    DEFAULT_REQUEST_RETRIES, DEFAULT_REQUEST_TIMEOUT_SECS, DEFAULT_USER_AGENTS,
    DEFAULT_VALIDATION_TIMEOUT_SECS, PROXY_JUDGE_URLS,
};

pub use enums::{
    AnonymityLevel, LogLevel, ProxyType, RotationStrategy, SourceStatus, ValidationState,
    VerificationMethod,
};

pub use errors::{
    CidrError, CidrResult, FilestoreError, FilestoreResult, JudgementError, JudgementResult,
    ManagerError, ManagerResult, OwnershipError, OwnershipResult, ProxyError, RequestResult,
    RequestorError, SleuthError, SleuthResult, SourceError, SourceResult, UtilError, UtilResult,
};

pub use proxy::Proxy;
pub use source::Source;
