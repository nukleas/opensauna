use anyhow::Result;
use reqwest::Client;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const BASE_URL: &str = "https://sailposapi.hotworx.net/api/v1";

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
    #[serde(skip_serializing_if = "Option::is_none")]
    _pending_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _pending_password_hash: Option<String>,
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

    fn token(&self) -> std::result::Result<&str, String> {
        self.auth_token
            .as_deref()
            .ok_or_else(|| "Not authenticated. Use hotworx_login first.".to_string())
    }

    fn ensure_loaded(self) -> Self {
        if self.auth_token.is_none() {
            Self::load()
        } else {
            self
        }
    }
}

// ── HTTP helpers ─────────────────────────────────────────────

async fn api_post_form(
    client: &Client,
    endpoint: &str,
    params: HashMap<&str, String>,
    config: &Config,
    auth: bool,
) -> std::result::Result<serde_json::Value, String> {
    let url = format!("{}/{}", BASE_URL, endpoint);

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "okhttp/4.12.0")
        .header("sec-ch-ua-platform", "Android")
        .header("application-version", "6.5.5")
        .header("device-id", &config.device_id);

    if auth {
        let token = config.token()?;
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let body: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let resp = req
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let text = resp.text().await.map_err(|e| format!("Read failed: {e}"))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("Parse error: {e} - {}", &text[..text.len().min(200)]))
}

async fn api_get(
    client: &Client,
    endpoint: &str,
    config: &Config,
) -> std::result::Result<serde_json::Value, String> {
    let url = format!("{}/{}", BASE_URL, endpoint);
    let token = config.token()?;

    let resp = client
        .get(&url)
        .header("User-Agent", "okhttp/4.12.0")
        .header("sec-ch-ua-platform", "Android")
        .header("application-version", "6.5.5")
        .header("device-id", &config.device_id)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    let text = resp.text().await.map_err(|e| format!("Read failed: {e}"))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("Parse error: {e} - {}", &text[..text.len().min(200)]))
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn pretty(val: &serde_json::Value) -> String {
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
    client: Client,
    config: Arc<Mutex<Config>>,
    tool_router: ToolRouter<Self>,
}

impl HotworxService {
    fn new() -> Self {
        let client = Client::new();
        let config = Arc::new(Mutex::new(Config::load()));
        Self {
            client,
            config,
            tool_router: Self::tool_router(),
        }
    }

    async fn cfg(&self) -> Config {
        self.config.lock().await.clone().ensure_loaded()
    }
}

#[tool_router(router = tool_router)]
impl HotworxService {
    #[tool(description = "Log in to HOTWORX with email and password. Required before any other operation.")]
    async fn hotworx_login(&self, Parameters(args): Parameters<LoginArgs>) -> String {
        let mut config = self.config.lock().await;
        *config = Config::load();

        let password_hash = sha256_hex(&args.password);
        let mut params = HashMap::new();
        params.insert("email_address", args.email.clone());
        params.insert("password", password_hash.clone());
        params.insert("device_id", config.device_id.clone());

        let res = match api_post_form(&self.client, "loginwithpassword", params, &config, false).await {
            Ok(v) => v,
            Err(e) => return format!("Login failed: {e}"),
        };

        if let Some(err) = res.get("error").and_then(|e| e.as_str()) {
            return format!("Login failed: {err}");
        }

        if res.get("two_factor").and_then(|v| v.as_str()) == Some("yes") {
            if let Some(token) = res.get("token").and_then(|t| t.as_str()) {
                config.auth_token = Some(token.to_string());
            }
            config._pending_email = Some(args.email);
            config._pending_password_hash = Some(password_hash);
            config.save();
            return "Two-factor auth required. Use hotworx_verify_otp with the code sent to your phone.".to_string();
        }

        if let Some(token) = res.get("token").and_then(|t| t.as_str()) {
            config.auth_token = Some(token.to_string());
            config.save();
            return "Login successful! You can now book sessions.".to_string();
        }

        "Login failed — unknown error.".to_string()
    }

    #[tool(description = "Verify OTP code sent to your phone after login (only if two-factor auth is enabled).")]
    async fn hotworx_verify_otp(&self, Parameters(args): Parameters<OtpArgs>) -> String {
        let mut config = self.config.lock().await;
        let (email, password_hash) = match (config._pending_email.clone(), config._pending_password_hash.clone()) {
            (Some(e), Some(p)) => (e, p),
            _ => return "No pending login. Call hotworx_login first.".to_string(),
        };

        let mut params = HashMap::new();
        params.insert("email_address", email);
        params.insert("password", password_hash);
        params.insert("phone_number", String::new());
        params.insert("device_id", config.device_id.clone());
        params.insert("otp", args.otp);
        params.insert("type", "password".to_string());

        let res = match api_post_form(&self.client, "verifyOtp", params, &config, true).await {
            Ok(v) => v,
            Err(e) => return format!("OTP failed: {e}"),
        };

        if let Some(err) = res.get("error").and_then(|e| e.as_str()) {
            return format!("OTP failed: {err}");
        }

        if let Some(new_token) = res.get("token").and_then(|t| t.as_str()) {
            config.auth_token = Some(new_token.to_string());
        }
        config._pending_email = None;
        config._pending_password_hash = None;
        config.save();

        "OTP verified! You can now book sessions.".to_string()
    }

