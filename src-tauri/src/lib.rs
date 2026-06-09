//! BookWorx Tauri backend.
//!
//! All HOTWORX HTTP traffic flows through [`hotworx_api::HotworxClient`].
//! This file is responsible for the *desktop-app concerns* the crate
//! intentionally doesn't cover:
//!
//! - AES-256-GCM token-at-rest encryption keyed off a per-install device ID.
//! - Persistence of preferences (preferred location, session type) and
//!   in-progress session-tracking state.
//! - The Tauri IPC surface — every `api_*` command here is a thin wrapper
//!   that loads credentials from disk, asks the crate to do the work, and
//!   returns the result in the JSON shape the WASM frontend expects.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use hotworx_api::{
    password_hash, ActivityPage, BookSessionResponse, CalorieStats, DailySummary, DashboardData,
    GoalsData, HotworxClient, HotworxError, Location, LoginResponse, NinetyDaySummary, ProfileData,
    SessionType, ThirtyDaySummary, TimeSlot, WeightEntry,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri_plugin_store::StoreExt;

/// Sentinel prefix the frontend matches on when a token *was* present but the
/// server rejected it (HTTP 401/403) — i.e. the session expired. The same
/// string lives in `src/state/auth_state.rs`.
const AUTH_EXPIRED_PREFIX: &str = "AUTH_EXPIRED";

/// Sentinel prefix for the distinct case where no token is stored at all —
/// the user was never logged in (or the token was cleared), as opposed to an
/// expired one. The frontend differentiates the two to message accordingly.
/// The same string lives in `src/state/auth_state.rs`.
const NOT_AUTHENTICATED_PREFIX: &str = "NOT_AUTHENTICATED";

// ─── Token-at-rest encryption ────────────────────────────────────────────

/// Derive a 32-byte AES-256 key from the device ID. The key never leaves
/// the device and changes if the device ID is reset (e.g. after an
/// `auth.json` wipe).
fn derive_key(device_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(b"bookworx-token-encryption-salt");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn encrypt_value(value: &str, device_id: &str) -> Result<String, String> {
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher error: {}", e))?;

    let nonce_bytes: [u8; 12] = {
        use aes_gcm::aead::rand_core::RngCore;
        let mut bytes = [0u8; 12];
        OsRng.fill_bytes(&mut bytes);
        bytes
    };
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|e| format!("Encrypt error: {}", e))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&combined))
}

fn decrypt_value(encrypted: &str, device_id: &str) -> Result<String, String> {
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher error: {}", e))?;

    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode error: {}", e))?;

    if combined.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed - token may be corrupted".to_string())?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 error: {}", e))
}

// ─── Frontend-facing error mapping ──────────────────────────────────────

/// Map a [`HotworxError`] into the string the frontend expects. Auth
/// failures get the `AUTH_EXPIRED:` prefix so `handle_invoke_error` in
/// `src/state/auth_state.rs` can flip the user back to the login screen.
fn ipc_error(err: HotworxError) -> String {
    match err {
        HotworxError::AuthExpired => format!("{}: token expired", AUTH_EXPIRED_PREFIX),
        other => other.to_string(),
    }
}

/// Build a [`HotworxClient`] from the persisted token + device-id. If no token
/// is stored, returns the `NOT_AUTHENTICATED` sentinel (never logged in) —
/// distinct from the `AUTH_EXPIRED` sentinel a 401 from the server produces.
/// Both send the user to login, but the frontend messages them differently.
async fn build_client(app: &tauri::AppHandle) -> Result<HotworxClient, String> {
    let device_id = get_device_id(app.clone()).await?;
    let token = get_auth_token(app.clone())
        .await?
        .ok_or_else(|| format!("{}: not authenticated", NOT_AUTHENTICATED_PREFIX))?;
    Ok(HotworxClient::new(device_id).with_token(token))
}

/// Build an unauthenticated client — used only by the login flow.
async fn build_anon_client(app: &tauri::AppHandle) -> Result<HotworxClient, String> {
    let device_id = get_device_id(app.clone()).await?;
    Ok(HotworxClient::new(device_id))
}

// ─── JSON envelopes the frontend currently consumes ──────────────────────

/// Most `general/*` endpoints come back as `{ "data": {...} }`. The
/// frontend pages descend into `data` rather than reading the field
/// directly, so we preserve that shape until Phase 4 retires the manual
/// JSON traversal.
#[derive(Serialize)]
struct DataEnvelope<T> {
    data: T,
}

