use serde::{Deserialize, Serialize};

// ===== Profile Models =====

/// Response from general/view_profile (POST)
#[derive(Debug, Clone, Deserialize)]
pub struct ViewProfileResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<ProfileData>,
}

/// Deserialize any JSON value as Option<String>, coercing non-strings to their JSON representation
fn deserialize_flexible_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(v.and_then(|val| match val {
        serde_json::Value::String(s) => {
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
        serde_json::Value::Null => None,
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None, // Skip objects/arrays
    }))
}

/// Profile data from the API
/// Uses flexible deserialization since the API returns mixed types for some fields
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
    // Catch-all for any extra fields the API returns
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl ProfileData {
    /// Full name, falling back to first+last, then "User".
    pub fn display_name(&self) -> String {
        self.full_name
            .clone()
            .or_else(
                || match (self.first_name.as_ref(), self.last_name.as_ref()) {
                    (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
                    (Some(f), None) => Some(f.clone()),
                    _ => None,
                },
            )
            .unwrap_or_else(|| "User".to_string())
    }

    /// Email address, checking both `email` and `email_address` fields.
    pub fn display_email(&self) -> String {
        self.email
            .clone()
            .or_else(|| self.email_address.clone())
            .unwrap_or_default()
    }

    /// Phone number, checking both `phone` and `phone_number` fields.
    pub fn display_phone(&self) -> String {
        self.phone
            .clone()
            .or_else(|| self.phone_number.clone())
            .unwrap_or_default()
    }

    /// Height string, defaults to "--".
    pub fn display_height(&self) -> String {
        self.height.clone().unwrap_or_else(|| "--".to_string())
    }

    /// Weight string, defaults to "--".
    pub fn display_weight(&self) -> String {
        self.weight.clone().unwrap_or_else(|| "--".to_string())
    }

    /// Gender normalized to "Male"/"Female", defaults to "--".
    pub fn display_gender(&self) -> String {
        match self.gender.as_deref() {
            Some("M") | Some("male") => "Male".to_string(),
            Some("F") | Some("female") => "Female".to_string(),
            Some(g) => g.to_string(),
            None => "--".to_string(),
        }
    }

    /// Date of birth string, defaults to "--".
    pub fn display_dob(&self) -> String {
        self.dob.clone().unwrap_or_else(|| "--".to_string())
    }
}

/// Request body for updating profile (POST form-encoded)
#[derive(Debug, Clone, Serialize)]
pub struct UpdateProfileRequest {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub dob: String,
    pub gender: String,
    pub height: String,
    pub weight: String,
    pub address: String,
}

/// Generic API response for update operations
#[derive(Debug, Clone, Deserialize)]
pub struct ApiStatusResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub error: Option<String>,
}

// ===== Summary / Stats Models =====

/// Response from general/get_summary (POST with date param)
#[derive(Debug, Clone, Deserialize)]
pub struct SummaryResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<SummaryData>,
}

/// Summary data - daily breakdown
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SummaryData {
    pub after_burn: Option<String>,
    pub hiit_calories: Option<String>,
    pub isometric_calories: Option<String>,
}

/// Response from general/get_summary_thirty_days (POST)
#[derive(Debug, Clone, Deserialize)]
pub struct ThirtyDaySummaryResponse {
    pub msg: Option<String>,
    pub data: Option<Vec<ThirtyDaySummaryWrapper>>,
}

/// Wrapper for the nested `data` field in 30-day summary responses.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThirtyDaySummaryWrapper {
    pub data: Option<ThirtyDaySummaryData>,
}

/// 30-day summary stats: sessions, calories, weight, and body fat.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ThirtyDaySummaryData {
    pub total_sessions: Option<String>,
    pub total_calorie_burned: Option<String>,
    pub workout_calorie_burned: Option<String>,
    pub afterburn_calorie_burned: Option<String>,
    pub last_weight_reading: Option<String>,
    pub last_body_fat_reading: Option<String>,
}

