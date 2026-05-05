//! HOTWORX MCP server.
//!
//! Exposes a small set of HOTWORX tools over the Model Context Protocol so
//! Claude Code (and any other MCP client) can read your dashboard, browse
//! locations, and book sessions on your behalf.
//!
//! The server is a thin layer over [`hotworx_api::HotworxClient`]; tokens
//! and the per-install device ID live in `~/.hotworx-mcp/config.json`.

use anyhow::Result;
use hotworx_api::{HotworxClient, HotworxError, password_hash};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Config persistence ───────────────────────────────────────

fn config_path() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hotworx-mcp");
    std::fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Config {
    device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_token: Option<String>,
    /// Email saved between `hotworx_login` and `hotworx_verify_otp` so
    /// the OTP step has the context it needs without prompting again.
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_email: Option<String>,
    /// SHA-256 hash of the plaintext password from the login step. The
    /// HOTWORX OTP endpoint takes the same hash; it never sees plaintext.
    #[serde(skip_serializing_if = "Option::is_none")]
    pending_password_hash: Option<String>,
}

impl Config {
    fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                let cfg = Config {
                    device_id: uuid::Uuid::new_v4().to_string(),
                    ..Default::default()
                };
                cfg.save();
                cfg
            })
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            std::fs::write(config_path(), json).ok();
        }
    }
}

/// Build an authenticated [`HotworxClient`] from the persisted config.
/// Returns the not-authenticated message used by the MCP tools below.
fn client_for(config: &Config) -> std::result::Result<HotworxClient, String> {
    let token = config
        .auth_token
        .as_deref()
        .ok_or_else(|| "Not authenticated. Use hotworx_login first.".to_string())?;
    Ok(HotworxClient::new(config.device_id.clone()).with_token(token))
}

/// Format a `HotworxError` for the MCP client.
fn format_err(err: HotworxError) -> String {
    match err {
        HotworxError::AuthExpired => {
            "Session expired. Use hotworx_login to sign in again.".to_string()
        }
        other => format!("HOTWORX error: {}", other),
    }
}

fn pretty<T: Serialize>(val: &T) -> String {
    serde_json::to_string_pretty(val).unwrap_or_default()
}

