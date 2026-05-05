//! User profile, calorie stats, weight history, and goal-tracking models.

use serde::{Deserialize, Serialize};

/// The detailed profile returned by `general/view_profile`.
///
/// The HOTWORX API returns a number of fields with mixed types (numbers
/// posing as strings, booleans posing as numbers, missing keys). The
/// deserializer below coerces everything to `Option<String>` so callers
/// don't have to special-case the wire format.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProfileData {
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub user_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub first_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub last_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub full_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub email_address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub phone_number: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub dob: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub gender: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub height: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub weight: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub location_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub country: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub dob_display: Option<String>,
    /// Catch-all for fields HOTWORX adds without rolling out a wire-format
    /// version bump. Unrecognized keys land here so existing callers keep
    /// working.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Per-day calorie breakdown returned by `general/get_summary`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DailySummary {
    pub after_burn: Option<String>,
    pub hiit_calories: Option<String>,
    pub isometric_calories: Option<String>,
}

/// Response from `general/get_summary_thirty_days`. The data field contains
/// an array of single-item wrappers around [`ThirtyDaySummary`] — that
/// double indirection is what the API produces; we preserve it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThirtyDaySummaryWrapper {
    pub data: Option<ThirtyDaySummary>,
}

/// Rolling 30-day stats.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThirtyDaySummary {
    pub total_sessions: Option<String>,
    pub total_calorie_burned: Option<String>,
    pub workout_calorie_burned: Option<String>,
    pub afterburn_calorie_burned: Option<String>,
    pub last_weight_reading: Option<String>,
    pub last_body_fat_reading: Option<String>,
}

/// Rolling 90-day calorie progress and level data.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NinetyDaySummary {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub current_calories: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub current_level: Option<i32>,
    pub total_days: Option<f64>,
}

/// Lifetime calorie statistics.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CalorieStats {
    pub total_session: Option<String>,
    pub total_calories_burned: Option<String>,
    pub workout_calories_burned: Option<String>,
    /// Despite the name, this is the lifetime afterburn (EPOC) total.
    pub total_one_hour_burned: Option<String>,
    pub avg_calorie_burned: Option<String>,
    pub last_workout_date: Option<String>,
}

/// User-set fitness goals.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GoalsData {
    /// Numeric or string current weight, depending on what was last saved.
    pub current_weight: Option<serde_json::Value>,
    /// Numeric or string target weight.
    pub target_weight: Option<serde_json::Value>,
    pub target_weight_goal_date: Option<String>,
    pub weekly_session_goal: Option<String>,
}

/// A single weight reading from `general/get_weight`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightEntry {
    pub weight_in_pound: Option<String>,
}

fn deserialize_flexible_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(v.and_then(|val| match val {
        serde_json::Value::String(s) if !s.is_empty() => Some(s),
        serde_json::Value::Null | serde_json::Value::String(_) => None,
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }))
}
