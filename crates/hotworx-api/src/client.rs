//! The [`HotworxClient`] type and its endpoint methods.

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::auth::password_hash;
use crate::error::{HotworxError, Result};
use crate::headers::{apply_app_headers, BASE_URL};
use crate::models::*;

/// HOTWORX API client.
///
/// `HotworxClient` is a thin, stateless wrapper around a `reqwest::Client`
/// that adds the right HOTWORX-app headers, applies the bearer token (when
/// set), and parses the per-endpoint response shapes into typed models.
///
/// The client itself doesn't persist tokens or device IDs — that's the
/// caller's responsibility. Build one client per request batch, or keep a
/// long-lived one if you don't mind the token going away on logout.
#[derive(Debug, Clone)]
pub struct HotworxClient {
    device_id: String,
    token: Option<String>,
    base_url: String,
    http: reqwest::Client,
}

impl HotworxClient {
    /// Construct a client with no auth token. Suitable for the login flow.
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            token: None,
            base_url: BASE_URL.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Builder-style — attach a bearer token and return the client. Useful
    /// when chaining off [`HotworxClient::new`] for one-off calls.
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Override the API base URL. Mostly useful for tests that point the
    /// client at a mock server; production callers should rely on the
    /// default and never call this.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set or replace the bearer token in place.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    /// Drop the bearer token. Subsequent authenticated calls will fail with
    /// [`HotworxError::AuthExpired`].
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Currently-set bearer token, if any.
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Per-device identifier this client is sending.
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    // ─── Auth ────────────────────────────────────────────────────────────

    /// First step of password login. Hashes `password` (SHA-256, per
    /// HOTWORX's protocol) and POSTs `loginwithpassword`. The returned
    /// [`LoginResponse`] either contains a `token` directly or signals OTP
    /// is required via [`LoginResponse::requires_otp`].
    pub async fn login_with_password(&self, email: &str, password: &str) -> Result<LoginResponse> {
        let hash = password_hash(password);
        self.post_form_raw(
            "loginwithpassword",
            &[
                ("email_address", email),
                ("password", &hash),
                ("device_id", &self.device_id),
            ],
            false,
        )
        .await
    }

    /// Complete two-factor login by submitting an OTP code. The
    /// `password_hash` argument is the SHA-256 hex digest of the user's
    /// password — typically the same value you computed (or had us compute)
    /// during [`login_with_password`](Self::login_with_password).
    pub async fn verify_otp(
        &self,
        email: &str,
        password_hash: &str,
        otp: &str,
    ) -> Result<LoginResponse> {
        self.post_form_raw(
            "verifyOtp",
            &[
                ("email_address", email),
                ("password", password_hash),
                ("phone_number", ""),
                ("device_id", &self.device_id),
                ("otp", otp),
                ("type", "password"),
            ],
            true,
        )
        .await
    }

    // ─── Dashboard / sessions ────────────────────────────────────────────

    /// Fetch today's pending and completed sessions plus the lifetime
    /// summary card. `current_date` should be `YYYY-MM-DD` if provided.
    pub async fn get_dashboard(&self, current_date: Option<&str>) -> Result<DashboardData> {
        let mut params: Vec<(&str, &str)> = Vec::new();
        if let Some(date) = current_date {
            params.push(("current_date", date));
        }
        let env: Envelope<DashboardData> = self.post_form("getDashboard", &params, true).await?;
        env.into_data()
    }

    // ─── Booking ─────────────────────────────────────────────────────────

    /// List the studio locations the member can book at.
    pub async fn get_locations(&self) -> Result<Vec<Location>> {
        let env: Envelope<LocationsData> = self.get("booking/getBookingLocations_v2", true).await?;
        Ok(env.into_data()?.locations.unwrap_or_default())
    }

    /// List the session types (e.g. `"HOT YOGA"`) available at a location
    /// on a particular date. Date format: `YYYY-MM-DD`.
    pub async fn get_session_types(
        &self,
        location_id: &str,
        date: &str,
    ) -> Result<Vec<SessionType>> {
        // The endpoint returns `{ list: [SessionType] }` rather than the
        // standard envelope.
        #[derive(Deserialize)]
        struct ListWrapper {
            #[serde(default)]
            list: Vec<SessionType>,
        }
        let resp: ListWrapper = self
            .post_form(
                "booking/getLevelTwo_v2",
                &[
                    ("selected_location_id", location_id),
                    ("selected_date", date),
                    ("view_type", "by_session_type"),
                ],
                true,
            )
            .await?;
        Ok(resp.list)
    }

