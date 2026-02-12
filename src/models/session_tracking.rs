use super::dashboard::PendingSession;
use serde::{Deserialize, Serialize};

/// Session tracking state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Upcoming,
    Active,
    Completed,
    Cancelled,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Upcoming
    }
}

/// Active session tracking data (persisted locally)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedSession {
    /// Original session record ID from the booking API
    pub session_record_id: String,
    /// Lead record ID (user identifier)
    pub lead_record_id: String,
    /// Session name (e.g., "Hot Yoga", "Hot Pilates")
    pub session_name: String,
    /// Location name
    pub location_name: String,
    /// Planned duration in minutes
    pub duration_minutes: u32,
    /// Timestamp when user started the session (checked in) - Unix ms
    pub started_at: Option<i64>,
    /// Timestamp when session was completed/ended - Unix ms
    pub ended_at: Option<i64>,
    /// Original scheduled start time (from booking)
    pub scheduled_start: String,
    /// Original scheduled end time (from booking)
    pub scheduled_end: String,
    /// Current state
    pub state: SessionState,
    /// Whether this session has been synced to the API
    pub synced: bool,
}

impl TrackedSession {
    /// Create from a PendingSession
    pub fn from_pending(session: &PendingSession) -> Option<Self> {
        Some(Self {
            session_record_id: session.session_record_id.clone()?,
            lead_record_id: session.lead_record_id.clone()?,
            session_name: session.session_name.clone().unwrap_or_default(),
            location_name: session.location_name.clone().unwrap_or_default(),
            duration_minutes: session
                .duration
                .as_ref()
                .and_then(|d| d.parse().ok())
                .unwrap_or(30),
            started_at: None,
            ended_at: None,
            scheduled_start: session.start_date_time.clone().unwrap_or_default(),
            scheduled_end: session.end_date_time.clone().unwrap_or_default(),
            state: SessionState::Upcoming,
            synced: false,
        })
    }

    /// Calculate elapsed time in seconds since session started
    pub fn elapsed_seconds(&self, now_ms: i64) -> Option<i64> {
        self.started_at
            .map(|start| ((now_ms - start) / 1000).max(0))
    }

    /// Calculate remaining time in seconds (based on planned duration)
    pub fn remaining_seconds(&self, now_ms: i64) -> Option<i64> {
        self.elapsed_seconds(now_ms).map(|elapsed| {
            let total = (self.duration_minutes as i64) * 60;
            (total - elapsed).max(0)
        })
    }

    /// Calculate progress as a percentage (0.0 to 1.0+)
    pub fn progress(&self, now_ms: i64) -> Option<f64> {
        self.elapsed_seconds(now_ms).map(|elapsed| {
            let total = (self.duration_minutes as i64) * 60;
            if total > 0 {
                elapsed as f64 / total as f64
            } else {
                0.0
            }
        })
    }

    /// Check if session is overtime (elapsed > planned duration)
    pub fn is_overtime(&self, now_ms: i64) -> bool {
        self.elapsed_seconds(now_ms)
            .map(|elapsed| elapsed > (self.duration_minutes as i64) * 60)
            .unwrap_or(false)
    }

    /// Total duration in seconds
    pub fn total_seconds(&self) -> i64 {
        (self.duration_minutes as i64) * 60
    }
}
