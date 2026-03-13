use crate::api::client::{ApiClient, ApiError};
use crate::models::profile::{
    CalorieStatsResponse, UpdateGoalsRequest, UpdateProfileRequest, ViewGoalsResponse,
    ViewProfileResponse,
};

impl ApiClient {
    /// Get user profile
    pub async fn view_profile(&self) -> Result<ViewProfileResponse, ApiError> {
        self.post_form("general/view_profile", &serde_json::json!({}))
            .await
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        request: &UpdateProfileRequest,
    ) -> Result<serde_json::Value, ApiError> {
        self.post_form("general/update_profile", request).await
    }

    /// Get calorie stats (lifetime)
    pub async fn view_calorie_stats(&self) -> Result<CalorieStatsResponse, ApiError> {
        self.post_form("general/view_calorie_stats", &serde_json::json!({}))
            .await
    }

    /// Get goals
    pub async fn view_goals(&self) -> Result<ViewGoalsResponse, ApiError> {
        self.post_form("general/viewGoals", &serde_json::json!({}))
            .await
    }

    /// Update goals
    pub async fn update_goals(
        &self,
        request: &UpdateGoalsRequest,
    ) -> Result<serde_json::Value, ApiError> {
        self.post_form("general/updateGoals", request).await
    }
}
