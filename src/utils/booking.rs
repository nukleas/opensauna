//! Shared booking-loop helper used by the regular booking page and Quick Book.
//!
//! Both flows iterate over a list of selected time slots, post each one
//! through `api_book_session`, track progress, and collect per-slot
//! failures. The mechanics are identical; only how the caller resolves
//! the session-type name differs (the regular page uses the slot's
//! `session_name`; Quick Book uses the saved preference).

use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::components::toast::ToastState;
use crate::models::booking::{BookSessionResponse, TimeSlot};
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
}

impl BookingOutcome {
    pub fn all_succeeded(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn success_count(&self) -> usize {
        self.total.saturating_sub(self.failures.len())
    }
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
            Ok(result) => {
                if let Ok(resp) = serde_wasm_bindgen::from_value::<BookSessionResponse>(result) {
                    if let Some(err) = resp.error.filter(|e| !e.is_empty()) {
                        failures.push(format!("{}: {}", time_slot, err));
                    }
                }
            }
            Err(e) => {
                log(&format!("[Booking] Error: {:?}", e));
                if handle_invoke_error(&e, auth, toast).await {
                    return BookingOutcome { total, failures };
                }
                failures.push(time_slot.clone());
            }
        }

        progress.set(idx + 1);
    }

    BookingOutcome { total, failures }
}
