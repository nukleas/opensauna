use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::models::session_tracking::{TrackedSession, SessionState};
use crate::models::dashboard::PendingSession;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_name = "Date.now")]
    fn date_now() -> f64;
}

/// Get current time in milliseconds
fn now_ms() -> i64 {
    date_now() as i64
}

/// Global session tracking state
#[derive(Clone, Copy)]
pub struct SessionTrackingState {
    /// The currently active session (if any)
    pub active_session: RwSignal<Option<TrackedSession>>,
    /// Loading state for session operations
    pub loading: RwSignal<bool>,
}

impl SessionTrackingState {
    /// Create a new SessionTrackingState
    pub fn new() -> Self {
        Self {
            active_session: RwSignal::new(None),
            loading: RwSignal::new(false),
        }
    }

    /// Check if there's an active session
    pub fn has_active_session(&self) -> Signal<bool> {
        let active = self.active_session;
        Signal::derive(move || active.get().is_some())
    }

    /// Start a new session from a PendingSession
    pub async fn start_session(&self, pending: &PendingSession) -> Result<(), String> {
        // Create tracked session from pending
        let mut tracked = TrackedSession::from_pending(pending)
            .ok_or_else(|| "Invalid session data".to_string())?;

        // Set the start time
        tracked.started_at = Some(now_ms());
        tracked.state = SessionState::Active;

        log(&format!("[SessionState] Starting session: {:?}", tracked.session_record_id));

        // Store in Tauri store
        let session_json = serde_json::to_value(&tracked)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let args = serde_wasm_bindgen::to_value(&serde_json::json!({
            "session": session_json
        })).map_err(|e| format!("Args error: {}", e))?;

        JsFuture::from(invoke("store_active_session", args))
            .await
            .map_err(|e| format!("Store error: {:?}", e))?;

        // Try to sync with API (graceful fallback)
        let checkin_args = serde_wasm_bindgen::to_value(&serde_json::json!({
            "sessionRecordId": tracked.session_record_id,
            "leadRecordId": tracked.lead_record_id
        })).map_err(|e| format!("Args error: {}", e))?;

        // Fire and forget - we don't care if API sync fails
        let _ = JsFuture::from(invoke("api_checkin_session", checkin_args)).await;

        // Update local state
        self.active_session.set(Some(tracked));

        Ok(())
    }

    /// End the current session
    pub async fn end_session(&self) -> Result<(), String> {
        let session = self.active_session.get()
            .ok_or_else(|| "No active session".to_string())?;

        log(&format!("[SessionState] Ending session: {:?}", session.session_record_id));

        let elapsed = session.elapsed_seconds(now_ms()).unwrap_or(0);

        // Try to sync with API
        let args = serde_wasm_bindgen::to_value(&serde_json::json!({
            "sessionRecordId": session.session_record_id,
            "leadRecordId": session.lead_record_id,
            "actualDurationSeconds": elapsed
        })).map_err(|e| format!("Args error: {}", e))?;

        let _ = JsFuture::from(invoke("api_complete_session", args)).await;

        // Store in history
        let mut completed = session.clone();
        completed.ended_at = Some(now_ms());
        completed.state = SessionState::Completed;

        let history_json = serde_json::to_value(&completed)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let history_args = serde_wasm_bindgen::to_value(&serde_json::json!({
            "session": history_json
        })).map_err(|e| format!("Args error: {}", e))?;

        let _ = JsFuture::from(invoke("store_session_history", history_args)).await;

        // Clear active session
        let clear_args = serde_wasm_bindgen::to_value(&serde_json::json!({}))
            .map_err(|e| format!("Args error: {}", e))?;

        JsFuture::from(invoke("clear_active_session", clear_args))
            .await
            .map_err(|e| format!("Clear error: {:?}", e))?;

        // Update local state
        self.active_session.set(None);

        Ok(())
    }

    /// Restore active session from storage (call on app start)
    pub async fn restore_session(&self) {
        self.loading.set(true);

        let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();

        match JsFuture::from(invoke("get_active_session", args)).await {
            Ok(result) => {
                if !result.is_null() && !result.is_undefined() {
                    match serde_wasm_bindgen::from_value::<TrackedSession>(result) {
                        Ok(session) => {
                            log(&format!("[SessionState] Restored session: {:?}", session.session_record_id));
                            self.active_session.set(Some(session));
                        }
                        Err(e) => {
                            log(&format!("[SessionState] Parse error: {:?}", e));
                        }
                    }
                }
            }
            Err(e) => {
                log(&format!("[SessionState] Restore error: {:?}", e));
            }
        }

        self.loading.set(false);
    }
}

impl Default for SessionTrackingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide SessionTrackingState context at the app root
pub fn provide_session_tracking_state() -> SessionTrackingState {
    let state = SessionTrackingState::new();
    provide_context(state);
    state
}

/// Use SessionTrackingState from context
pub fn use_session_tracking_state() -> SessionTrackingState {
    use_context::<SessionTrackingState>().expect("SessionTrackingState must be provided")
}
