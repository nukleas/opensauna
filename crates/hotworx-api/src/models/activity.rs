//! Lifetime activity history.
//!
//! `activities/ActivityByLifeTime` returns a paginated list of completed
//! sessions. The exact wire shape varies between endpoint versions, so the
//! response is exposed as a typed envelope around an array of records that
//! include the most useful fields and stash the rest in a flatten map.

use serde::{Deserialize, Serialize};

/// Paginated activity-history page from `activities/ActivityByLifeTime`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ActivityPage {
    /// Completed sessions for the requested page.
    #[serde(default)]
    pub activities: Vec<ActivityRecord>,
    /// Total number of pages available given the current page size.
    pub no_of_pages: Option<i64>,
    /// Sometimes populated with a 90-day window in addition to `activities`.
    #[serde(default)]
    pub ninety_days_activities: Vec<ActivityRecord>,
}

/// A single completed session in the activity log.
///
/// HOTWORX has shipped several versions of this shape; the well-known
/// fields are typed and everything else is captured in
/// [`ActivityRecord::extra`] so additions don't break existing callers.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ActivityRecord {
    pub session_record_id: Option<String>,
    pub lead_record_id: Option<String>,
    pub session_name: Option<String>,
    pub location_name: Option<String>,
    pub date: Option<String>,
    pub display_date: Option<String>,
    pub start_date_time: Option<String>,
    pub end_date_time: Option<String>,
    pub duration: Option<String>,
    pub cal_burnt: Option<String>,
    /// API uses `type` for the session-type label on some endpoints.
    #[serde(rename = "type")]
    pub session_type: Option<String>,
    /// Catch-all for fields the API may add over time.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}
