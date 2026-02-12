use serde::{Deserialize, Serialize};

/// Location data from booking API
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Location {
    pub is_allow: Option<String>,
    pub location_code: Option<String>,
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

/// Frequently used location
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrequentLocation {
    pub location_id: String,
    pub location_name: String,
    pub visit_count: Option<i32>,
}

/// Session type (e.g., Hot Yoga, Hot Pilates, etc.)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionType {
    pub by_session_type: String,
    pub session_name: Option<String>,
}

/// Response for getBookingLocations API
#[derive(Debug, Clone, Deserialize)]
pub struct LocationsResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<LocationsData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationsData {
    pub locations: Option<Vec<Location>>,
    pub frequent_locations: Option<Vec<FrequentLocation>>,
    pub session_types: Option<Vec<SessionType>>,
}

impl Location {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self.is_allow.as_deref(),
            Some("1") | Some("true") | Some("yes")
        )
    }
}
