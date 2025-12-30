use crate::api::client::{ApiClient, ApiError};
use crate::models::dashboard::DashboardResponse;

impl ApiClient {
    /// Get user dashboard with pending sessions
    pub async fn get_dashboard(&self) -> Result<DashboardResponse, ApiError> {
        // Dashboard endpoint uses POST with empty body
        self.post("getDashboard", &serde_json::json!({})).await
    }
}
