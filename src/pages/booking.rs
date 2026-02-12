use crate::components::{BottomNav, Button, IconChevronLeft, NavItem, PageLoading};
use crate::models::booking::{SessionType, TimeSlot};
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn navigate_to(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(path);
    }
}

#[component]
pub fn BookingPage() -> impl IntoView {
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
    let success_msg: RwSignal<Option<String>> = RwSignal::new(None);

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

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "locationId": loc_id,
                "selectedDate": date
            }))
            .unwrap();

            let promise = invoke("api_get_session_types", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        log(&format!(
                            "[Booking] Got session types response: {:?}",
                            response
                        ));
                        // API returns { list: [{slot: "HOT YOGA", value: "HOT YOGA"}, ...] }
                        if let Some(types_json) = response.get("list") {
                            if let Ok(types) =
                                serde_json::from_value::<Vec<SessionType>>(types_json.clone())
                            {
                                log(&format!("[Booking] Parsed {} session types", types.len()));
                                session_types.set(types);
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Booking] Error fetching session types: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Failed to load session types: {}", err_str)));
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

            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "bookingDate": date,
                "locationId": loc_id,
                "sessionType": session_type_name
            }))
            .unwrap();

            let promise = invoke("api_show_slots", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(response) =
                        serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                    {
                        log(&format!("[Booking] Got slots response: {:?}", response));

                        // API returns slots directly as an array
                        if response.is_array() {
                            if let Ok(slots) =
                                serde_json::from_value::<Vec<TimeSlot>>(response.clone())
                            {
                                log(&format!(
                                    "[Booking] Parsed {} slots from array",
                                    slots.len()
                                ));
                                time_slots.set(slots);
                            }
                        }
                        // Or nested under data.slots
                        else if let Some(data) = response.get("data") {
                            if let Some(slots_json) = data.get("slots") {
                                if let Ok(slots) =
                                    serde_json::from_value::<Vec<TimeSlot>>(slots_json.clone())
                                {
                                    log(&format!(
                                        "[Booking] Parsed {} slots from data.slots",
                                        slots.len()
                                    ));
                                    time_slots.set(slots);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Booking] Error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    // Try to extract the actual error message from the API response
                    let display_err = if err_str.contains("\"error\":") {
                        // Parse out the error field from JSON-like string
                        if let Some(start) = err_str.find("\"error\":\"") {
                            let start = start + 9;
                            if let Some(end) = err_str[start..].find("\"") {
                                err_str[start..start + end].to_string()
                            } else {
                                err_str.clone()
                            }
                        } else {
                            err_str.clone()
                        }
                    } else {
                        err_str
                    };
                    error.set(Some(display_err));
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
            let mut failed: Vec<String> = Vec::new();
            let total = slots.len();

            for (idx, slot) in slots.into_iter().enumerate() {
                let sauna_no = slot.sauna_no.clone().unwrap_or_default();
                let time_slot = slot.time_slot.clone().unwrap_or_default();
                let session_type = slot
                    .session_name
                    .clone()
                    .unwrap_or_else(|| "Hot Yoga".to_string());

                log(&format!(
                    "[Booking] Booking {}/{}: {} at {} on {}",
                    idx + 1,
                    total,
                    session_type,
                    time_slot,
                    date
                ));

                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                    "saunaNo": sauna_no,
                    "timeSlot": time_slot,
                    "bookingDate": date,
                    "sessionType": session_type,
                    "locationId": loc_id
                }))
                .unwrap();

                let promise = invoke("api_book_session", args);

                match JsFuture::from(promise).await {
                    Ok(result) => {
                        if let Ok(response) =
                            serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                        {
                            log(&format!("[Booking] Book response: {:?}", response));
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
                        log(&format!("[Booking] Error: {:?}", e));
                        failed.push(time_slot.clone());
                    }
                }

                booking_progress.set(idx + 1);
            }

            if failed.is_empty() {
                success_msg.set(Some(format!("Successfully booked {} session(s)!", total)));
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
                    <input
                        type="date"
                        class="date-input"
                        min=get_today_date()
                        max=get_max_date()
                        prop:value=move || selected_date.get()
                        on:input=move |ev| selected_date.set(event_target_value(&ev))
                    />
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
                                                    on:click=move |_| selected_session_type.set(Some(st_for_click.clone()))
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
                                    class="set-favorite-btn"
                                    on:click=move |_| {
                                        let type_value = session_type_value.clone();
                                        let type_display = session_type_display.clone();
                                        let loc = loc_id.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            // Get location name for storing
                                            let loc_args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
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
                                                let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                                    "locationId": loc,
                                                    "locationName": loc_name
                                                })).unwrap();
                                                let _ = JsFuture::from(invoke("store_preferred_location", args)).await;
                                            }

                                            // Store preferred session type
                                            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                                "sessionType": type_value,
                                                "sessionTypeDisplay": type_display
                                            })).unwrap();
                                            match JsFuture::from(invoke("store_preferred_session_type", args)).await {
                                                Ok(_) => log("[Booking] Saved favorite session type"),
                                                Err(e) => log(&format!("[Booking] Failed to save favorite: {:?}", e)),
                                            }
                                        });
                                    }
                                >
                                    "Set as Favorite for Quick Book"
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

/// Get today's date in YYYY-MM-DD format
fn get_today_date() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1; // 0-indexed
    let day = now.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// Get max booking date (today + 2 days) in YYYY-MM-DD format
fn get_max_date() -> String {
    let now = js_sys::Date::new_0();
    // Add 2 days (API allows within 3 days including today)
    now.set_date(now.get_date() + 2);
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}
