use serde::{Deserialize, Serialize};

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
    pub duration: Option<i32>,
    pub session_name: Option<String>,
    pub slot1: Option<String>,
    pub slot2: Option<String>,
    pub slot3: Option<String>,
    pub sauna_no: Option<String>,
    pub time_slot: Option<String>,
    pub is_available: Option<bool>,
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

impl BookSessionResponse {
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.data.is_some()
    }
}