#[derive(Serialize)]
struct LocationsEnvelope {
    data: LocationsInner,
}
#[derive(Serialize)]
struct LocationsInner {
    locations: Vec<Location>,
}

#[derive(Serialize)]
struct SessionTypesResponse {
    list: Vec<SessionType>,
}

#[derive(Serialize)]
struct UpcomingEnvelope {
    data: UpcomingInner,
}
#[derive(Serialize)]
struct UpcomingInner {
    upcoming: Vec<hotworx_api::PendingSession>,
}

#[derive(Serialize)]
struct ProfileEnvelope {
    data: Vec<ProfileWrapper>,
}
#[derive(Serialize)]
struct ProfileWrapper {
    data: ProfileData,
}

#[derive(Serialize)]
struct WeightEnvelope {
    data: Vec<WeightEntry>,
}

#[derive(Serialize)]
struct ThirtyDayEnvelope {
    data: Vec<ThirtyDayInner>,
}
#[derive(Serialize)]
struct ThirtyDayInner {
    data: ThirtyDaySummary,
}

// ─── Auth ────────────────────────────────────────────────────────────────

/// Step 1 of password login. Hashing happens inside the crate.
#[tauri::command(rename_all = "camelCase")]
async fn api_login_with_password(
    app: tauri::AppHandle,
    email: String,
    password: String,
) -> Result<LoginResponse, String> {
    let client = build_anon_client(&app).await?;
    client
        .login_with_password(&email, &password)
        .await
        .map_err(ipc_error)
}

/// Step 2 of password+OTP login. `password` is the plaintext password the
/// frontend stashed from step 1; we hash it here (SHA-256) before sending,
/// exactly as `api_login_with_password` does. The HOTWORX `verifyOtp`
/// endpoint expects the password field to carry the SHA-256 digest, not the
/// plaintext — sending plaintext makes OTP verification fail.
#[tauri::command(rename_all = "camelCase")]
async fn api_verify_otp(
    app: tauri::AppHandle,
    email: String,
    password: String,
    otp: String,
    token: String,
) -> Result<LoginResponse, String> {
    let device_id = get_device_id(app).await?;
    let client = HotworxClient::new(device_id).with_token(token);
    client
        .verify_otp(&email, &password_hash(&password), &otp)
        .await
        .map_err(ipc_error)
}

// ─── Dashboard / sessions ────────────────────────────────────────────────

#[tauri::command(rename_all = "camelCase")]
async fn api_get_dashboard(
    app: tauri::AppHandle,
    current_date: Option<String>,
) -> Result<DataEnvelope<DashboardData>, String> {
    let client = build_client(&app).await?;
    let data = client
        .get_dashboard(current_date.as_deref())
        .await
        .map_err(ipc_error)?;
    Ok(DataEnvelope { data })
}

/// Fetch upcoming bookings across today + the next two days. The HOTWORX
/// API doesn't expose a multi-day list directly, so we stitch together
/// three `getDashboard` calls.
#[tauri::command]
async fn api_get_upcoming_sessions(app: tauri::AppHandle) -> Result<UpcomingEnvelope, String> {
    let client = build_client(&app).await?;

    let mut all_upcoming = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for day_offset in 0..3 {
        let date = get_date_with_offset(day_offset);
        match client.get_dashboard(Some(&date)).await {
            Ok(dashboard) => {
                if let Some(sessions) = dashboard.todays_pending_sessions {
                    for session in sessions {
                        let id = session.session_record_id.clone().unwrap_or_default();
                        if !id.is_empty() && seen_ids.insert(id) {
                            all_upcoming.push(session);
                        }
                    }
                }
            }
            Err(HotworxError::AuthExpired) => {
                return Err(ipc_error(HotworxError::AuthExpired));
            }
            Err(e) => {
                eprintln!("[API] getDashboard for {} failed: {}", date, e);
            }
        }
    }

    Ok(UpcomingEnvelope {
        data: UpcomingInner {
            upcoming: all_upcoming,
        },
    })
}

// ─── Booking ─────────────────────────────────────────────────────────────

