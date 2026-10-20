//! Time slots, session-type lookups, and booking responses.

use serde::{Deserialize, Deserializer, Serialize};

/// A bookable workout time slot.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TimeSlot {
    #[serde(default, deserialize_with = "deserialize_optional_i32_from_str")]
    pub duration: Option<i32>,
    pub session_name: Option<String>,
    /// Per-station availability strings — when these contain the substring
    /// `"available"` the station is bookable. Helper:
    /// [`TimeSlot::is_slot_available`].
    pub slot1: Option<String>,
    pub slot2: Option<String>,
    pub slot3: Option<String>,
    /// Sauna number. The API mis-spells this as `suana_no` on some
    /// endpoints; both spellings deserialize correctly.
    #[serde(
        default,
        alias = "suana_no",
        deserialize_with = "deserialize_optional_string_from_number"
    )]
    pub sauna_no: Option<String>,
    pub time_slot: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool_from_str")]
    pub is_available: Option<bool>,
}

impl TimeSlot {
    /// Whether the given per-station slot string represents an available
    /// (bookable) station.
    pub fn is_slot_available(slot: &Option<String>) -> bool {
        slot.as_ref()
            .map(|v| v.contains("available"))
            .unwrap_or(false)
    }
}

/// A session type the user can book at a given location/date — e.g.
/// `"HOT YOGA"`, `"HOT PILATES"`. Returned by
/// [`get_session_types`](crate::HotworxClient::get_session_types).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SessionType {
    /// Display label (often the same as `value`).
    pub slot: Option<String>,
    /// API-canonical value to send back when booking or fetching slots.
    pub value: Option<String>,
}

/// Response from `booking/bookSession_v2`.
///
/// HOTWORX answers a booking request with HTTP 200 whether or not a
/// session was actually created, so the body has to be inspected. The
/// official app reads `success` / `session_id` for a confirmed booking and
/// `message_popup` / `card_available` / `payment_status` / `add_card_url`
/// when the location charges a reciprocal fee and wants a payment step
/// first. Older backends used `error` and `data`. All of them are kept so
/// [`BookSessionResponse::outcome`] can classify whatever era answers.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BookSessionResponse {
    pub status: Option<String>,
    pub msg: Option<String>,
    pub message: Option<String>,
    pub error: Option<String>,
    pub data: Option<BookedSessionData>,
    /// `"true"` / `true` / `"1"` when the server confirms the booking.
    #[serde(default, deserialize_with = "deserialize_tristate_flag")]
    pub success: Option<bool>,
    /// Record id of the created session (number on the wire).
    #[serde(default, deserialize_with = "deserialize_optional_string_from_number")]
    pub session_id: Option<String>,
    /// Human-readable message, e.g. a reciprocal-fee notice.
    pub text: Option<String>,
    pub description: Option<String>,
    /// Server wants the app to show a payment-card prompt before booking.
    #[serde(default, deserialize_with = "deserialize_tristate_flag")]
    pub message_popup: Option<bool>,
    /// Whether the reciprocal fee has already been settled.
    #[serde(default, deserialize_with = "deserialize_tristate_flag")]
    pub payment_status: Option<bool>,
    /// A card is on file and the server wants a fee confirmation.
    #[serde(default, deserialize_with = "deserialize_tristate_flag")]
    pub card_available: Option<bool>,
    /// Hosted page for adding a payment card.
    pub add_card_url: Option<String>,
}

/// What a booking response actually means. Produced by
/// [`BookSessionResponse::outcome`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BookingOutcome {
    /// The server accepted the booking. `session_id` is present when the
    /// backend returned one; callers that need certainty should confirm the
    /// session appears on the dashboard for that date.
    Accepted { session_id: Option<String> },
    /// The location charges a reciprocal fee and the server wants a payment
    /// or card-confirmation step first. Nothing was booked.
    PaymentRequired {
        message: String,
        add_card_url: Option<String>,
    },
    /// The server refused the booking. Nothing was booked.
    Rejected { message: String },
}

impl BookSessionResponse {
    /// Classify the response. See [`BookingOutcome`].
    pub fn outcome(&self) -> BookingOutcome {
        let has_error = self.error.as_deref().is_some_and(|e| !e.is_empty());
        if has_error || self.success == Some(false) {
            return BookingOutcome::Rejected {
                message: self.server_message(),
            };
        }
        // Exactly the markers the official app branches on for a
        // reciprocal-fee location. `payment_status` alone is not one: home
        // studios return it as `false` on perfectly good bookings.
        let wants_payment = self.message_popup == Some(true)
            || self.card_available == Some(true)
            || self
                .add_card_url
                .as_deref()
                .is_some_and(|u| !u.trim().is_empty());
        if wants_payment {
            return BookingOutcome::PaymentRequired {
                message: self.server_message(),
                add_card_url: self.add_card_url.clone(),
            };
        }
        let session_id = self
            .session_id
            .clone()
            .or_else(|| self.data.as_ref().and_then(|d| d.session_record_id.clone()));
        BookingOutcome::Accepted { session_id }
    }

    /// The most useful human-readable message in the response, falling back
    /// to the compact JSON of the whole body so nothing is silently dropped.
    pub fn server_message(&self) -> String {
        [
            &self.text,
            &self.description,
            &self.msg,
            &self.message,
            &self.error,
        ]
        .into_iter()
        .flatten()
        .find(|s| !s.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| serde_json::to_string(self).unwrap_or_default())
    }
}

/// IDs returned after a successful booking — needed to cancel the session
/// later via [`delete_session`](crate::HotworxClient::delete_session).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookedSessionData {
    pub session_record_id: Option<String>,
    pub lead_record_id: Option<String>,
}

/// Booking flags arrive as `"true"`, `true`, `"1"`, `"false"`, `false`, `"0"`
/// — and, since HOTWORX types them as strings, possibly values we have not
/// seen. Only the recognised spellings become `Some`; anything else is
/// `None` so an unfamiliar value can never be mistaken for a refusal.
fn deserialize_tristate_flag<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Bool(bool),
        Int(i64),
        Str(String),
    }

    Ok(match Option::<Raw>::deserialize(deserializer)? {
        Some(Raw::Bool(b)) => Some(b),
        Some(Raw::Int(i)) => Some(i != 0),
        Some(Raw::Str(s)) => match s.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        },
        None => None,
    })
}

fn deserialize_optional_i32_from_str<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i32),
    }

    match Option::<StringOrInt>::deserialize(deserializer)? {
        Some(StringOrInt::String(s)) => s.parse::<i32>().map(Some).map_err(de::Error::custom),
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        None => Ok(None),
    }
}

fn deserialize_optional_bool_from_str<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
        Int(i32),
    }

    match Option::<StringOrBool>::deserialize(deserializer)? {
        Some(StringOrBool::String(s)) => match s.to_lowercase().as_str() {
            "true" | "1" => Ok(Some(true)),
            _ => Ok(Some(false)),
        },
        Some(StringOrBool::Bool(b)) => Ok(Some(b)),
        Some(StringOrBool::Int(i)) => Ok(Some(i != 0)),
        None => Ok(None),
    }
}

fn deserialize_optional_string_from_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Int(i64),
        Float(f64),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => Ok(Some(s)),
        Some(StringOrNumber::Int(i)) => Ok(Some(i.to_string())),
        Some(StringOrNumber::Float(f)) => Ok(Some(f.to_string())),
        None => Ok(None),
    }
}