    /// Available time slots for a session type at a location on a date.
    /// Returns an empty vec if the API gives no slots; only network or
    /// auth failures produce an error.
    pub async fn show_slots(
        &self,
        location_id: &str,
        date: &str,
        session_type: &str,
    ) -> Result<Vec<TimeSlot>> {
        // `showSlots` returns either a bare array of slots or
        // `{ data: { slots: [...] } }`. We accept both.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum SlotsShape {
            Bare(Vec<TimeSlot>),
            Wrapped { data: Option<SlotsInner> },
        }
        #[derive(Deserialize)]
        struct SlotsInner {
            slots: Option<Vec<TimeSlot>>,
        }
        let resp: SlotsShape = self
            .post_form(
                "booking/showSlots",
                &[
                    ("selected_date", date),
                    ("selected_location_id", location_id),
                    ("view_type", "by_session_type"),
                    ("selected_time", "all"),
                    ("session_type", session_type),
                ],
                true,
            )
            .await?;
        Ok(match resp {
            SlotsShape::Bare(v) => v,
            SlotsShape::Wrapped { data } => data.and_then(|d| d.slots).unwrap_or_default(),
        })
    }

    /// Book a single session.
    pub async fn book_session(
        &self,
        location_id: &str,
        date: &str,
        session_type: &str,
        sauna_no: &str,
        time_slot: &str,
    ) -> Result<BookSessionResponse> {
        self.post_form_raw(
            "booking/bookSession_v2",
            &[
                ("sauna_no", sauna_no),
                ("time_slot", time_slot),
                ("booking_date", date),
                ("session_type", session_type),
                ("selected_location_id", location_id),
            ],
            true,
        )
        .await
    }

    /// Cancel a previously booked session.
    pub async fn delete_session(
        &self,
        session_record_id: &str,
        lead_record_id: &str,
    ) -> Result<()> {
        // Some HOTWORX backends expect the booking ID in a `booking_id`
        // field even though the value is the session record ID. We send
        // it under the name HOTWORX uses on the wire.
        let _: serde_json::Value = self
            .post_form(
                "booking/deleteSession",
                &[
                    ("booking_id", session_record_id),
                    ("lead_record_id", lead_record_id),
                ],
                true,
            )
            .await?;
        Ok(())
    }

    // ─── Profile ─────────────────────────────────────────────────────────

    /// Fetch the full member profile.
    pub async fn view_profile(&self) -> Result<ProfileData> {
        // The view_profile response is nested as `{ data: [{ data: {...} }] }`
        // on most accounts but flattens to `{ data: {...} }` on others. We
        // accept both shapes.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ProfileShape {
            Nested { data: Vec<ProfileWrapper> },
            Flat { data: Box<ProfileData> },
        }
        #[derive(Deserialize)]
        struct ProfileWrapper {
            data: ProfileData,
        }
        let resp: ProfileShape = self.post_form("general/view_profile", &[], true).await?;
        Ok(match resp {
            ProfileShape::Nested { mut data } => {
                if let Some(first) = data.drain(..).next() {
                    first.data
                } else {
                    return Err(HotworxError::Http {
                        status: 200,
                        body: "view_profile returned an empty `data` array".into(),
                    });
                }
            }
            ProfileShape::Flat { data } => *data,
        })
    }

    /// Update mutable profile fields. Pass empty strings for fields you
    /// don't want to change.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_profile(
        &self,
        first_name: &str,
        last_name: &str,
        dob: &str,
        gender: &str,
        height: &str,
        weight: &str,
        address: &str,
    ) -> Result<()> {
        let _: serde_json::Value = self
            .post_form(
                "general/update_profile",
                &[
                    ("first_name", first_name),
                    ("last_name", last_name),
                    ("dob", dob),
                    ("gender", gender),
                    ("height", height),
                    ("weight", weight),
                    ("address", address),
                ],
                true,
            )
            .await?;
        Ok(())
    }

    // ─── Goals ───────────────────────────────────────────────────────────

    /// View the user's current goal settings.
    pub async fn view_goals(&self) -> Result<GoalsData> {
        let env: Envelope<GoalsData> = self.post_form("general/viewGoals", &[], true).await?;
        env.into_data()
    }

    /// Update the user's goal settings.
    pub async fn update_goals(
        &self,
        current_weight: &str,
        target_weight: &str,
        target_weight_goal_date: &str,
        weekly_session_goal: &str,
    ) -> Result<()> {
        let _: serde_json::Value = self
            .post_form(
                "general/updateGoals",
                &[
                    ("current_weight", current_weight),
                    ("target_weight", target_weight),
                    ("target_weight_goal_date", target_weight_goal_date),
                    ("weekly_session_goal", weekly_session_goal),
                ],
                true,
            )
            .await?;
        Ok(())
    }

    // ─── Weight ──────────────────────────────────────────────────────────

    /// Recent weight log entries, newest first.
    pub async fn get_weight(&self) -> Result<Vec<WeightEntry>> {
        #[derive(Deserialize)]
        struct WeightShape {
            #[serde(default)]
            data: Vec<WeightEntry>,
        }
        let resp: WeightShape = self.post_form("general/get_weight", &[], true).await?;
        Ok(resp.data)
    }

    /// Record a new weight reading in pounds.
    pub async fn set_weight(&self, weight_in_pound: &str) -> Result<()> {
        let _: serde_json::Value = self
            .post_form(
                "general/set_weight",
                &[("weight_in_pound", weight_in_pound)],
                true,
            )
            .await?;
        Ok(())
    }

    // ─── Stats / summaries ───────────────────────────────────────────────

    /// Per-day calorie breakdown for `date` (`YYYY-MM-DD`).
    pub async fn get_summary(&self, date: &str) -> Result<DailySummary> {
        let env: Envelope<DailySummary> = self
            .post_form("general/get_summary", &[("date", date)], true)
            .await?;
        env.into_data()
    }

    /// Rolling 30-day stats. The HOTWORX API double-wraps the payload
    /// (`data: [{ data: {...} }]`), which this helper unwraps.
    pub async fn get_thirty_day_summary(&self) -> Result<ThirtyDaySummary> {
        #[derive(Deserialize)]
        struct ThirtyShape {
            data: Option<Vec<ThirtyDaySummaryWrapper>>,
        }
        let resp: ThirtyShape = self
            .post_form("general/get_summary_thirty_days", &[], true)
            .await?;
        let inner = resp
            .data
            .and_then(|mut v| v.drain(..).next())
            .and_then(|w| w.data)
            .unwrap_or_default();
        Ok(inner)
    }

    /// Rolling 90-day calorie progress and level.
    pub async fn get_ninety_day_summary(&self) -> Result<NinetyDaySummary> {
        let env: Envelope<NinetyDaySummary> = self
            .post_form("general/get_ninety_days_summary", &[], true)
            .await?;
        env.into_data()
    }

    /// Lifetime calorie statistics.
    pub async fn get_calorie_stats(&self) -> Result<CalorieStats> {
        let env: Envelope<CalorieStats> = self
            .post_form("general/view_calorie_stats", &[], true)
            .await?;
        env.into_data()
    }

    // ─── Activity history ────────────────────────────────────────────────

    /// Paginated activity history. `session_type` filters by name (e.g.
    /// `"HOT YOGA"`); pass `None` to get every session type.
    pub async fn get_activity_history(
        &self,
        page_no: u32,
        page_limit: u32,
        session_type: Option<&str>,
    ) -> Result<ActivityPage> {
        let session_filter = session_type
            .filter(|s| !s.is_empty() && *s != "all")
            .map(urlencoding::encode)
            .unwrap_or_default();
        let endpoint = format!(
            "activities/ActivityByLifeTime?page_no={}&page_limit={}&session_type={}",
            page_no, page_limit, session_filter
        );
        self.get(&endpoint, true).await
    }

    // ─── Internal helpers ────────────────────────────────────────────────

    async fn post_form_raw<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        require_auth: bool,
    ) -> Result<T> {
        self.post_form(endpoint, params, require_auth).await
    }

    async fn post_form<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        require_auth: bool,
    ) -> Result<T> {
        if require_auth && self.token.is_none() {
            return Err(HotworxError::AuthExpired);
        }
        let url = format!("{}/{}", self.base_url, endpoint);
        let body = encode_form(params);
        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body);
        req = apply_app_headers(req, &self.device_id);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        consume_response(resp).await
    }

    async fn get<T: DeserializeOwned>(&self, endpoint: &str, require_auth: bool) -> Result<T> {
        if require_auth && self.token.is_none() {
            return Err(HotworxError::AuthExpired);
        }
        let url = format!("{}/{}", self.base_url, endpoint);
        let mut req = self.http.get(&url);
        req = apply_app_headers(req, &self.device_id);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        consume_response(resp).await
    }
}

fn encode_form(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

async fn consume_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        return Err(HotworxError::from_status(status.as_u16(), body));
    }
    Ok(serde_json::from_str(&body)?)
}

/// Generic envelope used by most HOTWORX `general/*` endpoints. Internal
/// to the crate — public methods unwrap `data` for the caller.
#[derive(Debug, Clone, Deserialize)]
struct Envelope<T> {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    data: Option<T>,
}

impl<T> Envelope<T> {
    fn into_data(self) -> Result<T> {
        if let Some(err) = self.error.filter(|e| !e.is_empty()) {
            return Err(HotworxError::Http {
                status: 200,
                body: err,
            });
        }
        self.data.ok_or_else(|| HotworxError::Http {
            status: 200,
            body: format!(
                "missing `data` field; status={:?}, msg={:?}",
                self.status, self.msg
            ),
        })
    }
}