#[tauri::command]
async fn api_get_locations(app: tauri::AppHandle) -> Result<LocationsEnvelope, String> {
    let client = build_client(&app).await?;
    let locations = client.get_locations().await.map_err(ipc_error)?;
    Ok(LocationsEnvelope {
        data: LocationsInner { locations },
    })
}

#[tauri::command(rename_all = "camelCase")]
async fn api_get_session_types(
    app: tauri::AppHandle,
    location_id: String,
    selected_date: String,
) -> Result<SessionTypesResponse, String> {
    let client = build_client(&app).await?;
    let list = client
        .get_session_types(&location_id, &selected_date)
        .await
        .map_err(ipc_error)?;
    Ok(SessionTypesResponse { list })
}

#[tauri::command(rename_all = "camelCase")]
async fn api_show_slots(
    app: tauri::AppHandle,
    booking_date: String,
    location_id: String,
    session_type: String,
) -> Result<Vec<TimeSlot>, String> {
    let client = build_client(&app).await?;
    client
        .show_slots(&location_id, &booking_date, &session_type)
        .await
        .map_err(ipc_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn api_book_session(
    app: tauri::AppHandle,
    sauna_no: String,
    time_slot: String,
    booking_date: String,
    session_type: String,
    location_id: String,
) -> Result<BookSessionResponse, String> {
    let client = build_client(&app).await?;
    client
        .book_session(
            &location_id,
            &booking_date,
            &session_type,
            &sauna_no,
            &time_slot,
        )
        .await
        .map_err(ipc_error)
}

#[tauri::command(rename_all = "camelCase")]
async fn api_delete_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
) -> Result<serde_json::Value, String> {
    let client = build_client(&app).await?;
    client
        .delete_session(&session_record_id, &lead_record_id)
        .await
        .map_err(ipc_error)?;
    Ok(serde_json::json!({ "status": "ok" }))
}

#[tauri::command(rename_all = "camelCase")]
async fn api_get_activity_history(
    app: tauri::AppHandle,
    page_no: Option<u32>,
    page_limit: Option<u32>,
    session_type: Option<String>,
) -> Result<ActivityPage, String> {
    let client = build_client(&app).await?;
    client
        .get_activity_history(
            page_no.unwrap_or(1),
            page_limit.unwrap_or(50),
            session_type.as_deref(),
        )
        .await
        .map_err(ipc_error)
}

// ─── Profile / goals / weight / stats ────────────────────────────────────

#[tauri::command]
async fn api_view_profile(app: tauri::AppHandle) -> Result<ProfileEnvelope, String> {
    let client = build_client(&app).await?;
    let data = client.view_profile().await.map_err(ipc_error)?;
    Ok(ProfileEnvelope {
        data: vec![ProfileWrapper { data }],
    })
}

#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "camelCase")]
async fn api_update_profile(
    app: tauri::AppHandle,
    first_name: String,
    last_name: String,
    dob: String,
    gender: String,
    height: String,
    weight: String,
    address: String,
) -> Result<serde_json::Value, String> {
    let client = build_client(&app).await?;
    client
        .update_profile(
            &first_name,
            &last_name,
            &dob,
            &gender,
            &height,
            &weight,
            &address,
        )
        .await
        .map_err(ipc_error)?;
    Ok(serde_json::json!({ "status": "ok" }))
}

#[tauri::command(rename_all = "camelCase")]
async fn api_get_summary(
    app: tauri::AppHandle,
    date: String,
) -> Result<DataEnvelope<DailySummary>, String> {
    let client = build_client(&app).await?;
    let data = client.get_summary(&date).await.map_err(ipc_error)?;
    Ok(DataEnvelope { data })
}

#[tauri::command]
async fn api_get_thirty_day_summary(app: tauri::AppHandle) -> Result<ThirtyDayEnvelope, String> {
    let client = build_client(&app).await?;
    let data = client.get_thirty_day_summary().await.map_err(ipc_error)?;
    Ok(ThirtyDayEnvelope {
        data: vec![ThirtyDayInner { data }],
    })
}

#[tauri::command]
async fn api_get_ninety_day_summary(
    app: tauri::AppHandle,
) -> Result<DataEnvelope<NinetyDaySummary>, String> {
    let client = build_client(&app).await?;
    let data = client.get_ninety_day_summary().await.map_err(ipc_error)?;
    Ok(DataEnvelope { data })
}

#[tauri::command]
async fn api_get_calorie_stats(
    app: tauri::AppHandle,
) -> Result<DataEnvelope<CalorieStats>, String> {
    let client = build_client(&app).await?;
    let data = client.get_calorie_stats().await.map_err(ipc_error)?;
    Ok(DataEnvelope { data })
}

#[tauri::command]
async fn api_view_goals(app: tauri::AppHandle) -> Result<DataEnvelope<GoalsData>, String> {
    let client = build_client(&app).await?;
    let data = client.view_goals().await.map_err(ipc_error)?;
    Ok(DataEnvelope { data })
}

#[tauri::command(rename_all = "camelCase")]
async fn api_update_goals(
    app: tauri::AppHandle,
    current_weight: String,
    target_weight: String,
    target_weight_goal_date: String,
    weekly_session_goal: String,
) -> Result<serde_json::Value, String> {
    let client = build_client(&app).await?;
    client
        .update_goals(
            &current_weight,
            &target_weight,
            &target_weight_goal_date,
            &weekly_session_goal,
        )
        .await
        .map_err(ipc_error)?;
    Ok(serde_json::json!({ "status": "ok" }))
}

#[tauri::command]
async fn api_get_weight(app: tauri::AppHandle) -> Result<WeightEnvelope, String> {
    let client = build_client(&app).await?;
    let data = client.get_weight().await.map_err(ipc_error)?;
    Ok(WeightEnvelope { data })
}

#[tauri::command(rename_all = "camelCase")]
async fn api_set_weight(
    app: tauri::AppHandle,
    weight_in_pound: String,
) -> Result<serde_json::Value, String> {
    let client = build_client(&app).await?;
    client
        .set_weight(&weight_in_pound)
        .await
        .map_err(ipc_error)?;
    Ok(serde_json::json!({ "status": "ok" }))
}

// ─── Session-tracking sync (best-effort) ─────────────────────────────────

/// HOTWORX doesn't have a documented check-in endpoint, but the app
/// historically POSTed to `booking/checkinSession` on session start. We
/// keep that behavior as a best-effort sync; failures are silently
/// downgraded to "tracked locally."
#[tauri::command(rename_all = "camelCase")]
async fn api_checkin_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
) -> Result<serde_json::Value, String> {
    let _ = (app, session_record_id, lead_record_id);
    Ok(serde_json::json!({
        "status": "local_only",
        "msg": "Session tracked locally"
    }))
}

