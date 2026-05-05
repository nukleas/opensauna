//! Time slots, session-type lookups, and booking responses.

use serde::{Deserialize, Deserializer, Serialize};

/// A bookable workout time slot.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TimeSlot {
    #[serde(default, deserialize_with = "deserialize_optional_i32_from_str")]
    pub duration: Option<i32>,
    pub session_name: Option<String>,
    /// Per-station availability strings — when these contain the substring
    /// `"available"` the station is bookable. Helper:
    /// [`TimeSlot::is_slot_available`].
    pub slot1: Option<String>,
    pub slot2: Option<String>,
    pub slot3: Option<String>,
    /// Sauna number. The API mis-spells this as `suana_no` on some
    /// endpoints; both spellings deserialize correctly.
    #[serde(
        default,
        alias = "suana_no",
        deserialize_with = "deserialize_optional_string_from_number"
    )]
    pub sauna_no: Option<String>,
    pub time_slot: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool_from_str")]
    pub is_available: Option<bool>,
}

impl TimeSlot {
    /// Whether the given per-station slot string represents an available
    /// (bookable) station.
    pub fn is_slot_available(slot: &Option<String>) -> bool {
        slot.as_ref()
            .map(|v| v.contains("available"))
            .unwrap_or(false)
    }
}

/// A session type the user can book at a given location/date — e.g.
/// `"HOT YOGA"`, `"HOT PILATES"`. Returned by
/// [`get_session_types`](crate::HotworxClient::get_session_types).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionType {
    /// Display label (often the same as `value`).
    pub slot: Option<String>,
    /// API-canonical value to send back when booking or fetching slots.
    pub value: Option<String>,
}

/// Response from booking a session.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BookSessionResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub data: Option<BookedSessionData>,
}

impl BookSessionResponse {
    /// `true` when the booking was accepted (no error and an ID was returned).
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.data.is_some()
    }
}

/// IDs returned after a successful booking — needed to cancel the session
/// later via [`delete_session`](crate::HotworxClient::delete_session).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookedSessionData {
    pub session_record_id: Option<String>,
    pub lead_record_id: Option<String>,
}

fn deserialize_optional_i32_from_str<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i32),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(s)) => s.parse::<i32>().map(Some).map_err(de::Error::custom),
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

fn deserialize_optional_bool_from_str<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
        Int(i32),
    }

    match Option::<StringOrBool>::deserialize(deserializer)? {
        Some(StringOrBool::String(s)) => match s.to_lowercase().as_str() {
            "true" | "1" => Ok(Some(true)),
            _ => Ok(Some(false)),
        },
        Some(StringOrBool::Bool(b)) => Ok(Some(b)),
        Some(StringOrBool::Int(i)) => Ok(Some(i != 0)),
        None => Ok(None),
    }
}

fn deserialize_optional_string_from_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
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

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => Ok(Some(s)),
        Some(StringOrNumber::Int(i)) => Ok(Some(i.to_string())),
        Some(StringOrNumber::Float(f)) => Ok(Some(f.to_string())),
        None => Ok(None),
    }
}