// ── Tool parameter types ─────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct LoginArgs {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct OtpArgs {
    /// The 6-digit OTP code
    otp: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct DashboardArgs {
    /// Date in YYYY-MM-DD format, defaults to today
    #[serde(default)]
    date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct LocationDateArgs {
    /// The location ID from hotworx_get_locations
    location_id: String,
    /// Date in YYYY-MM-DD format
    date: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SlotsArgs {
    /// The location ID
    location_id: String,
    /// Date in YYYY-MM-DD format
    date: String,
    /// Session type, e.g. HOT YOGA or HOT BLAST
    session_type: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct BookArgs {
    /// The location ID
    location_id: String,
    /// Date in YYYY-MM-DD format
    date: String,
    /// Session type, e.g. HOT YOGA or HOT BLAST
    session_type: String,
    /// Time slot, e.g. 06:45PM-07:00PM
    time_slot: String,
    /// Sauna number from available slots
    sauna_no: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct CancelArgs {
    /// Session record ID from booking or dashboard
    session_record_id: String,
    /// Lead record ID from booking or dashboard
    lead_record_id: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct HistoryArgs {
    /// Page number, starts at 1
    #[serde(default = "default_page")]
    page: u32,
    /// Results per page
    #[serde(default = "default_limit")]
    limit: u32,
    /// Filter by session type, e.g. HOT YOGA
    #[serde(default)]
    session_type: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_limit() -> u32 {
    20
}

// ── MCP Service ──────────────────────────────────────────────

#[derive(Clone)]
struct HotworxService {
    config: Arc<Mutex<Config>>,
    tool_router: ToolRouter<Self>,
}

impl HotworxService {
    fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(Config::load())),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl HotworxService {
    #[tool(
        description = "Log in to HOTWORX with email and password. Required before any other operation."
    )]
    async fn hotworx_login(&self, Parameters(args): Parameters<LoginArgs>) -> String {
        let mut config = self.config.lock().await;
        let client = HotworxClient::new(config.device_id.clone());
        let resp = match client
            .login_with_password(&args.email, &args.password)
            .await
        {
            Ok(r) => r,
            Err(e) => return format!("Login failed: {}", format_err(e)),
        };

        if let Some(err) = resp.error.as_deref() {
            return format!("Login failed: {}", err);
        }

        if resp.requires_otp() {
            if let Some(token) = resp.token {
                config.auth_token = Some(token);
            }
            config.pending_email = Some(args.email);
            config.pending_password_hash = Some(password_hash(&args.password));
            config.save();
            return "Two-factor auth required. Use hotworx_verify_otp with the code sent to your phone.".to_string();
        }

        if let Some(token) = resp.token {
            config.auth_token = Some(token);
            config.save();
            return "Login successful! You can now book sessions.".to_string();
        }

        "Login failed — unknown error.".to_string()
    }

    #[tool(
        description = "Verify OTP code sent to your phone after login (only if two-factor auth is enabled)."
    )]
    async fn hotworx_verify_otp(&self, Parameters(args): Parameters<OtpArgs>) -> String {
        let mut config = self.config.lock().await;
        let (email, hash) = match (
            config.pending_email.clone(),
            config.pending_password_hash.clone(),
        ) {
            (Some(e), Some(p)) => (e, p),
            _ => return "No pending login. Call hotworx_login first.".to_string(),
        };

        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        let resp = match client.verify_otp(&email, &hash, &args.otp).await {
            Ok(r) => r,
            Err(e) => return format!("OTP failed: {}", format_err(e)),
        };

        if let Some(err) = resp.error.as_deref() {
            return format!("OTP failed: {}", err);
        }

        if let Some(new_token) = resp.token {
            config.auth_token = Some(new_token);
        }
        config.pending_email = None;
        config.pending_password_hash = None;
        config.save();

        "OTP verified! You can now book sessions.".to_string()
    }

    #[tool(description = "Log out and clear stored credentials.")]
    async fn hotworx_logout(&self) -> String {
        let mut config = self.config.lock().await;
        config.auth_token = None;
        config.pending_email = None;
        config.pending_password_hash = None;
        config.save();
        "Logged out.".to_string()
    }

    #[tool(
        description = "Get today's sessions, completed sessions, and summary stats. Optionally pass a date (YYYY-MM-DD)."
    )]
    async fn hotworx_dashboard(&self, Parameters(args): Parameters<DashboardArgs>) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client.get_dashboard(args.date.as_deref()).await {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Get all HOTWORX studio locations available for booking.")]
    async fn hotworx_get_locations(&self) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client.get_locations().await {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(
        description = "Get available session types (e.g. HOT YOGA, HOT BLAST) for a location and date."
    )]
    async fn hotworx_get_session_types(
        &self,
        Parameters(args): Parameters<LocationDateArgs>,
    ) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client
            .get_session_types(&args.location_id, &args.date)
            .await
        {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Get available time slots for a session type at a location and date.")]
    async fn hotworx_get_available_slots(&self, Parameters(args): Parameters<SlotsArgs>) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client
            .show_slots(&args.location_id, &args.date, &args.session_type)
            .await
        {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Book a HOTWORX session. Call once per slot for back-to-back sessions.")]
    async fn hotworx_book_session(&self, Parameters(args): Parameters<BookArgs>) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client
            .book_session(
                &args.location_id,
                &args.date,
                &args.session_type,
                &args.sauna_no,
                &args.time_slot,
            )
            .await
        {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(
        description = "Cancel a booked session. Requires session_record_id and lead_record_id from dashboard."
    )]
    async fn hotworx_cancel_session(&self, Parameters(args): Parameters<CancelArgs>) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client
            .delete_session(&args.session_record_id, &args.lead_record_id)
            .await
        {
            Ok(()) => "Session cancelled.".to_string(),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Get your HOTWORX profile (name, email, phone, height, weight).")]
    async fn hotworx_get_profile(&self) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client.view_profile().await {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Get past session history with pagination.")]
    async fn hotworx_get_activity_history(
        &self,
        Parameters(args): Parameters<HistoryArgs>,
    ) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client
            .get_activity_history(args.page, args.limit, args.session_type.as_deref())
            .await
        {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }

    #[tool(description = "Get lifetime calorie statistics.")]
    async fn hotworx_get_calorie_stats(&self) -> String {
        let config = self.config.lock().await;
        let client = match client_for(&config) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match client.get_calorie_stats().await {
            Ok(v) => pretty(&v),
            Err(e) => format_err(e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for HotworxService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("hotworx", "0.1.0"))
            .with_instructions(
                "HOTWORX session booking and management. Use hotworx_login first to authenticate.",
            )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = HotworxService::new();
    let server = service.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;
    Ok(())
}
