use crate::components::toast::use_toast;
use crate::components::{BottomNav, Button, IconChevronLeft, NavItem, PageLoading};
use crate::models::booking::{SessionType, TimeSlot};
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::booking::{book_slots, BookableSlot};
use crate::utils::dates::{bookable_days, today as get_today_date};
use crate::utils::nav::go as navigate_to;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;

/// `api_get_session_types` returns `{ list: [SessionType] }`.
#[derive(Debug, Clone, Deserialize)]
struct SessionTypesResponse {
    #[serde(default)]
    list: Vec<SessionType>,
}

/// Session booking flow: date picker, session type selector, and time slot grid.
#[component]
pub fn BookingPage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let params = use_params_map();

    let location_id = move || params.get().get("location_id").unwrap_or_default();

    let session_types: RwSignal<Vec<SessionType>> = RwSignal::new(Vec::new());
    let selected_session_type: RwSignal<Option<SessionType>> = RwSignal::new(None);
    let time_slots: RwSignal<Vec<TimeSlot>> = RwSignal::new(Vec::new());
    let selected_date = RwSignal::new(get_today_date());
    let selected_slots: RwSignal<Vec<TimeSlot>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let session_types_loading = RwSignal::new(false);
    let booking_loading = RwSignal::new(false);
    let booking_progress = RwSignal::new(0usize);
    let booking_total = RwSignal::new(0usize);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    // Whether the current session type is saved as the Quick Book favorite.
    let favorited = RwSignal::new(false);

    // Fetch session types when date changes
    Effect::new(move |_| {
        let date = selected_date.get();
        let loc_id = location_id();

        if loc_id.is_empty() {
            loading.set(false);
            return;
        }

        // Reset selections when date changes
        selected_session_type.set(None);
        selected_slots.set(Vec::new());
        time_slots.set(Vec::new());
        session_types_loading.set(true);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            log(&format!(
                "[Booking] Fetching session types for {} on {}",
                loc_id, date
            ));

            let args = crate::json_args!({
                "locationId": loc_id,
                "selectedDate": date,
            });

            let promise = invoke("api_get_session_types", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(resp) = serde_wasm_bindgen::from_value::<SessionTypesResponse>(result)
                    {
                        log(&format!(
                            "[Booking] Parsed {} session types",
                            resp.list.len()
                        ));
                        session_types.set(resp.list);
                    }
                }
                Err(e) => {
                    log(&format!("[Booking] Error fetching session types: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        session_types_loading.set(false);
                        loading.set(false);
                        return;
                    }
                    error.set(Some("Failed to load session types".to_string()));
                }
            }

            session_types_loading.set(false);
            loading.set(false);
        });
    });

    // Fetch time slots when session type is selected
    Effect::new(move |_| {
        let session_type = selected_session_type.get();
        let date = selected_date.get();
        let loc_id = location_id();

        if loc_id.is_empty() || session_type.is_none() {
            return;
        }

        let session_type = session_type.unwrap();
        let session_type_name = session_type.value.clone().unwrap_or_default();

        if session_type_name.is_empty() {
            return;
        }

        // Reset slot selection
        selected_slots.set(Vec::new());
        loading.set(true);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            log(&format!(
                "[Booking] Fetching slots for {} - {} on {}",
                loc_id, session_type_name, date
            ));

            let args = crate::json_args!({
                "bookingDate": date,
                "locationId": loc_id,
                "sessionType": session_type_name,
            });

            let promise = invoke("api_show_slots", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(slots) = serde_wasm_bindgen::from_value::<Vec<TimeSlot>>(result) {
                        log(&format!("[Booking] Parsed {} slots", slots.len()));
                        time_slots.set(slots);
                    }
                }
                Err(e) => {
                    log(&format!("[Booking] Error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        loading.set(false);
                        return;
                    }
                    error.set(Some("Failed to load time slots".to_string()));
                }
            }

            loading.set(false);
        });
    });

    let on_book = move || {
        let slots = selected_slots.get();
        let date = selected_date.get();
        let loc_id = location_id();

        if slots.is_empty() {
            error.set(Some("Please select at least one time slot".to_string()));
            return;
        }

        booking_loading.set(true);
        booking_total.set(slots.len());
        booking_progress.set(0);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            // Each slot resolves its own session-type — fall back to "Hot
            // Yoga" if the slot is missing the field (older API rows).
            let bookables: Vec<BookableSlot> = slots
                .into_iter()
                .map(|slot| {
                    let session_type = slot
                        .session_name
                        .clone()
                        .unwrap_or_else(|| "Hot Yoga".to_string());
                    BookableSlot { slot, session_type }
                })
                .collect();

            let outcome = book_slots(auth, toast, bookables, loc_id, date, booking_progress).await;

            if outcome.all_succeeded() {
                toast.success(format!("Booked {} session(s)!", outcome.total));
                navigate_to("/");
            } else {
                error.set(Some(format!(
                    "Booked {} of {} sessions. Failed: {}",
                    outcome.success_count(),
                    outcome.total,
                    outcome.failures.join(", ")
                )));
            }

            booking_loading.set(false);
        });
    };

    let on_back = move || {
        navigate_to("/book");
    };

    view! {
        <div class="booking-page">
            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="booking-header">
                <button class="back-button" on:click=move |_| on_back()>
                    <IconChevronLeft size=crate::components::icons::IconSize::Sm />
                    "Back"
                </button>
                <h1 class="page-title">"Book Session"</h1>
            </div>

            <div class="booking-content">
                // Date picker - restricted to today + 2 days (API allows within 3 days)
                <div class="date-picker">
                    <label>"Select Date"</label>
                    <div class="date-pills">
                        {bookable_days().into_iter().map(|(ymd, label)| {
                            let ymd_sel = ymd.clone();
                            view! {
                                <button
                                    class=move || if selected_date.get() == ymd_sel { "date-pill active" } else { "date-pill" }
                                    on:click=move |_| selected_date.set(ymd.clone())
                                >
                                    {label}
                                </button>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>

                // Session type selector
                <div class="session-types-section">
                    <h2>"Select Session Type"</h2>
                    {move || {
                        if session_types_loading.get() {
                            view! {
                                <div class="loading-indicator">"Loading session types..."</div>
                            }.into_any()
                        } else {
                            let types = session_types.get();
                            if types.is_empty() {
                                view! {
                                    <div class="empty-state">
                                        <p>"No session types available for this date"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="session-types-grid">
                                        {types.into_iter().map(|st| {
                                            let st_clone = st.clone();
                                            let is_selected = move || {
                                                selected_session_type.get().as_ref() == Some(&st)
                                            };
                                            let st_for_click = st_clone.clone();
                                            let display_name = st_clone.slot.clone()
                                                .unwrap_or_else(|| "Unknown".to_string());
                                            view! {
                                                <button
                                                    class=move || if is_selected() { "session-type-card selected" } else { "session-type-card" }
                                                    on:click=move |_| {
                                                        selected_session_type.set(Some(st_for_click.clone()));
                                                        favorited.set(false);
                                                    }
                                                >
                                                    <span class="session-type-name">{display_name}</span>
                                                </button>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                    // Set as Favorite button (only show when session type is selected)
                    {move || {
                        selected_session_type.get().map(|st| {
                            let session_type_value = st.value.clone().unwrap_or_default();
                            let session_type_display = st.slot.clone().unwrap_or_default();
                            let loc_id = location_id();
                            view! {
                                <button
                                    class=move || if favorited.get() { "set-favorite-btn favorited" } else { "set-favorite-btn" }
                                    disabled=move || favorited.get()
                                    on:click=move |_| {
                                        let type_value = session_type_value.clone();
                                        let type_display = session_type_display.clone();
                                        let loc = loc_id.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            // Get location name for storing
                                            let loc_args = crate::json_args!({});
                                            let loc_name = match JsFuture::from(invoke("get_preferred_location", loc_args)).await {
                                                Ok(result) => {
                                                    serde_wasm_bindgen::from_value::<Option<(String, String)>>(result)
                                                        .ok()
                                                        .flatten()
                                                        .map(|(_, name)| name)
                                                        .unwrap_or_default()
                                                }
                                                Err(_) => String::new(),
                                            };

                                            // Store preferred location if we have it
                                            if !loc.is_empty() && !loc_name.is_empty() {
                                                let args = crate::json_args!({
                                                    "locationId": loc,
                                                    "locationName": loc_name,
                                                });
                                                let _ = JsFuture::from(invoke("store_preferred_location", args)).await;
                                            }

                                            // Store preferred session type
                                            let args = crate::json_args!({
                                                "sessionType": type_value,
                                                "sessionTypeDisplay": type_display,
                                            });
                                            match JsFuture::from(invoke("store_preferred_session_type", args)).await {
                                                Ok(_) => {
                                                    log("[Booking] Saved favorite session type");
                                                    favorited.set(true);
                                                    toast.success("Saved to Quick Book ★");
                                                }
                                                Err(e) => {
                                                    log(&format!("[Booking] Failed to save favorite: {:?}", e));
                                                    toast.error("Couldn't save favorite — try again");
                                                }
                                            }
                                        });
                                    }
                                >
                                    {move || if favorited.get() {
                                        "★ Saved to Quick Book"
                                    } else {
                                        "Set as Favorite for Quick Book"
                                    }}
                                </button>
                            }
                        })
                    }}
                </div>

                // Time slots (only show after session type is selected)
                {move || {
                    if selected_session_type.get().is_some() {
                        view! {
                            <div class="time-slots-section">
                                <h2>"Select Time Slots"</h2>
                                <p class="slots-hint">"Tap multiple slots for back-to-back sessions"</p>
                                {move || {
                                    let slots = time_slots.get();
                                    if loading.get() {
                                        view! {
                                            <div class="loading-indicator">"Loading time slots..."</div>
                                        }.into_any()
                                    } else if slots.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p>"No available time slots"</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="time-slots-grid">
                                                {slots.into_iter().map(|slot| {
                                                    let slot_clone = slot.clone();
                                                    let is_selected = move || {
                                                        selected_slots.get().iter().any(|s| s == &slot)
                                                    };
                                                    let slot_for_click = slot_clone.clone();

                                                    // Count available spots (slot1/2/3 contain "available" when open)
                                                    let is_available = |s: &Option<String>| {
                                                        s.as_ref().map(|v| v.contains("available")).unwrap_or(false)
                                                    };
                                                    let spot1_open = is_available(&slot_clone.slot1);
                                                    let spot2_open = is_available(&slot_clone.slot2);
                                                    let spot3_open = is_available(&slot_clone.slot3);
                                                    let available_count = [spot1_open, spot2_open, spot3_open].iter().filter(|&&x| x).count();

                                                    view! {
                                                        <button
                                                            class=move || if is_selected() { "time-slot selected" } else { "time-slot" }
                                                            on:click=move |_| {
                                                                selected_slots.update(|slots| {
                                                                    let s = slot_for_click.clone();
                                                                    if let Some(idx) = slots.iter().position(|x| x == &s) {
                                                                        slots.remove(idx);
                                                                    } else {
                                                                        slots.push(s);
                                                                        // Sort by time
                                                                        slots.sort_by(|a, b| {
                                                                            a.time_slot.as_ref().unwrap_or(&String::new())
                                                                                .cmp(b.time_slot.as_ref().unwrap_or(&String::new()))
                                                                        });
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <span class="slot-time">
                                                                {slot_clone.time_slot.clone().unwrap_or_else(|| "N/A".to_string())}
                                                            </span>
                                                            <div class="slot-availability">
                                                                <span class=if spot1_open { "spot open" } else { "spot taken" }></span>
                                                                <span class=if spot2_open { "spot open" } else { "spot taken" }></span>
                                                                <span class=if spot3_open { "spot open" } else { "spot taken" }></span>
                                                                <span class="availability-text">{format!("{}/3", available_count)}</span>
                                                            </div>
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="time-slots-section">
                                <h2>"Select Time Slots"</h2>
                                <div class="empty-state">
                                    <p>"Select a session type to see available time slots"</p>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}

                // Booking summary (when multiple selected)
                {move || {
                    let slots = selected_slots.get();
                    (slots.len() > 1).then(|| view! {
                        <div class="booking-summary">
                            <h3>{format!("{} sessions selected", slots.len())}</h3>
                            <ul class="selected-times">
                                {slots.iter().map(|s| {
                                    let time = s.time_slot.clone().unwrap_or_default();
                                    view! { <li>{time}</li> }
                                }).collect::<Vec<_>>()}
                            </ul>
                        </div>
                    })
                }}

                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}

                // Booking progress
                {move || {
                    booking_loading.get().then(|| {
                        let p = booking_progress.get();
                        let t = booking_total.get();
                        view! {
                            <div class="booking-progress">
                                <div class="progress-bar">
                                    <div
                                        class="progress-fill"
                                        style=move || format!("width: {}%", (p * 100).checked_div(t).unwrap_or(0))
                                    />
                                </div>
                                <span class="progress-text">{format!("Booking {} of {}...", p + 1, t)}</span>
                            </div>
                        }
                    })
                }}

                // Book button
                <div class="book-action">
                    {move || {
                        let count = selected_slots.get().len();
                        let label = if count == 0 {
                            "Select Time Slots".to_string()
                        } else if count == 1 {
                            "Confirm Booking".to_string()
                        } else {
                            format!("Book {} Sessions", count)
                        };
                        view! {
                            <Button
                                label=label
                                loading=Signal::derive(move || booking_loading.get())
                                disabled=Signal::derive(move || selected_slots.get().is_empty())
                                on_click=on_book
                            />
                        }
                    }}
                </div>
            </div>

            <BottomNav active=Signal::derive(|| NavItem::Book) />
        </div>
    }
}