#[tauri::command(rename_all = "camelCase")]
async fn api_complete_session(
    app: tauri::AppHandle,
    session_record_id: String,
    lead_record_id: String,
    actual_duration_seconds: i64,
) -> Result<serde_json::Value, String> {
    let _ = (
        app,
        session_record_id,
        lead_record_id,
        actual_duration_seconds,
    );
    Ok(serde_json::json!({
        "status": "local_only",
        "msg": "Session completion tracked locally"
    }))
}

// ─── Token + device-id storage ───────────────────────────────────────────

/// Convenience for the legacy frontend code path that hashed passwords on
/// the WASM side. New code should use [`hotworx_api::password_hash`]
/// directly.
#[tauri::command]
fn hash_password(password: &str) -> String {
    password_hash(password)
}

/// Get (or generate + persist) the per-install device identifier.
#[tauri::command]
async fn get_device_id(app: tauri::AppHandle) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    if let Some(device_id) = store.get("device_id") {
        if let Some(id) = device_id.as_str() {
            return Ok(id.to_string());
        }
    }

    let device_id = uuid::Uuid::new_v4().to_string();
    store.set("device_id", serde_json::json!(device_id.clone()));
    store.save().map_err(|e| e.to_string())?;

    Ok(device_id)
}

#[tauri::command]
async fn store_auth_token(app: tauri::AppHandle, token: String) -> Result<(), String> {
    let device_id = get_device_id(app.clone()).await?;
    let encrypted = encrypt_value(&token, &device_id)?;
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("token", serde_json::json!(encrypted));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Read the persisted bearer token. If the stored ciphertext can't be
/// decrypted we treat it as corrupt and return `None`, which the frontend
/// surfaces as "session expired" — better than handing the encrypted blob
/// back as if it were the token (which the previous version did).
#[tauri::command]
async fn get_auth_token(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    let Some(token_val) = store.get("token") else {
        return Ok(None);
    };
    let Some(encrypted) = token_val.as_str() else {
        return Ok(None);
    };
    let device_id = get_device_id(app).await?;
    match decrypt_value(encrypted, &device_id) {
        Ok(token) => Ok(Some(token)),
        Err(e) => {
            eprintln!("[auth] discarding corrupt token: {}", e);
            Ok(None)
        }
    }
}

#[tauri::command]
async fn clear_auth_token(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.delete("token");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Pending-login storage (OTP flow continuation) ───────────────────────

#[tauri::command]
async fn store_pending_login(
    app: tauri::AppHandle,
    email: String,
    password: String,
    token: String,
) -> Result<(), String> {
    // The password is sensitive and lingers on disk until OTP succeeds, so
    // encrypt it at rest exactly like the bearer token (see store_auth_token).
    let device_id = get_device_id(app.clone()).await?;
    let encrypted_password = encrypt_value(&password, &device_id)?;
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.set("pending_email", serde_json::json!(email));
    store.set("pending_password", serde_json::json!(encrypted_password));
    store.set("pending_token", serde_json::json!(token));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_pending_login(
    app: tauri::AppHandle,
) -> Result<Option<(String, String, String)>, String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;

    let email = store
        .get("pending_email")
        .and_then(|v| v.as_str().map(String::from));
    let encrypted_password = store
        .get("pending_password")
        .and_then(|v| v.as_str().map(String::from));
    let token = store
        .get("pending_token")
        .and_then(|v| v.as_str().map(String::from));

    let (Some(e), Some(enc), Some(t)) = (email, encrypted_password, token) else {
        return Ok(None);
    };
    let device_id = get_device_id(app).await?;
    // Drop the pending login if the password can't be decrypted rather than
    // surfacing a hard error — the user can simply log in again.
    match decrypt_value(&enc, &device_id) {
        Ok(password) => Ok(Some((e, password, t))),
        Err(err) => {
            eprintln!("[auth] discarding undecryptable pending login: {}", err);
            Ok(None)
        }
    }
}

#[tauri::command]
async fn clear_pending_login(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("auth.json").map_err(|e| e.to_string())?;
    store.delete("pending_email");
    store.delete("pending_password");
    store.delete("pending_token");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Preferences (preferred location + session type) ─────────────────────

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

#[tauri::command]
async fn get_preferred_location(app: tauri::AppHandle) -> Result<Option<(String, String)>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let location_id = store
        .get("preferred_location_id")
        .and_then(|v| v.as_str().map(String::from));
    let location_name = store
        .get("preferred_location_name")
        .and_then(|v| v.as_str().map(String::from));

    match (location_id, location_name) {
        (Some(id), Some(name)) => Ok(Some((id, name))),
        _ => Ok(None),
    }
}

#[tauri::command]
async fn clear_preferred_location(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.delete("preferred_location_id");
    store.delete("preferred_location_name");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
async fn store_preferred_session_type(
    app: tauri::AppHandle,
    session_type: String,
    session_type_display: String,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("preferred_session_type", serde_json::json!(session_type));
    store.set(
        "preferred_session_type_display",
        serde_json::json!(session_type_display),
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_preferred_session_type(
    app: tauri::AppHandle,
) -> Result<Option<(String, String)>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let session_type = store
        .get("preferred_session_type")
        .and_then(|v| v.as_str().map(String::from));
    let display = store
        .get("preferred_session_type_display")
        .and_then(|v| v.as_str().map(String::from));

    match (session_type, display) {
        (Some(t), Some(d)) => Ok(Some((t, d))),
        _ => Ok(None),
    }
}

// ─── Local session tracking ──────────────────────────────────────────────

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

#[tauri::command]
async fn get_active_session(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    Ok(store.get("active_session"))
}

#[tauri::command]
async fn clear_active_session(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    store.delete("active_session");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn store_session_history(
    app: tauri::AppHandle,
    session: serde_json::Value,
) -> Result<(), String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;

    let mut history: Vec<serde_json::Value> = store
        .get("session_history")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    history.push(session);

    if history.len() > 100 {
        let skip_count = history.len() - 100;
        history = history.into_iter().skip(skip_count).collect();
    }

    store.set("session_history", serde_json::json!(history));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_session_history(app: tauri::AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;

    let history: Vec<serde_json::Value> = store
        .get("session_history")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    Ok(history)
}

// ─── Date helper for upcoming-sessions stitching ─────────────────────────

fn get_date_with_offset(days: i64) -> String {
    let now = chrono::Local::now() + chrono::Duration::days(days);
    now.format("%Y-%m-%d").to_string()
}

// ─── Tauri bootstrap ─────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // Auth + storage
            hash_password,
            get_device_id,
            store_auth_token,
            get_auth_token,
            clear_auth_token,
            store_pending_login,
            get_pending_login,
            clear_pending_login,
            // Preferences
            store_preferred_location,
            get_preferred_location,
            clear_preferred_location,
            store_preferred_session_type,
            get_preferred_session_type,
            // API: auth
            api_login_with_password,
            api_verify_otp,
            // API: dashboard / sessions
            api_get_dashboard,
            api_get_upcoming_sessions,
            api_get_activity_history,
            // API: booking
            api_get_locations,
            api_get_session_types,
            api_show_slots,
            api_book_session,
            api_delete_session,
            // API: profile / goals / weight / stats
            api_view_profile,
            api_update_profile,
            api_get_summary,
            api_get_thirty_day_summary,
            api_get_ninety_day_summary,
            api_get_calorie_stats,
            api_view_goals,
            api_update_goals,
            api_get_weight,
            api_set_weight,
            // Local session tracking
            store_active_session,
            get_active_session,
            clear_active_session,
            store_session_history,
            get_session_history,
            api_checkin_session,
            api_complete_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let device_id = "device-abc";
        let plaintext = "secret-token-12345";
        let ciphertext = encrypt_value(plaintext, device_id).unwrap();
        assert_ne!(
            ciphertext, plaintext,
            "ciphertext must differ from plaintext"
        );
        let decrypted = decrypt_value(&ciphertext, device_id).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_ciphertext_each_call() {
        // Random nonce should make repeated encryption outputs differ.
        let device_id = "device-abc";
        let plaintext = "hello";
        let a = encrypt_value(plaintext, device_id).unwrap();
        let b = encrypt_value(plaintext, device_id).unwrap();
        assert_ne!(
            a, b,
            "AES-GCM with random nonce should not produce stable output"
        );
    }

    #[test]
    fn decrypt_with_wrong_device_id_returns_err() {
        let ciphertext = encrypt_value("token", "right-device").unwrap();
        let result = decrypt_value(&ciphertext, "wrong-device");
        assert!(
            result.is_err(),
            "decrypt with mismatched device id must fail, got {:?}",
            result
        );
    }

    #[test]
    fn decrypt_garbage_returns_err() {
        // Not base64 → decode error path.
        assert!(decrypt_value("not base64!!", "device").is_err());
        // Valid base64 but too short → length-check error path.
        assert!(decrypt_value("AAAA", "device").is_err());
    }

    #[test]
    fn derive_key_is_stable_per_device_id() {
        let a = derive_key("same-id");
        let b = derive_key("same-id");
        assert_eq!(a, b);
    }

    #[test]
    fn derive_key_differs_across_device_ids() {
        let a = derive_key("device-a");
        let b = derive_key("device-b");
        assert_ne!(a, b);
    }

    #[test]
    fn ipc_error_marks_auth_expired_with_sentinel() {
        let s = ipc_error(HotworxError::AuthExpired);
        assert!(
            s.starts_with(AUTH_EXPIRED_PREFIX),
            "expected AUTH_EXPIRED prefix, got {:?}",
            s
        );
    }

    #[test]
    fn ipc_error_passes_other_errors_through() {
        let s = ipc_error(HotworxError::Http {
            status: 500,
            body: "server died".into(),
        });
        assert!(!s.contains(AUTH_EXPIRED_PREFIX));
        assert!(s.contains("500"));
    }

    #[test]
    fn auth_sentinels_are_mutually_unambiguous() {
        // The frontend distinguishes the two cases with substring matching, so
        // neither prefix may contain the other.
        assert!(!NOT_AUTHENTICATED_PREFIX.contains(AUTH_EXPIRED_PREFIX));
        assert!(!AUTH_EXPIRED_PREFIX.contains(NOT_AUTHENTICATED_PREFIX));
    }

    #[test]
    fn hash_password_command_matches_known_sha256() {
        // Wire-compatible with what the HOTWORX API expects.
        assert_eq!(
            hash_password("password"),
            "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
        );
    }
}
