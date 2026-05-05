use crate::components::toast::use_toast;
use crate::components::{BottomNav, Button, IconChevronLeft, NavItem, PageLoading};
use crate::models::booking::TimeSlot;
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::dates::{max_booking_date as get_max_date, today as get_today_date};
use crate::utils::nav::go as navigate_to;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// One-tap rebooking using saved location and session type preferences.
#[component]
pub fn QuickBookPage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    // Preferences
    let location: RwSignal<Option<(String, String)>> = RwSignal::new(None);
    let session_type: RwSignal<Option<(String, String)>> = RwSignal::new(None);

    // Booking state
    let time_slots: RwSignal<Vec<TimeSlot>> = RwSignal::new(Vec::new());
    let selected_date = RwSignal::new(get_today_date());
    let selected_slots: RwSignal<Vec<TimeSlot>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let slots_loading = RwSignal::new(false);
    let booking_loading = RwSignal::new(false);
    let booking_progress = RwSignal::new(0usize);
    let booking_total = RwSignal::new(0usize);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let missing_prefs = RwSignal::new(false);

    // Load preferences on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[QuickBook] Loading preferences...");

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();

            // Get preferred location
            let loc_promise = invoke("get_preferred_location", args.clone());
            let type_promise = invoke("get_preferred_session_type", args);

            let loc_result = JsFuture::from(loc_promise).await;
            let type_result = JsFuture::from(type_promise).await;

            let loc_parsed = loc_result.ok().and_then(|v| {
                serde_wasm_bindgen::from_value::<Option<(String, String)>>(v)
                    .ok()
                    .flatten()
            });

            let type_parsed = type_result.ok().and_then(|v| {
                serde_wasm_bindgen::from_value::<Option<(String, String)>>(v)
                    .ok()
                    .flatten()
            });

            if loc_parsed.is_none() || type_parsed.is_none() {
                log("[QuickBook] Missing preferences");
                missing_prefs.set(true);
                loading.set(false);
                return;
            }

            log(&format!(
                "[QuickBook] Loaded: loc={:?}, type={:?}",
                loc_parsed, type_parsed
            ));
            location.set(loc_parsed);
            session_type.set(type_parsed);
            loading.set(false);
        });
    });

    // Fetch slots when date changes (and prefs are loaded)
    Effect::new(move |_| {
        let date = selected_date.get();
        let loc = location.get();
        let typ = session_type.get();

        if loc.is_none() || typ.is_none() {
            return;
        }

        let (loc_id, _loc_name) = loc.unwrap();
        let (type_value, _type_display) = typ.unwrap();

        // Reset selection
        selected_slots.set(Vec::new());
        slots_loading.set(true);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            log(&format!(
                "[QuickBook] Fetching slots for {} - {} on {}",
                loc_id, type_value, date
            ));

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "bookingDate": date,
                "locationId": loc_id,
                "sessionType": type_value
            }))
            .unwrap();

            let promise = invoke("api_show_slots", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        // API returns slots directly as an array
                        if response.is_array() {
                            if let Ok(slots) =
                                serde_json::from_value::<Vec<TimeSlot>>(response.clone())
                            {
                                log(&format!("[QuickBook] Got {} slots", slots.len()));
                                time_slots.set(slots);
                            }
                        }
                        // Or nested under data.slots
                        else if let Some(data) = response.get("data") {
                            if let Some(slots_json) = data.get("slots") {
                                if let Ok(slots) =
                                    serde_json::from_value::<Vec<TimeSlot>>(slots_json.clone())
                                {
                                    time_slots.set(slots);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[QuickBook] Slots error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        slots_loading.set(false);
                        return;
                    }
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Failed to load slots: {}", err_str)));
                }
            }

            slots_loading.set(false);
        });
    });

    let on_book = move || {
        let slots = selected_slots.get();
        let date = selected_date.get();
        let loc = location.get();
        let typ = session_type.get();

        if slots.is_empty() {
            error.set(Some("Please select at least one time slot".to_string()));
            return;
        }

        if loc.is_none() || typ.is_none() {
            error.set(Some("Missing preferences".to_string()));
            return;
        }

        let (loc_id, _) = loc.unwrap();
        let (session_type_value, _) = typ.unwrap();

        booking_loading.set(true);
        booking_total.set(slots.len());
        booking_progress.set(0);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let mut failed: Vec<String> = Vec::new();
            let total = slots.len();

            for (idx, slot) in slots.into_iter().enumerate() {
                let sauna_no = slot.sauna_no.clone().unwrap_or_default();
                let time_slot = slot.time_slot.clone().unwrap_or_default();

                log(&format!(
                    "[QuickBook] Booking {}/{}: {} at {}",
                    idx + 1,
                    total,
                    session_type_value,
                    time_slot
                ));

                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "saunaNo": sauna_no,
                    "timeSlot": time_slot,
                    "bookingDate": date,
                    "sessionType": session_type_value,
                    "locationId": loc_id
                }))
                .unwrap();

                let promise = invoke("api_book_session", args);

                match JsFuture::from(promise).await {
                    Ok(result) => {
                        if let Ok(response) =
                            serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                        {
                            if response.get("error").is_some()
                                && !response.get("error").unwrap().is_null()
                            {
                                let err = response
                                    .get("error")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or("Unknown error");
                                failed.push(format!("{}: {}", time_slot, err));
                            }
                        }
                    }
                    Err(e) => {
                        if handle_invoke_error(&e, auth, toast).await {
                            booking_loading.set(false);
                            return;
                        }
                        failed.push(time_slot.clone());
                    }
                }

                booking_progress.set(idx + 1);
            }

            if failed.is_empty() {
                log(&format!(
                    "[QuickBook] Successfully booked {} sessions!",
                    total
                ));
                navigate_to("/");
            } else {
                let success_count = total - failed.len();
                error.set(Some(format!(
                    "Booked {} of {} sessions. Failed: {}",
                    success_count,
                    total,
                    failed.join(", ")
                )));
            }

            booking_loading.set(false);
        });
    };

    let on_back = move || {
        navigate_to("/");
    };

    view! {
        <div class="quick-book-page">
            {move || (loading.get() || slots_loading.get()).then(|| view! { <PageLoading /> })}

            <div class="booking-header">
                <button class="back-button" on:click=move |_| on_back()>
                    <IconChevronLeft size=crate::components::icons::IconSize::Sm />
                    "Back"
                </button>
                <h1 class="page-title">"Quick Book"</h1>
            </div>

            // Show missing preferences message
            {move || {
                missing_prefs.get().then(|| view! {
                    <div class="booking-content">
                        <div class="empty-state">
                            <p>"Set up your preferences first!"</p>
                            <p style="margin-top: 1rem; font-size: 0.875rem;">
                                "Go to Book a Session and select your preferred location and session type, then tap 'Set as Favorite'."
                            </p>
                            <div style="margin-top: 1.5rem;">
                                <Button
                                    label="Go to Booking"
                                    on_click=move || navigate_to("/book")
                                />
                            </div>
                        </div>
                    </div>
                })
            }}

            // Main booking UI (only when prefs are loaded)
            {move || {
                let has_prefs = location.get().is_some() && session_type.get().is_some();
                has_prefs.then(|| {
                    let (_loc_id, loc_name) = location.get().unwrap();
                    let (_type_value, type_display) = session_type.get().unwrap();

                    view! {
                        <div class="booking-content">
                            // Show what we're booking
                            <div class="quick-book-info">
                                <div class="info-row">
                                    <span class="info-label">"Session"</span>
                                    <span class="info-value">{type_display}</span>
                                </div>
                                <div class="info-row">
                                    <span class="info-label">"Location"</span>
                                    <span class="info-value">{loc_name}</span>
                                </div>
                            </div>

                            // Date picker
                            <div class="date-picker">
                                <label>"Select Date"</label>
                                <input
                                    type="date"
                                    class="date-input"
                                    min=get_today_date()
                                    max=get_max_date()
                                    prop:value=move || selected_date.get()
                                    on:input=move |ev| selected_date.set(event_target_value(&ev))
                                />
                            </div>

                            // Time slots (multi-select)
                            <div class="time-slots-section">
                                <h2>"Select Time Slots"</h2>
                                <p class="slots-hint">"Tap multiple slots for back-to-back sessions"</p>
                                {move || {
                                    let slots = time_slots.get();
                                    if slots.is_empty() && !slots_loading.get() {
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

                                                    // Count available spots
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
                                                    style=move || format!("width: {}%", if t > 0 { p * 100 / t } else { 0 })
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
                                        "Book Session".to_string()
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
                    }
                })
            }}

            <BottomNav active=Signal::derive(|| NavItem::Home) />
        </div>
    }
}
