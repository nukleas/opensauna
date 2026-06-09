use crate::components::{Button, IconChevronLeft, LoadingOverlay, OtpInput};
use crate::models::auth::LoginResponse as VerifyOtpResponse;
use crate::state::use_auth_state;
use crate::utils::nav::go as navigate_to;
use crate::utils::tauri::{invoke, log};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

/// Pending login data from Tauri store: `(email, password, token)`.
#[derive(Debug, Clone, serde::Deserialize)]
struct PendingLoginData(String, String, String);

/// OTP code entry page, shown after password login when two-factor auth is enabled.
#[component]
pub fn OtpPage() -> impl IntoView {
    let auth = use_auth_state();

    // HOTWORX's "one-time passcode" is the constant 123456 — the server emails
    // that exact value every time and doesn't actually validate it. Prefilling
    // it is correct behavior, not a test stub. If they ever ship real codes,
    // the field stays editable.
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
            let args = crate::json_args!({});
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

            log("[OTP] Verifying OTP...");

            // Call backend API command
            let args = crate::json_args!({
                "email": email,
                "password": password,
                "otp": otp_val,
                "token": token,
            });

            let promise = invoke("api_verify_otp", args);
            match JsFuture::from(promise).await {
                Ok(result) => {
                    // Note: do not log `result` — it carries the bearer token.
                    let response: VerifyOtpResponse = serde_wasm_bindgen::from_value(result)
                        .unwrap_or_else(|e| {
                            log(&format!("[OTP] Parse error: {:?}", e));
                            VerifyOtpResponse {
                                error: Some(format!("Parse error: {:?}", e)),
                                ..Default::default()
                            }
                        });

                    if let Some(token) = response.token {
                        // Clear pending login from store
                        let clear_args = crate::json_args!({});
                        let _ = JsFuture::from(invoke("clear_pending_login", clear_args)).await;
                        // Store auth token
                        auth.set_token(token).await;
                        navigate_to("/");
                    } else {
                        error.set(Some(
                            response
                                .error
                                .unwrap_or_else(|| "Verification failed".to_string()),
                        ));
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
        // Drop the stashed credentials (incl. the password) when abandoning OTP.
        wasm_bindgen_futures::spawn_local(async move {
            let _ = JsFuture::from(invoke("clear_pending_login", crate::json_args!({}))).await;
            navigate_to("/login");
        });
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

                    // No resend endpoint exists, and the code is prefilled, so a
                    // dead "Resend" button just reads as broken. Show a hint instead.
                    <p class="resend-hint">"The code is already filled in — just tap Verify."</p>
                </div>
            </div>
        </div>
    }
}
