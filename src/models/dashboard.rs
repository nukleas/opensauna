use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_optional_i32_from_str<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i32),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(s)) => {
            s.parse::<i32>().map(Some).map_err(serde::de::Error::custom)
        }
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

/// Dashboard API response
#[derive(Debug, Clone, Deserialize)]
pub struct DashboardResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<DashboardData>,
}

/// Dashboard data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashboardData {
    pub todays_pending_sessions: Option<Vec<PendingSession>>,
    pub todays_completed_sessions: Option<Vec<PendingSession>>,
    pub summary: Option<Summary>,
}

/// Pending or completed session
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PendingSession {
    pub date: Option<String>,
    pub duration: Option<String>,
    pub lead_record_id: Option<String>,
    pub location_name: Option<String>,
    pub sauna: Option<String>,
    pub session_name: Option<String>,
    pub session_record_id: Option<String>,
    pub slot: Option<String>,
    pub display_date: Option<String>,
    pub start_date_time: Option<String>,
    pub end_date_time: Option<String>,
    pub cal_burnt: Option<String>,
    pub week_day: Option<String>,
    #[serde(rename = "type")]
    pub session_type: Option<String>,
}

/// Summary statistics (fields match API response)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Summary {
    #[serde(default)]
    pub total_sessions: Option<String>,
    #[serde(default)]
    pub total_cal_burned: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i32_from_str")]
    pub day_for_current_sprint: Option<i32>,
    #[serde(default)]
    pub continious_streak: Option<String>,
    #[serde(default)]
    pub reward_level: Option<String>,
}

impl Summary {
    /// Total sessions as a display string (comma-grouped), defaults to "0".
    pub fn sessions_count(&self) -> String {
        crate::utils::format::with_commas(self.total_sessions.as_deref().unwrap_or("0"))
    }

    /// Total calories burned as a display string (comma-grouped), defaults to "0".
    pub fn calories_burned(&self) -> String {
        crate::utils::format::with_commas(self.total_cal_burned.as_deref().unwrap_or("0"))
    }

    /// Current consecutive-day streak as a display string, defaults to "0".
    pub fn streak(&self) -> String {
        self.continious_streak
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }
}

impl PendingSession {
    /// Get a display-friendly time string
    pub fn display_time(&self) -> String {
        self.slot.clone().unwrap_or_default()
    }

    /// Get the session display name — falls back to `type` field for completed sessions
    pub fn display_name(&self) -> String {
        self.session_name
            .clone()
            .or_else(|| self.session_type.clone())
            .unwrap_or_else(|| "Session".to_string())
    }

    /// Get location display name
    pub fn display_location(&self) -> String {
        self.location_name.clone().unwrap_or_default()
    }

    /// Get calories burned (completed sessions)
    pub fn display_calories(&self) -> Option<String> {
        self.cal_burnt
            .as_ref()
            .filter(|c| !c.is_empty() && *c != "0")
            .map(|c| format!("{} cal", crate::utils::format::with_commas(c)))
    }
}
