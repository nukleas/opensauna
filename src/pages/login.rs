use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::components::{Button, TextInput, LoadingOverlay, IconFlame};
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

/// Response from login API
#[derive(Debug, Clone, serde::Deserialize)]
struct LoginResponse {
    msg: Option<String>,
    token: Option<String>,
    two_factor: Option<String>,
    error: Option<String>,
}

impl LoginResponse {
    fn requires_otp(&self) -> bool {
        self.two_factor.as_deref() == Some("yes")
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth_state();

    // Redirect to dashboard if already authenticated
    Effect::new(move |_| {
        if !auth.loading.get() && auth.token.get().is_some() {
            navigate_to("/");
        }
    });

    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    let on_submit = move || {
        let email_val = email.get();
        let password_val = password.get();

        if email_val.is_empty() || password_val.is_empty() {
            error.set(Some("Please enter email and password".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            log("[Login] Attempting login...");

            // Call backend API command
            let args = serde_wasm_bindgen::to_value(&serde_json::json!({
                "email": email_val.clone(),
                "password": password_val.clone()
            })).unwrap();

            let promise = invoke("api_login_with_password", args);
            match JsFuture::from(promise).await {
                Ok(result) => {
                    log("[Login] Got result");

                    let response: LoginResponse = serde_wasm_bindgen::from_value(result)
                        .unwrap_or_else(|e| {
                            log(&format!("[Login] Parse error: {:?}", e));
                            LoginResponse {
                                msg: None,
                                token: None,
                                two_factor: None,
                                error: Some(format!("Parse error: {:?}", e)),
                            }
                        });

                    if response.requires_otp() {
                        // Store credentials in Tauri store for OTP verification (survives page nav)
                        if let Some(token) = response.token {
                            let store_args = serde_wasm_bindgen::to_value(&serde_json::json!({
                                "email": email_val,
                                "password": password_val,
                                "token": token
                            })).unwrap();
                            let _ = JsFuture::from(invoke("store_pending_login", store_args)).await;
                            navigate_to("/otp");
                        }
                    } else if let Some(token) = response.token {
                        auth.set_token(token).await;
                        navigate_to("/");
                    } else {
                        error.set(Some(response.error.unwrap_or_else(|| "Login failed".to_string())));
                    }
                }
                Err(e) => {
                    log(&format!("[Login] Error: {:?}", e));
                    let err_str = js_sys::JSON::stringify(&e)
                        .map(|s| s.as_string().unwrap_or_default())
                        .unwrap_or_else(|_| format!("{:?}", e));
                    error.set(Some(format!("Login failed: {}", err_str)));
                }
            }

            loading.set(false);
        });
    };

    view! {
        <div class="login-page">
            {move || loading.get().then(|| view! { <LoadingOverlay message="Logging in...".to_string() /> })}

            <div class="login-container">
                <div class="login-header">
                    <div class="login-logo">
                        <IconFlame size=crate::components::icons::IconSize::Xl />
                    </div>
                    <h1 class="login-title">"BOOKWORX"</h1>
                    <p class="login-subtitle">"Sign in to your account"</p>
                </div>

                <div class="login-form">
                    <TextInput
                        placeholder="Email"
                        value=email
                        input_type="email".to_string()
                        label="Email".to_string()
                    />

                    <TextInput
                        placeholder="Password"
                        value=password
                        input_type="password".to_string()
                        label="Password".to_string()
                    />

                    {move || error.get().map(|e| view! {
                        <div class="error-message">{e}</div>
                    })}

                    <Button
                        label="Sign In"
                        loading=Signal::derive(move || loading.get())
                        on_click=on_submit
                    />
                </div>
            </div>
        </div>
    }
}