    #[tool(description = "Log out and clear stored credentials.")]
    async fn hotworx_logout(&self) -> String {
        let mut config = self.config.lock().await;
        config.auth_token = None;
        config._pending_email = None;
        config._pending_password_hash = None;
        config.save();
        "Logged out.".to_string()
    }

    #[tool(description = "Get today's sessions, completed sessions, and summary stats. Optionally pass a date (YYYY-MM-DD).")]
    async fn hotworx_dashboard(&self, Parameters(args): Parameters<DashboardArgs>) -> String {
        let cfg = self.cfg().await;
        let mut params = HashMap::new();
        if let Some(date) = args.date {
            params.insert("current_date", date);
        }
        match api_post_form(&self.client, "getDashboard", params, &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get all HOTWORX studio locations available for booking.")]
    async fn hotworx_get_locations(&self) -> String {
        let cfg = self.cfg().await;
        match api_get(&self.client, "booking/getBookingLocations_v2", &cfg).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get available session types (e.g. HOT YOGA, HOT BLAST) for a location and date.")]
    async fn hotworx_get_session_types(&self, Parameters(args): Parameters<LocationDateArgs>) -> String {
        let cfg = self.cfg().await;
        let mut params = HashMap::new();
        params.insert("selected_location_id", args.location_id);
        params.insert("selected_date", args.date);
        params.insert("view_type", "by_session_type".to_string());
        match api_post_form(&self.client, "booking/getLevelTwo_v2", params, &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get available time slots for a session type at a location and date.")]
    async fn hotworx_get_available_slots(&self, Parameters(args): Parameters<SlotsArgs>) -> String {
        let cfg = self.cfg().await;
        let mut params = HashMap::new();
        params.insert("selected_date", args.date);
        params.insert("selected_location_id", args.location_id);
        params.insert("view_type", "by_session_type".to_string());
        params.insert("selected_time", "all".to_string());
        params.insert("session_type", args.session_type);
        match api_post_form(&self.client, "booking/showSlots", params, &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Book a HOTWORX session. Call once per slot for back-to-back sessions.")]
    async fn hotworx_book_session(&self, Parameters(args): Parameters<BookArgs>) -> String {
        let cfg = self.cfg().await;
        let mut params = HashMap::new();
        params.insert("sauna_no", args.sauna_no);
        params.insert("time_slot", args.time_slot);
        params.insert("booking_date", args.date);
        params.insert("session_type", args.session_type);
        params.insert("selected_location_id", args.location_id);
        match api_post_form(&self.client, "booking/bookSession_v2", params, &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Cancel a booked session. Requires session_record_id and lead_record_id from dashboard.")]
    async fn hotworx_cancel_session(&self, Parameters(args): Parameters<CancelArgs>) -> String {
        let cfg = self.cfg().await;
        let mut params = HashMap::new();
        params.insert("booking_id", args.session_record_id);
        params.insert("lead_record_id", args.lead_record_id);
        match api_post_form(&self.client, "booking/deleteSession", params, &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get your HOTWORX profile (name, email, phone, height, weight).")]
    async fn hotworx_get_profile(&self) -> String {
        let cfg = self.cfg().await;
        match api_post_form(&self.client, "general/view_profile", HashMap::new(), &cfg, true).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get past session history with pagination.")]
    async fn hotworx_get_activity_history(&self, Parameters(args): Parameters<HistoryArgs>) -> String {
        let cfg = self.cfg().await;
        let mut endpoint = format!(
            "activities/ActivityByLifeTime?page_no={}&page_limit={}",
            args.page, args.limit
        );
        if let Some(st) = &args.session_type {
            endpoint.push_str(&format!("&session_type={}", urlencoding::encode(st)));
        }
        match api_get(&self.client, &endpoint, &cfg).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
        }
    }

    #[tool(description = "Get lifetime calorie statistics.")]
    async fn hotworx_get_calorie_stats(&self) -> String {
        let cfg = self.cfg().await;
        match api_get(&self.client, "general/view_calorie_stats", &cfg).await {
            Ok(v) => pretty(&v),
            Err(e) => e,
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
