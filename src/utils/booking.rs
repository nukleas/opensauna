//! Shared booking-loop helper used by the regular booking page and Quick Book.
//!
//! Both flows iterate over a list of selected time slots, post each one
//! through `api_book_session`, track progress, and collect per-slot
//! failures. The mechanics are identical; only how the caller resolves
//! the session-type name differs (the regular page uses the slot's
//! `session_name`; Quick Book uses the saved preference).
//!
//! HOTWORX answers every booking request with HTTP 200, booked or not, so
//! two checks stand between the request and a "Booked!" toast: the backend
//! classifies the body (`BookingResult`), and after the loop this helper
//! re-reads the dashboard for the date and insists each accepted slot shows
//! up as a new pending session.

use std::collections::HashSet;

use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::components::toast::ToastState;
use crate::models::api::ApiEnvelope;
use crate::models::booking::{BookingResult, TimeSlot};
use crate::models::dashboard::DashboardData;
use crate::state::{handle_invoke_error, AuthState};
use crate::utils::tauri::{invoke, log};

/// A single slot the user has chosen to book, paired with the resolved
/// session-type name to send to HOTWORX.
pub struct BookableSlot {
    pub slot: TimeSlot,
    pub session_type: String,
}

/// Outcome of a [`book_slots`] run. The `total` count plus the human-
/// readable `failures` list is everything either page needs to render its
/// success/error message.
pub struct BookingOutcome {
    pub total: usize,
    pub failures: Vec<String>,
    /// False when the post-booking dashboard check could not run (the
    /// dashboard fetch itself failed). Bookings are then reported as the
    /// server accepted them, without independent confirmation.
    pub verified: bool,
}

impl BookingOutcome {
    pub fn all_succeeded(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn success_count(&self) -> usize {
        self.total.saturating_sub(self.failures.len())
    }
}

/// Session record ids pending on the dashboard for `date`, or `None` if the
/// dashboard could not be fetched.
async fn pending_session_ids(date: &str) -> Option<HashSet<String>> {
    let args = crate::json_args!({ "currentDate": date });
    let result = JsFuture::from(invoke("api_get_dashboard", args)).await.ok()?;
    let env = serde_wasm_bindgen::from_value::<ApiEnvelope<DashboardData>>(result).ok()?;
    let sessions = env.data?.todays_pending_sessions.unwrap_or_default();
    Some(
        sessions
            .into_iter()
            .filter_map(|s| s.session_record_id)
            .collect(),
    )
}

/// Book every slot in `slots` sequentially, updating `progress` after each
/// attempt. On auth-expiry the loop stops early and returns the outcome
/// with whatever ran so far (the page will redirect to `/login` via the
/// existing `<Show>` guards).
pub async fn book_slots(
    auth: AuthState,
    toast: ToastState,
    slots: Vec<BookableSlot>,
    location_id: String,
    date: String,
    progress: RwSignal<usize>,
) -> BookingOutcome {
    let total = slots.len();
    let mut failures: Vec<String> = Vec::new();
    // Slots the server accepted, with the session id it returned (if any).
    let mut accepted: Vec<(String, Option<String>)> = Vec::new();

    let before = pending_session_ids(&date).await;

    for (idx, BookableSlot { slot, session_type }) in slots.into_iter().enumerate() {
        let sauna_no = slot.sauna_no.clone().unwrap_or_default();
        let time_slot = slot.time_slot.clone().unwrap_or_default();

        log(&format!(
            "[Booking] {}/{}: {} at {} on {}",
            idx + 1,
            total,
            session_type,
            time_slot,
            date
        ));

        let args = crate::json_args!({
            "saunaNo": sauna_no,
            "timeSlot": time_slot,
            "bookingDate": date,
            "sessionType": session_type,
            "locationId": location_id,
        });

        match JsFuture::from(invoke("api_book_session", args)).await {
            Ok(result) => match serde_wasm_bindgen::from_value::<BookingResult>(result) {
                Ok(BookingResult::Accepted { session_id }) => {
                    accepted.push((time_slot, session_id));
                }
                Ok(BookingResult::PaymentRequired { message, .. }) => {
                    failures.push(format!(
                        "{time_slot}: not booked, this location needs a payment step first ({message}). Finish it in the HOTWORX app."
                    ));
                }
                Ok(BookingResult::Rejected { message }) => {
                    failures.push(format!("{time_slot}: {message}"));
                }
                Err(e) => {
                    log(&format!("[Booking] Unreadable booking result: {:?}", e));
                    failures.push(format!("{time_slot}: unreadable response from server"));
                }
            },
            Err(e) => {
                log(&format!("[Booking] Error: {:?}", e));
                if handle_invoke_error(&e, auth, toast).await {
                    return BookingOutcome {
                        total,
                        failures,
                        verified: false,
                    };
                }
                failures.push(format!(
                    "{time_slot}: {}",
                    crate::state::invoke_error_string(&e)
                ));
            }
        }

        progress.set(idx + 1);
    }

    if accepted.is_empty() {
        return BookingOutcome {
            total,
            failures,
            verified: true,
        };
    }

    // Independent confirmation: every accepted slot must now be a pending
    // session that was not there before we started.
    let (Some(before), Some(after)) = (before, pending_session_ids(&date).await) else {
        log("[Booking] Could not fetch dashboard to verify bookings");
        return BookingOutcome {
            total,
            failures,
            verified: false,
        };
    };
    let mut new_ids: HashSet<String> = after.difference(&before).cloned().collect();
    log(&format!(
        "[Booking] Verification: {} accepted, {} new pending sessions",
        accepted.len(),
        new_ids.len()
    ));

    // A returned id that matches a new pending session is an exact
    // confirmation. The booking and dashboard ids come from different
    // backend models and may not share a format, so anything unmatched
    // falls back to the floor check: one new pending session per accepted
    // slot.
    let mut unmatched: Vec<String> = Vec::new();
    for (time_slot, id) in accepted {
        match id {
            Some(id) if new_ids.remove(&id) => {}
            _ => unmatched.push(time_slot),
        }
    }
    let mut spare = new_ids.into_iter();
    for time_slot in unmatched {
        if spare.next().is_none() {
            failures.push(format!("{time_slot}: server did not confirm the booking"));
        }
    }

    BookingOutcome {
        total,
        failures,
        verified: true,
    }
}
