//! # Location Module
//!
//! This module provides types and functionality for handling geographical location
//! information related to IP addresses and proxy servers.
//!
//! ## Overview
//!
//! The module is centered around the `Location` struct, which encapsulates
//! geographic data including:
//!
//! - Country and country code
//! - Region or state
//! - City
//! - Postal/zip code
//! - Geographic coordinates (latitude and longitude)
//! - Time zone information
//!
//! This module is primarily used for enriching proxy and IP metadata with
//! location information, enabling geographical filtering and organization.
//!
//! ## Examples
//!
//! ```
//! use gooty_proxy::inspection::Location;
//!
//! // Create a location for a proxy server
//! let location = Location::new()
//!     .with_country("United States".to_string())
//!     .with_country_code("US".to_string())
//!     .with_city("New York".to_string())
//!     .with_coordinates(40.7128, -74.0060);
//!
//! assert_eq!(location.country, Some("United States".to_string()));
//! assert_eq!(location.city, Some("New York".to_string()));
//! ```

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Represents the geographical location of an IP address.
///
/// This struct contains detailed location data that can be gathered
/// from IP geolocation services.
///
/// # Examples
///
/// ```
/// use gooty_proxy::inspection::Location;
///
/// let location = Location {
///     country: Some("United States".to_string()),
///     city: Some("Seattle".to_string()),
///     state: Some("Washington".to_string()),
///     postal_code: Some("98101".to_string()),
///     facility_name: None,
/// };
///
/// assert_eq!(location.country.as_deref(), Some("United States"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Location {
    /// City name
    pub city: Option<String>,

    /// State, province, or region name
    pub state: Option<String>,

    /// Postal or ZIP code
    pub postal_code: Option<String>,

    /// Country name
    pub country: Option<String>,

    /// Specific facility name (e.g., data center name)
    pub facility_name: Option<String>,
}

impl Location {
    /// Creates a new Location with the given country
    ///
    /// # Arguments
    ///
    /// * `country` - The country name
    ///
    /// # Returns
    ///
    /// A new Location with only the country field set
    #[must_use]
    pub fn with_country(country: String) -> Self {
        Location {
            country: Some(country),
            city: None,
            state: None,
            postal_code: None,
            facility_name: None,
        }
    }

    /// Creates a new detailed Location
    ///
    /// # Arguments
    ///
    /// * `country` - The country name
    /// * `state` - The state/region name
    /// * `city` - The city name
    /// * `postal_code` - The postal/ZIP code
    ///
    /// # Returns
    ///
    /// A new Location with the specified fields set
    #[must_use]
    pub fn new(
        country: Option<String>,
        state: Option<String>,
        city: Option<String>,
        postal_code: Option<String>,
    ) -> Self {
        Location {
            country,
            state,
            city,
            postal_code,
            facility_name: None,
        }
    }

    /// Adds facility name information to this location
    ///
    /// # Arguments
    ///
    /// * `facility_name` - The name of the facility or data center
    ///
    /// # Returns
    ///
    /// Self with the `facility_name` field updated
    #[must_use]
    pub fn with_facility(mut self, facility_name: String) -> Self {
        self.facility_name = Some(facility_name);
        self
    }

    /// Checks if this location has any information
    ///
    /// # Returns
    ///
    /// True if at least one field is populated, false otherwise
    #[must_use]
    pub fn has_info(&self) -> bool {
        self.country.is_some()
            || self.state.is_some()
            || self.city.is_some()
            || self.postal_code.is_some()
            || self.facility_name.is_some()
    }

    /// Gets a formatted string representation of this location
    ///
    /// # Returns
    ///
    /// A formatted string with available location information
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(city) = &self.city {
            parts.push(city.clone());
        }

        if let Some(state) = &self.state {
            parts.push(state.clone());
        }

        if let Some(country) = &self.country {
            parts.push(country.clone());
        }

        if parts.is_empty() {
            "Unknown location".to_string()
        } else {
            parts.join(", ")
        }
    }
}

// Implement Display trait instead of using inherent to_string method
impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let Some(city) = &self.city {
            parts.push(city.clone());
        }

        if let Some(state) = &self.state {
            parts.push(state.clone());
        }

        if let Some(country) = &self.country {
            parts.push(country.clone());
        }

        write!(f, "{}", parts.join(", "))
    }
}
