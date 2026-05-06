use crate::components::toast::use_toast;
use crate::components::{BottomNav, Button, NavItem, PageLoading, TextInput};
use crate::models::api::{ApiEnvelope, NestedDataEnvelope};
use crate::models::profile::{CalorieStatsData, GoalsData, ProfileData};
use crate::state::{handle_invoke_error, use_auth_state};
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Profile page with user info, calorie stats, goals management, and logout.
#[component]
pub fn ProfilePage() -> impl IntoView {
    let auth = use_auth_state();
    let toast = use_toast();
    let profile: RwSignal<Option<ProfileData>> = RwSignal::new(None);
    let calorie_stats: RwSignal<Option<CalorieStatsData>> = RwSignal::new(None);
    let goals: RwSignal<Option<GoalsData>> = RwSignal::new(None);
    let loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let saving = RwSignal::new(false);
    let save_message: RwSignal<Option<String>> = RwSignal::new(None);

    // Edit mode states
    let editing_profile = RwSignal::new(false);
    let editing_goals = RwSignal::new(false);

    // Profile edit form fields
    let edit_first_name = RwSignal::new(String::new());
    let edit_last_name = RwSignal::new(String::new());
    let edit_dob = RwSignal::new(String::new());
    let edit_gender = RwSignal::new(String::new());
    let edit_height = RwSignal::new(String::new());
    let edit_weight = RwSignal::new(String::new());
    let edit_address = RwSignal::new(String::new());

    // Goals edit form fields
    let edit_current_weight = RwSignal::new(String::new());
    let edit_target_weight = RwSignal::new(String::new());
    let edit_goal_date = RwSignal::new(String::new());
    let edit_weekly_sessions = RwSignal::new(String::new());

    // Fetch profile data
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            log("[Profile] Fetching profile data...");

            let args = crate::json_args!({});
            let promise = invoke("api_view_profile", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    // The HOTWORX `view_profile` endpoint double-wraps the
                    // payload as `{ data: [{ data: ProfileData }] }`. The
                    // backend forwards that shape verbatim.
                    match serde_wasm_bindgen::from_value::<NestedDataEnvelope<ProfileData>>(result)
                    {
                        Ok(env) => {
                            if let Some(p) = env.first() {
                                log(&format!("[Profile] Parsed profile: {}", p.display_name()));
                                profile.set(Some(p));
                            } else {
                                log("[Profile] Empty profile data");
                                error.set(Some("Failed to load profile".to_string()));
                            }
                        }
                        Err(e) => {
                            log(&format!("[Profile] Deserialize error: {:?}", e));
                            error.set(Some("Failed to load profile".to_string()));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Profile] Error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        loading.set(false);
                        return;
                    }
                    error.set(Some("Failed to load profile".to_string()));
                }
            }

            loading.set(false);
        });
    });

    // Fetch calorie stats
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let args = crate::json_args!({});
            let promise = invoke("api_get_calorie_stats", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(env) =
                        serde_wasm_bindgen::from_value::<ApiEnvelope<CalorieStatsData>>(result)
                    {
                        if let Some(stats) = env.data {
                            calorie_stats.set(Some(stats));
                        }
                    }
                }
                Err(e) => {
                    log(&format!(
                        "[Profile] Calorie stats error (non-fatal): {:?}",
                        e
                    ));
                    let _ = handle_invoke_error(&e, auth, toast).await;
                }
            }
        });
    });

    // Fetch goals
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let args = crate::json_args!({});
            let promise = invoke("api_view_goals", args);

            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Ok(env) =
                        serde_wasm_bindgen::from_value::<ApiEnvelope<GoalsData>>(result)
                    {
                        if let Some(g) = env.data {
                            goals.set(Some(g));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Profile] Goals error (non-fatal): {:?}", e));
                    let _ = handle_invoke_error(&e, auth, toast).await;
                }
            }
        });
    });

    // Start editing profile - populate fields
    let start_edit_profile = move || {
        if let Some(p) = profile.get() {
            edit_first_name.set(p.first_name.unwrap_or_default());
            edit_last_name.set(p.last_name.unwrap_or_default());
            edit_dob.set(p.dob.unwrap_or_default());
            edit_gender.set(p.gender.unwrap_or_default());
            edit_height.set(p.height.unwrap_or_default());
            edit_weight.set(p.weight.unwrap_or_default());
            edit_address.set(p.address.unwrap_or_default());
        }
        editing_profile.set(true);
    };

    // Save profile changes
    let save_profile = move || {
        saving.set(true);
        save_message.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let args = crate::json_args!({
                "firstName": edit_first_name.get(),
                "lastName": edit_last_name.get(),
                "dob": edit_dob.get(),
                "gender": edit_gender.get(),
                "height": edit_height.get(),
                "weight": edit_weight.get(),
                "address": edit_address.get(),
            });

            let promise = invoke("api_update_profile", args);

            match JsFuture::from(promise).await {
                Ok(_) => {
                    save_message.set(Some("Profile updated successfully".to_string()));
                    editing_profile.set(false);

                    // Refresh profile data
                    let args = crate::json_args!({});
                    let promise = invoke("api_view_profile", args);
                    if let Ok(result) = JsFuture::from(promise).await {
                        if let Ok(response) =
                            serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                        {
                            let refreshed = response.get("data").and_then(|d| {
                                d.as_array()
                                    .and_then(|arr| arr.first())
                                    .and_then(|item| item.get("data"))
                                    .or(if d.is_object() { Some(d) } else { None })
                            });
                            if let Some(data) = refreshed {
                                if let Ok(p) = serde_json::from_value::<ProfileData>(data.clone()) {
                                    profile.set(Some(p));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Profile] Save error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        saving.set(false);
                        return;
                    }
                    save_message.set(Some("Failed to update profile".to_string()));
                }
            }

            saving.set(false);
        });
    };

    // Start editing goals
    let start_edit_goals = move || {
        if let Some(g) = goals.get() {
            edit_current_weight.set(g.current_weight_display());
            edit_target_weight.set(g.target_weight_display());
            edit_goal_date.set(g.target_weight_goal_date.unwrap_or_default());
            edit_weekly_sessions.set(g.weekly_session_goal.unwrap_or_default());
        }
        editing_goals.set(true);
    };

    // Save goals changes
    let save_goals = move || {
        saving.set(true);
        save_message.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let args = crate::json_args!({
                "currentWeight": edit_current_weight.get(),
                "targetWeight": edit_target_weight.get(),
                "targetWeightGoalDate": edit_goal_date.get(),
                "weeklySessionGoal": edit_weekly_sessions.get(),
            });

            let promise = invoke("api_update_goals", args);

            match JsFuture::from(promise).await {
                Ok(_) => {
                    save_message.set(Some("Goals updated successfully".to_string()));
                    editing_goals.set(false);

                    // Refresh goals data
                    let args = crate::json_args!({});
                    let promise = invoke("api_view_goals", args);
                    if let Ok(result) = JsFuture::from(promise).await {
                        if let Ok(response) =
                            serde_wasm_bindgen::from_value::<serde_json::Value>(result)
                        {
                            if let Some(data) = response.get("data") {
                                if let Ok(g) = serde_json::from_value::<GoalsData>(data.clone()) {
                                    goals.set(Some(g));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log(&format!("[Profile] Save goals error: {:?}", e));
                    if handle_invoke_error(&e, auth, toast).await {
                        saving.set(false);
                        return;
                    }
                    save_message.set(Some("Failed to update goals".to_string()));
                }
            }

            saving.set(false);
        });
    };

    view! {
        <div class="profile-page">
            {move || loading.get().then(|| view! { <PageLoading /> })}

            <div class="page-header">
                <h1 class="page-title">"Profile"</h1>
            </div>

            <div class="profile-content">
                // Success/error message
                {move || save_message.get().map(|msg| {
                    let class = if msg.contains("success") { "save-message success" } else { "save-message error" };
                    view! { <div class=class>{msg}</div> }
                })}

                // Profile info section
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">"Personal Info"</h2>
                        {move || {
                            if editing_profile.get() {
                                view! {
                                    <div class="section-actions">
                                        <button class="link-btn" on:click=move |_| editing_profile.set(false)>"Cancel"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="section-actions">
                                        <button class="link-btn" on:click=move |_| start_edit_profile()>"Edit"</button>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    {move || {
                        if editing_profile.get() {
                            // Edit mode
                            view! {
                                <div class="profile-form">
                                    <TextInput
                                        placeholder="First Name"
                                        value=edit_first_name
                                        label="First Name".to_string()
                                    />
                                    <TextInput
                                        placeholder="Last Name"
                                        value=edit_last_name
                                        label="Last Name".to_string()
                                    />
                                    <TextInput
                                        placeholder="Date of Birth (YYYY-MM-DD)"
                                        value=edit_dob
                                        label="Date of Birth".to_string()
                                        input_type="date".to_string()
                                    />
                                    <TextInput
                                        placeholder="Gender (M/F)"
                                        value=edit_gender
                                        label="Gender".to_string()
                                    />
                                    <TextInput
                                        placeholder="Height"
                                        value=edit_height
                                        label="Height".to_string()
                                    />
                                    <TextInput
                                        placeholder="Weight (lbs)"
                                        value=edit_weight
                                        label="Weight (lbs)".to_string()
                                    />
                                    <TextInput
                                        placeholder="Address"
                                        value=edit_address
                                        label="Address".to_string()
                                    />
                                    <Button
                                        label="Save Profile"
                                        on_click=save_profile
                                        loading=Signal::derive(move || saving.get())
                                    />
                                </div>
                            }.into_any()
                        } else {
                            // View mode
                            match profile.get() {
                                Some(p) => view! {
                                    <div class="profile-info">
                                        <div class="profile-avatar">
                                            <div class="avatar-circle">
                                                {p.first_name.as_ref().and_then(|n| n.chars().next()).unwrap_or('U').to_uppercase().to_string()}
                                            </div>
                                            <div class="profile-name">{p.display_name()}</div>
                                        </div>
                                        <div class="info-grid">
                                            <div class="info-item">
                                                <span class="info-label">"Email"</span>
                                                <span class="info-value">{p.display_email()}</span>
                                            </div>
                                            <div class="info-item">
                                                <span class="info-label">"Phone"</span>
                                                <span class="info-value">{p.display_phone()}</span>
                                            </div>
                                            <div class="info-item">
                                                <span class="info-label">"Date of Birth"</span>
                                                <span class="info-value">{p.display_dob()}</span>
                                            </div>
                                            <div class="info-item">
                                                <span class="info-label">"Gender"</span>
                                                <span class="info-value">{p.display_gender()}</span>
                                            </div>
                                            <div class="info-item">
                                                <span class="info-label">"Height"</span>
                                                <span class="info-value">{p.display_height()}</span>
                                            </div>
                                            <div class="info-item">
                                                <span class="info-label">"Weight"</span>
                                                <span class="info-value">{p.display_weight()}" lbs"</span>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! { <div class="empty-state">"No profile data"</div> }.into_any(),
                            }
                        }
                    }}
                </div>

                // Calorie Stats section
                <div class="section">
                    <h2 class="section-title">"Lifetime Stats"</h2>
                    {move || {
                        match calorie_stats.get() {
                            Some(stats) => view! {
                                <div class="stats-grid">
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.total_sessions_display()}</div>
                                        <div class="stat-label">"Total Sessions"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.total_calories_display()}</div>
                                        <div class="stat-label">"Total Calories"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.workout_calories_display()}</div>
                                        <div class="stat-label">"Workout Calories"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.afterburn_display()}</div>
                                        <div class="stat-label">"Afterburn"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.avg_calories_display()}</div>
                                        <div class="stat-label">"Avg / Session"</div>
                                    </div>
                                    <div class="stat-card">
                                        <div class="stat-value">{stats.last_workout_date.clone().unwrap_or_else(|| "--".to_string())}</div>
                                        <div class="stat-label">"Last Workout"</div>
                                    </div>
                                </div>
                            }.into_any(),
                            None => view! {
                                <div class="stats-grid">
                                    <div class="stat-card">
                                        <div class="stat-value">"--"</div>
                                        <div class="stat-label">"Loading stats..."</div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Goals section
                <div class="section">
                    <div class="section-header">
                        <h2 class="section-title">"Goals"</h2>
                        {move || {
                            if editing_goals.get() {
                                view! {
                                    <div class="section-actions">
                                        <button class="link-btn" on:click=move |_| editing_goals.set(false)>"Cancel"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="section-actions">
                                        <button class="link-btn" on:click=move |_| start_edit_goals()>"Edit"</button>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>

                    {move || {
                        if editing_goals.get() {
                            view! {
                                <div class="profile-form">
                                    <TextInput
                                        placeholder="Current Weight (lbs)"
                                        value=edit_current_weight
                                        label="Current Weight (lbs)".to_string()
                                    />
                                    <TextInput
                                        placeholder="Target Weight (lbs)"
                                        value=edit_target_weight
                                        label="Target Weight (lbs)".to_string()
                                    />
                                    <TextInput
                                        placeholder="Target Date (YYYY-MM-DD)"
                                        value=edit_goal_date
                                        label="Target Date".to_string()
                                        input_type="date".to_string()
                                    />
                                    <TextInput
                                        placeholder="Sessions per Week"
                                        value=edit_weekly_sessions
                                        label="Weekly Session Goal".to_string()
                                    />
                                    <Button
                                        label="Save Goals"
                                        on_click=save_goals
                                        loading=Signal::derive(move || saving.get())
                                    />
                                </div>
                            }.into_any()
                        } else {
                            match goals.get() {
                                Some(g) => view! {
                                    <div class="goals-grid">
                                        <div class="goal-card">
                                            <div class="goal-value">{g.current_weight_display()}</div>
                                            <div class="goal-label">"Current Weight (lbs)"</div>
                                        </div>
                                        <div class="goal-card">
                                            <div class="goal-value">{g.target_weight_display()}</div>
                                            <div class="goal-label">"Target Weight (lbs)"</div>
                                        </div>
                                        <div class="goal-card">
                                            <div class="goal-value">{g.goal_date_display()}</div>
                                            <div class="goal-label">"Target Date"</div>
                                        </div>
                                        <div class="goal-card">
                                            <div class="goal-value">{g.weekly_sessions_display()}</div>
                                            <div class="goal-label">"Sessions / Week"</div>
                                        </div>
                                    </div>
                                }.into_any(),
                                None => view! {
                                    <div class="goals-grid">
                                        <div class="goal-card">
                                            <div class="goal-value">"--"</div>
                                            <div class="goal-label">"No goals set"</div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </div>

                {move || error.get().map(|e| view! {
                    <div class="error-message">{e}</div>
                })}
            </div>

            <BottomNav active=Signal::derive(|| NavItem::Profile) />
        </div>
    }
}
