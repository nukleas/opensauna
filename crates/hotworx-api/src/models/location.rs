//! HOTWORX studio locations.

use serde::{Deserialize, Deserializer, Serialize};

/// A bookable HOTWORX studio.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Location {
    /// `"1"`, `"true"`, or `"yes"` when the member is allowed to book here
    /// without paying a reciprocal fee. See [`Location::is_allowed`].
    pub is_allow: Option<String>,
    pub location_code: Option<String>,
    /// Stable studio identifier. Comes back as either a string or a number
    /// depending on the endpoint; we always expose it as a string.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub location_id: String,
    pub location_name: String,
    pub location_tier: Option<String>,
    pub tier_badge: Option<String>,
    pub location_tier_fee: Option<String>,
    pub reciprocal_fees: Option<String>,
    pub currency_symbol: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub location_type: Option<String>,
}

impl Location {
    /// Whether the member is allowed to book at this location without a
    /// reciprocal fee. Accepts the various truthy values the API uses.
    pub fn is_allowed(&self) -> bool {
        matches!(
            self.is_allow.as_deref(),
            Some("1") | Some("true") | Some("yes")
        )
    }
}

/// Response for `booking/getBookingLocations_v2`. Note that `frequent_locations`
/// is also accepted under the API's mis-spelled `frequently_locations` key.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LocationsData {
    pub locations: Option<Vec<Location>>,
    #[serde(alias = "frequently_locations")]
    pub frequent_locations: Option<Vec<Location>>,
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Int(i64),
        Float(f64),
    }

    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => Ok(s),
        StringOrNumber::Int(i) => Ok(i.to_string()),
        StringOrNumber::Float(f) => Ok(f.to_string()),
    }
}
