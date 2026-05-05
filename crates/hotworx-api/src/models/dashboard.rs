//! Dashboard summary and the per-day pending/completed session lists.

use serde::{Deserialize, Deserializer, Serialize};

/// Response payload for `getDashboard`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DashboardData {
    /// Sessions booked for today that haven't been used yet.
    pub todays_pending_sessions: Option<Vec<PendingSession>>,
    /// Sessions completed today.
    pub todays_completed_sessions: Option<Vec<PendingSession>>,
    /// Lifetime + current-sprint summary stats.
    pub summary: Option<Summary>,
}

/// A single booked or completed session.
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
    /// Some endpoints use `type` as the session-type name instead of
    /// `session_name`; both are exposed.
    #[serde(rename = "type")]
    pub session_type: Option<String>,
}

/// Aggregate stats shown on the dashboard.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Summary {
    #[serde(default)]
    pub total_sessions: Option<String>,
    #[serde(default)]
    pub total_cal_burned: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i32_from_str")]
    pub day_for_current_sprint: Option<i32>,
    /// Note: API uses the misspelled key `continious_streak`; we preserve it.
    #[serde(default)]
    pub continious_streak: Option<String>,
    #[serde(default)]
    pub reward_level: Option<String>,
}

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
