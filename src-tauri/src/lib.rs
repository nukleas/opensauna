use sha2::{Sha256, Digest};
use tauri_plugin_store::StoreExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BASE_URL: &str = "https://sailposapi.hotworx.net/api/v1";

/// Response from login/OTP verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub msg: Option<String>,
    pub token: Option<String>,
    pub two_factor: Option<String>,
    pub error: Option<String>,
    pub status: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Make a form-urlencoded POST request to the Hotworx API
async fn api_post_form(
    endpoint: &str,
    params: HashMap<String, String>,
    auth_token: Option<&str>,
    device_id: &str,
) -> Result<String, String> {
    let url = format!("{}/{}", BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "okhttp/4.9.3")
        .header("sec-ch-ua-platform", "Android")
        .header("application-version", "5.0.0")
        .header("device-id", device_id);

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    // Build form body
    let body: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    println!("[API] POST {} with body: {}", url, body);

    let response = request
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

    println!("[API] Response status: {}, body: {}", status, &text[..text.len().min(500)]);

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }

    Ok(text)
}

/// Make a GET request to the Hotworx API
async fn api_get(
    endpoint: &str,
    auth_token: Option<&str>,
    device_id: &str,
) -> Result<String, String> {
    let url = format!("{}/{}", BASE_URL, endpoint);

    let client = reqwest::Client::new();
    let mut request = client
        .get(&url)
        .header("User-Agent", "okhttp/4.9.3")
        .header("sec-ch-ua-platform", "Android")
        .header("application-version", "5.0.0")
        .header("device-id", device_id);

    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    println!("[API] GET {}", url);

    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

    println!("[API] Response status: {}, body: {}", status, &text[..text.len().min(500)]);

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }

    Ok(text)
}

