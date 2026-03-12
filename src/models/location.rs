use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a string that may come as either a string or a number from the API
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

/// Location data from booking API
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Location {
    pub is_allow: Option<String>,
    pub location_code: Option<String>,
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
    /// API uses "frequently_locations" (typo) — accept both spellings
    #[serde(alias = "frequently_locations")]
    pub frequent_locations: Option<Vec<Location>>,
    /// API now returns a single object instead of an array
    #[serde(default, deserialize_with = "deserialize_session_types")]
    pub session_types: Option<Vec<SessionType>>,
    /// API also uses singular "session_type"
    #[serde(default)]
    pub session_type: Option<SessionType>,
}

/// Session type (e.g., Hot Yoga, Hot Pilates, etc.)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionType {
    pub by_session_type: String,
    pub session_name: Option<String>,
}

/// Handle session_types being either an array or a single object
fn deserialize_session_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<SessionType>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        Many(Vec<SessionType>),
        One(SessionType),
    }

    match Option::<OneOrMany>::deserialize(deserializer)? {
        Some(OneOrMany::Many(v)) => Ok(Some(v)),
        Some(OneOrMany::One(s)) => Ok(Some(vec![s])),
        None => Ok(None),
    }
}

impl Location {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self.is_allow.as_deref(),
            Some("1") | Some("true") | Some("yes")
        )
    }
}
