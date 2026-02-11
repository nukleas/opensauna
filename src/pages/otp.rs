use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::components::{Button, OtpInput, LoadingOverlay, IconChevronLeft};
use crate::state::use_auth_state;

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

/// Response from verify OTP API
#[derive(Debug, Clone, serde::Deserialize)]
struct VerifyOtpResponse {
    #[allow(dead_code)]
    msg: Option<String>,
    token: Option<String>,
    error: Option<String>,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

/// Pending login data from Tauri store
#[derive(Debug, Clone, serde::Deserialize)]
struct PendingLoginData(String, String, String); // (email, password, token)

#[component]
pub fn OtpPage() -> impl IntoView {
    let auth = use_auth_state();

    // Pre-fill OTP for testing
    let otp = RwSignal::new("123456".to_string());
    let loading = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    let on_verify = move || {
        let otp_val = otp.get();

        if otp_val.len() < 6 {
            error.set(Some("Please enter the 6-digit code".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            // Get pending login data from Tauri store
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
            let pending_result = JsFuture::from(invoke("get_pending_login", args)).await;

            let pending_data: Option<PendingLoginData> = match pending_result {
                Ok(val) => serde_wasm_bindgen::from_value(val).ok(),
                Err(_) => None,
            };

            let Some(PendingLoginData(email, password, token)) = pending_data else {
                error.set(Some("Session expired. Please login again.".to_string()));
                loading.set(false);
                navigate_to("/login");
                return;
            };

            log(&format!("[OTP] Verifying OTP for: {}", email));

            // Call backend API command
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "email": email,
                "password": password,
                "otp": otp_val,
                "token": token
            })).unwrap();

            let promise = invoke("api_verify_otp", args);
            match JsFuture::from(promise).await {
                Ok(result) => {
                    log(&format!("[OTP] Got result: {:?}", result));

                    let response: VerifyOtpResponse = serde_wasm_bindgen::from_value(result)
                        .unwrap_or_else(|e| {
                            log(&format!("[OTP] Parse error: {:?}", e));
                            VerifyOtpResponse {
                                msg: None,
                                token: None,
                                error: Some(format!("Parse error: {:?}", e)),
                                data: None,
                            }
                        });

                    if let Some(token) = response.token {
                        // Clear pending login from store
                        let clear_args = serde_wasm_bindgen::to_value(&serde_json::json!({})).unwrap();
                        let _ = JsFuture::from(invoke("clear_pending_login", clear_args)).await;
                        // Store auth token
                        auth.set_token(token).await;
                        navigate_to("/");
                    } else {
                        error.set(Some(response.error.unwrap_or_else(|| "Verification failed".to_string())));
                    }
                }
                Err(e) => {
                    log(&format!("[OTP] Error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Verification failed: {}", err_str)));
                }
            }

            loading.set(false);
        });
    };

    let on_back = move || {
        navigate_to("/login");
    };

    view! {
        <div class="otp-page">
            {move || loading.get().then(|| view! { <LoadingOverlay message="Verifying...".to_string() /> })}

            <div class="otp-container">
                <div class="otp-header">
                    <button class="back-button" on:click=move |_| on_back()>
                        <IconChevronLeft size=crate::components::icons::IconSize::Sm />
                        "Back"
                    </button>
                    <h1 class="otp-title">"Verify OTP"</h1>
                    <p class="otp-subtitle">"Enter the 6-digit code sent to your phone"</p>
                </div>

                <div class="otp-form">
                    <OtpInput value=otp length=6 />

                    {move || error.get().map(|e| view! {
                        <div class="error-message">{e}</div>
                    })}

                    <Button
                        label="Verify"
                        loading=Signal::derive(move || loading.get())
                        on_click=on_verify
                    />

                    <button class="resend-button">
                        "Resend Code"
                    </button>
                </div>
            </div>
        </div>
    }
}