/// Login with email and password (returns token, may require OTP)
#[tauri::command]
async fn api_login_with_password(
    app: tauri::AppHandle,
    email: String,
    password: String,
) -> Result<LoginResponse, String> {
    // Hash password
    let password_hash = {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    };

    // Get device ID
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("email_address".to_string(), email);
    params.insert("password".to_string(), password_hash);
    params.insert("device_id".to_string(), device_id.clone());

    let response_text = api_post_form("loginwithpassword", params, None, &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Verify OTP after password login
#[tauri::command]
async fn api_verify_otp(
    app: tauri::AppHandle,
    email: String,
    password: String,
    otp: String,
    token: String,
) -> Result<LoginResponse, String> {
    // Hash password
    let password_hash = {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hex::encode(hasher.finalize())
    };

    // Get device ID
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("email_address".to_string(), email);
    params.insert("password".to_string(), password_hash);
    params.insert("phone_number".to_string(), String::new());
    params.insert("device_id".to_string(), device_id.clone());
    params.insert("otp".to_string(), otp);
    params.insert("type".to_string(), "password".to_string());

    let response_text = api_post_form("verifyOtp", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Get dashboard data (requires auth token)
#[tauri::command]
async fn api_get_dashboard(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    // Get stored auth token
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;

    // Get device ID
    let device_id = get_device_id(app).await?;

    let params = HashMap::new(); // Empty body for dashboard
    let response_text = api_post_form("getDashboard", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Get booking locations (requires auth token)
#[tauri::command]
async fn api_get_locations(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let response_text = api_get("booking/getBookingLocations_v2", Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Get available session types for a location and date
#[tauri::command(rename_all = "camelCase")]
async fn api_get_session_types(
    app: tauri::AppHandle,
    location_id: String,
    selected_date: String,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("selected_location_id".to_string(), location_id);
    params.insert("selected_date".to_string(), selected_date);
    params.insert("view_type".to_string(), "by_session_type".to_string());

    let response_text = api_post_form("booking/getLevelTwo_v2", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Get available time slots for booking
#[tauri::command(rename_all = "camelCase")]
async fn api_show_slots(
    app: tauri::AppHandle,
    booking_date: String,
    location_id: String,
    session_type: String,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    // Original app uses "selected_date" not "booking_date"
    params.insert("selected_date".to_string(), booking_date);
    params.insert("selected_location_id".to_string(), location_id);
    params.insert("view_type".to_string(), "by_session_type".to_string());
    params.insert("selected_time".to_string(), "all".to_string());
    params.insert("session_type".to_string(), session_type);

    let response_text = api_post_form("booking/showSlots", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Book a session
#[tauri::command(rename_all = "camelCase")]
async fn api_book_session(
    app: tauri::AppHandle,
    sauna_no: String,
    time_slot: String,
    booking_date: String,
    session_type: String,
    location_id: String,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("sauna_no".to_string(), sauna_no);
    params.insert("time_slot".to_string(), time_slot);
    params.insert("booking_date".to_string(), booking_date);
    params.insert("session_type".to_string(), session_type);
    params.insert("selected_location_id".to_string(), location_id);

    let response_text = api_post_form("booking/bookSession_v2", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Cancel/delete a session
#[tauri::command(rename_all = "camelCase")]
async fn api_delete_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    // API expects "booking_id" but we pass session_record_id value (original app does the same)
    params.insert("booking_id".to_string(), session_record_id);
    params.insert("lead_record_id".to_string(), lead_record_id);

    let response_text = api_post_form("booking/deleteSession", params, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Hash a password using SHA-256 (matching the original Hotworx app)
#[tauri::command]
fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Get the device ID (or generate one if not exists)
#[tauri::command]
async fn get_device_id(app: tauri::AppHandle) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    if let Some(device_id) = store.get("device_id") {
        if let Some(id) = device_id.as_str() {
            return Ok(id.to_string());
        }
    }

    // Generate a new device ID
    let device_id = uuid::Uuid::new_v4().to_string();
    store.set("device_id", serde_json::json!(device_id.clone()));
    store.save().map_err(|e| e.to_string())?;

    Ok(device_id)
}

/// Store the auth token securely
#[tauri::command]
async fn store_auth_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("token", serde_json::json!(token));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the stored auth token
#[tauri::command]
async fn get_auth_token(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;

    if let Some(token) = store.get("token") {
        if let Some(t) = token.as_str() {
            return Ok(Some(t.to_string()));
        }
    }

    Ok(None)
}

/// Clear the auth token (logout)
#[tauri::command]
async fn clear_auth_token(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.delete("token");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Store pending login data for OTP flow (survives page navigation)
#[tauri::command]
async fn store_pending_login(
    app: tauri::AppHandle,
    email: String,
    password: String,
    token: String,
) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("pending_email", serde_json::json!(email));
    store.set("pending_password", serde_json::json!(password));
    store.set("pending_token", serde_json::json!(token));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get pending login data for OTP verification
#[tauri::command]
async fn get_pending_login(app: tauri::AppHandle) -> Result<Option<(String, String, String)>, String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;

    let email = store.get("pending_email").and_then(|v| v.as_str().map(String::from));
    let password = store.get("pending_password").and_then(|v| v.as_str().map(String::from));
    let token = store.get("pending_token").and_then(|v| v.as_str().map(String::from));

    match (email, password, token) {
        (Some(e), Some(p), Some(t)) => Ok(Some((e, p, t))),
        _ => Ok(None),
    }
}

/// Clear pending login data
#[tauri::command]
async fn clear_pending_login(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.delete("pending_email");
    store.delete("pending_password");
    store.delete("pending_token");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Store the user's preferred location
#[tauri::command(rename_all = "camelCase")]
async fn store_preferred_location(
    app: tauri::AppHandle,
    location_id: String,
    location_name: String,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("preferred_location_id", serde_json::json!(location_id));
    store.set("preferred_location_name", serde_json::json!(location_name));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the user's preferred location
#[tauri::command]
async fn get_preferred_location(app: tauri::AppHandle) -> Result<Option<(String, String)>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let location_id = store.get("preferred_location_id").and_then(|v| v.as_str().map(String::from));
    let location_name = store.get("preferred_location_name").and_then(|v| v.as_str().map(String::from));

    match (location_id, location_name) {
        (Some(id), Some(name)) => Ok(Some((id, name))),
        _ => Ok(None),
    }
}

/// Clear the user's preferred location
#[tauri::command]
async fn clear_preferred_location(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.delete("preferred_location_id");
    store.delete("preferred_location_name");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Store the user's preferred session type (e.g., "HOT BLAST")
#[tauri::command(rename_all = "camelCase")]
async fn store_preferred_session_type(
    app: tauri::AppHandle,
    session_type: String,
    session_type_display: String,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("preferred_session_type", serde_json::json!(session_type));
    store.set("preferred_session_type_display", serde_json::json!(session_type_display));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the user's preferred session type
#[tauri::command]
async fn get_preferred_session_type(app: tauri::AppHandle) -> Result<Option<(String, String)>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let session_type = store.get("preferred_session_type").and_then(|v| v.as_str().map(String::from));
    let display = store.get("preferred_session_type_display").and_then(|v| v.as_str().map(String::from));

    match (session_type, display) {
        (Some(t), Some(d)) => Ok(Some((t, d))),
        _ => Ok(None),
    }
}

/// Get activity history (completed sessions) from the server
/// Uses the activities/ActivityByLifeTime endpoint with pagination
#[tauri::command(rename_all = "camelCase")]
async fn api_get_activity_history(
    app: tauri::AppHandle,
    page_no: Option<u32>,
    page_limit: Option<u32>,
    session_type: Option<String>,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    // Build query string
    let page = page_no.unwrap_or(1);
    let limit = page_limit.unwrap_or(50);
    let session_filter = session_type.unwrap_or_else(|| "all".to_string());

    let endpoint = format!(
        "activities/ActivityByLifeTime?page_no={}&page_limit={}&session_type={}",
        page, limit, urlencoding::encode(&session_filter)
    );

    let response_text = api_get(&endpoint, Some(&token), &device_id).await?;

    serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &response_text[..response_text.len().min(200)]))
}

/// Get all upcoming booked sessions (not just today)
#[tauri::command]
async fn api_get_upcoming_sessions(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    // Try the booking history endpoint with "upcoming" filter
    let mut params = HashMap::new();
    params.insert("type".to_string(), "upcoming".to_string());

    match api_post_form("booking/getBookingHistory", params, Some(&token), &device_id).await {
        Ok(response_text) => {
            serde_json::from_str(&response_text)
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
        Err(e) => {
            // Fallback: if endpoint doesn't exist, return empty structure
            println!("[API] getBookingHistory failed, falling back: {}", e);
            Ok(serde_json::json!({
                "status": "fallback",
                "data": {
                    "upcoming": [],
                    "message": "Upcoming sessions endpoint not available"
                }
            }))
        }
    }
}

// ========== SESSION TRACKING COMMANDS ==========

/// Store the active session state
#[tauri::command]
async fn store_active_session(
    app: tauri::AppHandle,
    session: serde_json::Value,
) -> Result<(), String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    store.set("active_session", session);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get the active session state
#[tauri::command]
async fn get_active_session(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;

    if let Some(session) = store.get("active_session") {
        Ok(Some(session.clone()))
    } else {
        Ok(None)
    }
}

/// Clear the active session state
#[tauri::command]
async fn clear_active_session(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    store.delete("active_session");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Store session history (completed sessions)
#[tauri::command]
async fn store_session_history(
    app: tauri::AppHandle,
    session: serde_json::Value,
) -> Result<(), String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;

    // Get existing history or create empty array
    let mut history: Vec<serde_json::Value> = store
        .get("session_history")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Add new session to history
    history.push(session);

    // Keep only last 100 sessions
    if history.len() > 100 {
        let skip_count = history.len() - 100;
        history = history.into_iter().skip(skip_count).collect();
    }

    store.set("session_history", serde_json::json!(history));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get session history
#[tauri::command]
async fn get_session_history(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;

    let history: Vec<serde_json::Value> = store
        .get("session_history")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(history)
}

/// API endpoint for syncing session check-in (graceful fallback)
#[tauri::command(rename_all = "camelCase")]
async fn api_checkin_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("session_record_id".to_string(), session_record_id);
    params.insert("lead_record_id".to_string(), lead_record_id);

    // Try to sync with API - this endpoint may or may not exist
    match api_post_form("booking/checkinSession", params, Some(&token), &device_id).await {
        Ok(response_text) => {
            serde_json::from_str(&response_text)
                .map_err(|e| format!("Parse error: {}", e))
        }
        Err(e) => {
            // API endpoint doesn't exist or failed - return success anyway
            // Session tracking continues locally
            println!("[API] Check-in sync failed (expected): {}", e);
            Ok(serde_json::json!({
                "status": "local_only",
                "msg": "Session tracked locally, API sync not available"
            }))
        }
    }
}

/// API endpoint for syncing session completion (graceful fallback)
#[tauri::command(rename_all = "camelCase")]
async fn api_complete_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
    actual_duration_seconds: i64,
) -> Result<serde_json::Value, String> {
    let token = get_auth_token(app.clone()).await?
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(app).await?;

    let mut params = HashMap::new();
    params.insert("session_record_id".to_string(), session_record_id);
    params.insert("lead_record_id".to_string(), lead_record_id);
    params.insert("actual_duration".to_string(), actual_duration_seconds.to_string());

    // Try to sync with API - this endpoint may or may not exist
    match api_post_form("booking/completeSession", params, Some(&token), &device_id).await {
        Ok(response_text) => {
            serde_json::from_str(&response_text)
                .map_err(|e| format!("Parse error: {}", e))
        }
        Err(e) => {
            println!("[API] Complete session sync failed (expected): {}", e);
            Ok(serde_json::json!({
                "status": "local_only",
                "msg": "Session completion tracked locally, API sync not available"
            }))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            hash_password,
            get_device_id,
            store_auth_token,
            get_auth_token,
            clear_auth_token,
            store_pending_login,
            get_pending_login,
            clear_pending_login,
            store_preferred_location,
            get_preferred_location,
            clear_preferred_location,
            store_preferred_session_type,
            get_preferred_session_type,
            api_get_upcoming_sessions,
            api_get_activity_history,
            api_login_with_password,
            api_verify_otp,
            api_get_dashboard,
            api_get_locations,
            api_get_session_types,
            api_show_slots,
            api_book_session,
            api_delete_session,
            // Session tracking commands
            store_active_session,
            get_active_session,
            clear_active_session,
            store_session_history,
            get_session_history,
            api_checkin_session,
            api_complete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "testPassword123";
        let hash = hash_password(password);

        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same password should produce same hash
        let hash2 = hash_password(password);
        assert_eq!(hash, hash2);

        // Different password should produce different hash
        let different_hash = hash_password("differentPassword");
        assert_ne!(hash, different_hash);
    }

    #[test]
    fn test_hash_password_known_value() {
        // SHA-256 of "password" - verified value
        let hash = hash_password("password");
        // Just verify it's a valid hex string of correct length
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        // The actual SHA-256 hash of "password"
        assert_eq!(
            hash,
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
        );
    }

    #[test]
    fn test_hash_password_empty() {
        let hash = hash_password("");
        // SHA-256 of empty string is known
        assert_eq!(hash.len(), 64);
        // e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_login_response_deserialization() {
        let json = r#"{"msg":"success","token":"abc123","two_factor":null,"error":null,"status":"ok"}"#;
        let response: LoginResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.msg, Some("success".to_string()));
        assert_eq!(response.token, Some("abc123".to_string()));
        assert!(response.two_factor.is_none());
        assert!(response.error.is_none());
        assert_eq!(response.status, Some("ok".to_string()));
    }

    #[test]
    fn test_login_response_with_two_factor() {
        let json = r#"{"msg":null,"token":"temp123","two_factor":"required","error":null,"status":"pending"}"#;
        let response: LoginResponse = serde_json::from_str(json).unwrap();

        assert!(response.msg.is_none());
        assert_eq!(response.token, Some("temp123".to_string()));
        assert_eq!(response.two_factor, Some("required".to_string()));
        assert_eq!(response.status, Some("pending".to_string()));
    }

    #[test]
    fn test_login_response_with_error() {
        let json = r#"{"msg":null,"token":null,"two_factor":null,"error":"Invalid credentials","status":"error"}"#;
        let response: LoginResponse = serde_json::from_str(json).unwrap();

        assert!(response.token.is_none());
        assert_eq!(response.error, Some("Invalid credentials".to_string()));
        assert_eq!(response.status, Some("error".to_string()));
    }
}
