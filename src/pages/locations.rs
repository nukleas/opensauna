use crate::components::toast::use_toast;
use crate::components::{BottomNav, IconSearch, NavItem, PageLoading};
use crate::models::location::{Location, LocationsResponse};
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::nav::go as navigate_to;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Studio location picker — lists available and frequent locations, then navigates to booking.
#[component]
pub fn LocationsPage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let locations: RwSignal<Vec<Location>> = RwSignal::new(Vec::new());
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let search_query = RwSignal::new(String::new());
    let preferred_location: RwSignal<Option<(String, String)>> = RwSignal::new(None);
    let show_all_locations = RwSignal::new(false);

    // Fetch preferred location and all locations on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            // First check for saved preferred location
            log("[Locations] Checking for preferred location...");
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
            let promise = invoke("get_preferred_location", args);

            if let Ok(result) = JsFuture::from(promise).await {
                if let Ok(Some((id, name))) =
                    serde_wasm_bindgen::from_value::<Option<(String, String)>>(result)
                {
                    log(&format!(
                        "[Locations] Found preferred location: {} ({})",
                        name, id
                    ));
                    preferred_location.set(Some((id, name)));
                }
            }

            // Then fetch all locations
            log("[Locations] Fetching locations...");
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
            let promise = invoke("api_get_locations", args);

            match JsFuture::from(promise).await {
                Ok(result) => match serde_wasm_bindgen::from_value::<LocationsResponse>(result) {
                    Ok(resp) => {
                        let locs = resp.data.and_then(|d| d.locations).unwrap_or_default();
                        log(&format!("[Locations] Parsed {} locations", locs.len()));
                        locations.set(locs);
                    }
                    Err(e) => {
                        log(&format!("[Locations] Parse error: {:?}", e));
                        error.set(Some("Failed to load locations".to_string()));
                    }
                },
                Err(e) => {
                    log(&format!("[Locations] Error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        loading.set(false);
                        return;
                    }
                    error.set(Some("Failed to load locations".to_string()));
                }
            }

            loading.set(false);
        });
    });

    let filtered_locations = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            locations.get()
        } else {
            locations
                .get()
                .into_iter()
                .filter(|loc| loc.location_name.to_lowercase().contains(&query))
                .collect()
        }
    };

    let on_select_location = move |location_id: String, location_name: String| {
        // Save as preferred location
        let id = location_id.clone();
        let name = location_name.clone();
        wasm_bindgen_futures::spawn_local(async move {
            log(&format!("[Locations] Saving preferred location: {}", name));
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "locationId": id,
                "locationName": name
            }))
            .unwrap();
            let _ = JsFuture::from(invoke("store_preferred_location", args)).await;
        });

        navigate_to(&format!("/book/{}", location_id));
    };

    let on_use_preferred = move || {
        if let Some((id, _)) = preferred_location.get() {
            navigate_to(&format!("/book/{}", id));
        }
    };

    let on_change_location = move || {
        show_all_locations.set(true);
    };

    view! {
        <div class="locations-page">
            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="locations-header">
                <h1 class="page-title">"Select Location"</h1>
            </div>

            // Show preferred location card if set and not showing all
            {move || {
                let pref = preferred_location.get();
                let showing_all = show_all_locations.get();

                if let (Some((_, name)), false) = (pref, showing_all) {
                    view! {
                        <div class="preferred-location-section">
                            <div class="preferred-card">
                                <div class="preferred-label">"Your Location"</div>
                                <h3 class="preferred-name">{name}</h3>
                                <div class="preferred-actions">
                                    <button
                                        class="button button-primary"
                                        on:click=move |_| on_use_preferred()
                                    >
                                        "Book Here"
                                    </button>
                                    <button
                                        class="button button-secondary"
                                        on:click=move |_| on_change_location()
                                    >
                                        "Change"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="locations-search">
                            <span class="search-icon">
                                <IconSearch size=crate::components::icons::IconSize::Sm />
                            </span>
                            <input
                                type="search"
                                class="search-input"
                                placeholder="Search locations..."
                                prop:value=move || search_query.get()
                                on:input=move |ev| search_query.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="locations-list">
                            {move || {
                                let locs = filtered_locations();
                                if locs.is_empty() && !loading.get() {
                                    view! {
                                        <div class="empty-state">
                                            <p>"No locations found"</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="location-grid">
                                            {locs.into_iter().map(|loc| {
                                                let loc_id = loc.location_id.clone();
                                                let loc_name = loc.location_name.clone();
                                                let on_click = on_select_location;
                                                view! {
                                                    <button
                                                        class="location-card"
                                                        on:click=move |_| on_click(loc_id.clone(), loc_name.clone())
                                                    >
                                                        <h3 class="location-name">{loc.location_name.clone()}</h3>
                                                        {loc.description.clone().filter(|d| !d.is_empty()).map(|d| view! {
                                                            <p class="location-desc">{d}</p>
                                                        })}
                                                        {loc.location_tier.clone().map(|t| view! {
                                                            <span class="location-tier">{t}</span>
                                                        })}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    }.into_any()
                }
            }}

            {move || error.get().map(|e| view! {
                <div class="error-message">{e}</div>
            })}

            <BottomNav active=Signal::derive(|| NavItem::Book) />
        </div>
    }
}