/// Response from general/get_ninety_days_summary (POST)
#[derive(Debug, Clone, Deserialize)]
pub struct NinetyDaySummaryResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<NinetyDaySummaryData>,
}

/// 90-day summary stats: date range, calorie progress, and level.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NinetyDaySummaryData {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub current_calories: Option<f64>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub current_level: Option<i32>,
    pub total_days: Option<f64>,
}

/// Response from general/view_calorie_stats (GET)
#[derive(Debug, Clone, Deserialize)]
pub struct CalorieStatsResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<CalorieStatsData>,
}

/// Lifetime calorie statistics breakdown.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CalorieStatsData {
    pub total_session: Option<String>,
    pub total_calories_burned: Option<String>,
    pub workout_calories_burned: Option<String>,
    pub total_one_hour_burned: Option<String>,
    pub avg_calorie_burned: Option<String>,
    pub last_workout_date: Option<String>,
}

impl CalorieStatsData {
    /// Total session count, defaults to "0".
    pub fn total_sessions_display(&self) -> String {
        self.total_session
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }

    /// Total calories burned (lifetime), defaults to "0".
    pub fn total_calories_display(&self) -> String {
        self.total_calories_burned
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }

    /// Workout-only calories (excluding afterburn), defaults to "0".
    pub fn workout_calories_display(&self) -> String {
        self.workout_calories_burned
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }

    /// Afterburn (EPOC) calories, defaults to "0".
    pub fn afterburn_display(&self) -> String {
        self.total_one_hour_burned
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }

    /// Average calories per session, defaults to "0".
    pub fn avg_calories_display(&self) -> String {
        self.avg_calorie_burned
            .clone()
            .unwrap_or_else(|| "0".to_string())
    }
}

// ===== Goals Models =====

/// Response from general/viewGoals (GET)
#[derive(Debug, Clone, Deserialize)]
pub struct ViewGoalsResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub data: Option<GoalsData>,
}

/// User fitness goals (weight target, session frequency).
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GoalsData {
    pub current_weight: Option<serde_json::Value>,
    pub target_weight: Option<serde_json::Value>,
    pub target_weight_goal_date: Option<String>,
    pub weekly_session_goal: Option<String>,
}

impl GoalsData {
    /// Current weight as string, handles both number and string JSON values.
    pub fn current_weight_display(&self) -> String {
        match &self.current_weight {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "--".to_string(),
        }
    }

    /// Target weight as string, handles both number and string JSON values.
    pub fn target_weight_display(&self) -> String {
        match &self.target_weight {
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => "--".to_string(),
        }
    }

    /// Target date for reaching goal weight, defaults to "--".
    pub fn goal_date_display(&self) -> String {
        self.target_weight_goal_date
            .clone()
            .unwrap_or_else(|| "--".to_string())
    }

    /// Weekly session goal count, defaults to "--".
    pub fn weekly_sessions_display(&self) -> String {
        self.weekly_session_goal
            .clone()
            .unwrap_or_else(|| "--".to_string())
    }
}

/// Request body for updating goals (POST form-encoded)
#[derive(Debug, Clone, Serialize)]
pub struct UpdateGoalsRequest {
    pub current_weight: String,
    pub target_weight: String,
    pub target_weight_goal_date: String,
    pub weekly_session_goal: String,
}

// ===== Weight Models =====

/// Response from general/get_weight (POST)
#[derive(Debug, Clone, Deserialize)]
pub struct WeightResponse {
    pub msg: Option<String>,
    pub data: Option<Vec<WeightEntry>>,
}

/// A single weight log entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightEntry {
    pub weight_in_pound: Option<String>,
}

/// Request for setting weight (POST form-encoded)
#[derive(Debug, Clone, Serialize)]
pub struct SetWeightRequest {
    pub weight_in_pound: String,
}

/// Request for setting weight + height (POST form-encoded)
#[derive(Debug, Clone, Serialize)]
pub struct SetWeightHeightRequest {
    pub weight_in_pound: String,
    pub height_in_ft: String,
    pub dob: String,
}
