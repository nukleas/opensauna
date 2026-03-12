use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a value that may be either a number or a string containing a number
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
        Some(StringOrInt::String(s)) => s
            .parse::<i32>()
            .map(Some)
            .map_err(de::Error::custom),
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

/// Deserialize a value that may be either a bool or a string containing a bool
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

/// Deserialize an optional string that may come as a number from the API
fn deserialize_optional_string_from_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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

/// Request to show available time slots
#[derive(Debug, Clone, Serialize)]
pub struct ShowSlotsRequest {
    pub booking_date: String,
    pub location_id: String,
    pub view_type: String,
    pub time: String,
    pub session_type: String,
}

/// Time slot information
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TimeSlot {
    #[serde(default, deserialize_with = "deserialize_optional_i32_from_str")]
    pub duration: Option<i32>,
    pub session_name: Option<String>,
    pub slot1: Option<String>,
    pub slot2: Option<String>,
    pub slot3: Option<String>,
    #[serde(default, alias = "suana_no", deserialize_with = "deserialize_optional_string_from_number")]
    pub sauna_no: Option<String>,
    pub time_slot: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool_from_str")]
    pub is_available: Option<bool>,
}

impl TimeSlot {
    /// Check if a slot value indicates availability
    /// Handles both old format ("available") and new format ("/images/availables.jfif")
    pub fn is_slot_available(slot: &Option<String>) -> bool {
        slot.as_ref()
            .map(|v| v.contains("available"))
            .unwrap_or(false)
    }
}

/// Response for showSlots API
#[derive(Debug, Clone, Deserialize)]
pub struct SlotsResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<SlotsData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlotsData {
    pub slots: Option<Vec<TimeSlot>>,
    pub session_types: Option<Vec<String>>,
}

/// Request to book a session
#[derive(Debug, Clone, Serialize)]
pub struct BookSessionRequest {
    pub sauna_no: String,
    pub time_slot: String,
    pub booking_date: String,
    pub session_type: String,
    pub selected_location_id: String,
    pub message_popup: Option<String>,
}

/// Response for bookSession API
#[derive(Debug, Clone, Deserialize)]
pub struct BookSessionResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub data: Option<BookedSessionData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookedSessionData {
    pub session_record_id: Option<String>,
    pub lead_record_id: Option<String>,
}

/// Request to cancel/delete a session
#[derive(Debug, Clone, Serialize)]
pub struct DeleteSessionRequest {
    pub session_record_id: String,
    pub lead_record_id: String,
}

/// Level two booking details request
#[derive(Debug, Clone, Serialize)]
pub struct GetLevelTwoRequest {
    pub booking_date: String,
    pub location_id: String,
    pub view_type: String,
}

/// Session type available for booking (from getLevelTwo_v2 API)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionType {
    pub slot: Option<String>,
    pub value: Option<String>,
}

impl BookSessionResponse {
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.data.is_some()
    }
}
