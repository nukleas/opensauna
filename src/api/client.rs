use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const BASE_URL: &str = "https://sailposapi.hotworx.net/api/v1";

/// Simple URL encoding for form data
fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;

    // Tauri HTTP plugin fetch
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "http"])]
    fn fetch(url: &str, options: JsValue) -> js_sys::Promise;

    // Console logging for debugging
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

/// API Client for making HTTP requests through Tauri
#[derive(Clone)]
pub struct ApiClient {
    token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub fn with_token(token: String) -> Self {
        Self { token: Some(token) }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    /// Make a POST request to the API
    pub async fn post<T, R>(&self, endpoint: &str, body: &T) -> Result<R, ApiError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", BASE_URL, endpoint);
        self.fetch_request("POST", &url, Some(body)).await
    }

    /// Make a GET request to the API
    pub async fn get<R>(&self, endpoint: &str) -> Result<R, ApiError>
    where
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", BASE_URL, endpoint);
        self.fetch_request::<(), R>("GET", &url, None).await
    }

    async fn fetch_request<T, R>(
        &self,
        method: &str,
        url: &str,
        body: Option<&T>,
    ) -> Result<R, ApiError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.fetch_internal(method, url, body, false).await
    }

    /// Make a form-urlencoded POST request
    pub async fn post_form<T, R>(&self, endpoint: &str, body: &T) -> Result<R, ApiError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let url = format!("{}/{}", BASE_URL, endpoint);
        self.fetch_internal("POST", &url, Some(body), true).await
    }

    async fn fetch_internal<T, R>(
        &self,
        method: &str,
        url: &str,
        body: Option<&T>,
        form_encoded: bool,
    ) -> Result<R, ApiError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        log(&format!("[API] {} {}", method, url));

        // Get device ID for headers
        let device_id = get_device_id()
            .await
            .unwrap_or_else(|_| format!("tauri-{}", js_sys::Date::now() as u64));

        // Build headers as a simple string map - matching the Android app
        let mut headers_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        if form_encoded {
            headers_map.insert(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            );
        } else {
            headers_map.insert("Content-Type".to_string(), "application/json".to_string());
        }
        headers_map.insert("User-Agent".to_string(), "okhttp/4.9.3".to_string());
        headers_map.insert("sec-ch-ua-platform".to_string(), "Android".to_string());
        headers_map.insert("application-version".to_string(), "5.0.0".to_string());
        headers_map.insert("device-id".to_string(), device_id.clone());

        if let Some(ref token) = self.token {
            headers_map.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        log(&format!("[API] Headers: {:?}", headers_map));

        // Build request options for Tauri HTTP plugin
        let mut options = serde_json::Map::new();
        options.insert("method".to_string(), serde_json::json!(method));
        options.insert(
            "headers".to_string(),
            serde_json::to_value(&headers_map).unwrap(),
        );

        if let Some(b) = body {
            if form_encoded {
                // Convert to form-urlencoded
                let body_value = serde_json::to_value(b).map_err(|e| ApiError {
                    message: format!("Failed to serialize body: {}", e),
                })?;

                let form_body = if let serde_json::Value::Object(map) = body_value {
                    map.iter()
                        .map(|(k, v)| {
                            let value_str = match v {
                                serde_json::Value::String(s) => s.clone(),
                                _ => v.to_string().trim_matches('"').to_string(),
                            };
                            format!("{}={}", urlencoded(k), urlencoded(&value_str))
                        })
                        .collect::<Vec<_>>()
                        .join("&")
                } else {
                    return Err(ApiError {
                        message: "Body must be an object for form encoding".to_string(),
                    });
                };

                log(&format!("[API] Form body: {}", form_body));
                options.insert("body".to_string(), serde_json::json!(form_body));
            } else {
                let body_json = serde_json::to_string(b).map_err(|e| ApiError {
                    message: format!("Failed to serialize body: {}", e),
                })?;
                log(&format!("[API] Request body: {}", body_json));
                options.insert("body".to_string(), serde_json::json!(body_json));
            }
        }

        // Convert options to JsValue
        let options_js = serde_wasm_bindgen::to_value(&serde_json::Value::Object(options))
            .map_err(|e| ApiError {
                message: format!("Failed to convert options: {:?}", e),
            })?;

        // Call Tauri's HTTP fetch
        let promise = fetch(url, options_js);
        let result = JsFuture::from(promise).await.map_err(|e| {
            error(&format!("[API] HTTP request failed: {:?}", e));
            ApiError {
                message: format!("HTTP request failed: {:?}", e),
            }
        })?;

        log("[API] Got response, extracting body...");

        // The result is a Response object, we need to get the JSON body
        // First, call .text() to see raw response for debugging
        let response: js_sys::Object = result.dyn_into().map_err(|_| ApiError {
            message: "Response is not an object".to_string(),
        })?;

        // Log response status
        if let Ok(status) = js_sys::Reflect::get(&response, &JsValue::from_str("status")) {
            log(&format!("[API] Response status: {:?}", status));
        }

        // Get text first for debugging
        let text_fn =
            js_sys::Reflect::get(&response, &JsValue::from_str("text")).map_err(|e| ApiError {
                message: format!("Failed to get text method: {:?}", e),
            })?;

        let text_fn: js_sys::Function = text_fn.dyn_into().map_err(|_| ApiError {
            message: "text is not a function".to_string(),
        })?;

        let text_promise = text_fn.call0(&response).map_err(|e| ApiError {
            message: format!("Failed to call text(): {:?}", e),
        })?;

        let text_promise: js_sys::Promise = text_promise.dyn_into().map_err(|_| ApiError {
            message: "text() did not return a promise".to_string(),
        })?;

        let text_result = JsFuture::from(text_promise).await.map_err(|e| ApiError {
            message: format!("Failed to get text: {:?}", e),
        })?;

        let text_str = text_result
            .as_string()
            .unwrap_or_else(|| "No text".to_string());
        log(&format!(
            "[API] Response body: {}",
            &text_str[..text_str.len().min(500)]
        ));

        // Parse the text as JSON
        let parsed: R = serde_json::from_str(&text_str).map_err(|e| ApiError {
            message: format!(
                "Failed to parse JSON: {} - Body: {}",
                e,
                &text_str[..text_str.len().min(200)]
            ),
        })?;

        Ok(parsed)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a password using SHA-256 via Tauri command
pub async fn hash_password(password: &str) -> Result<String, ApiError> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "password": password
    }))
    .map_err(|e| ApiError {
        message: format!("Failed to convert args: {:?}", e),
    })?;

    let promise = invoke("hash_password", args);
    let result = JsFuture::from(promise).await.map_err(|e| ApiError {
        message: format!("Hash failed: {:?}", e),
    })?;

    serde_wasm_bindgen::from_value(result).map_err(|e| ApiError {
        message: format!("Failed to parse hash: {:?}", e),
    })
}

/// Get device ID via Tauri command
pub async fn get_device_id() -> Result<String, ApiError> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| ApiError {
        message: format!("Failed to convert args: {:?}", e),
    })?;

    let promise = invoke("get_device_id", args);
    let result = JsFuture::from(promise).await.map_err(|e| ApiError {
        message: format!("Get device ID failed: {:?}", e),
    })?;

    serde_wasm_bindgen::from_value(result).map_err(|e| ApiError {
        message: format!("Failed to parse device ID: {:?}", e),
    })
}

/// Store auth token via Tauri command
pub async fn store_auth_token(token: &str) -> Result<(), ApiError> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "token": token
    }))
    .map_err(|e| ApiError {
        message: format!("Failed to convert args: {:?}", e),
    })?;

    let promise = invoke("store_auth_token", args);
    JsFuture::from(promise).await.map_err(|e| ApiError {
        message: format!("Store token failed: {:?}", e),
    })?;

    Ok(())
}

/// Get stored auth token via Tauri command
pub async fn get_auth_token() -> Result<Option<String>, ApiError> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| ApiError {
        message: format!("Failed to convert args: {:?}", e),
    })?;

    let promise = invoke("get_auth_token", args);
    let result = JsFuture::from(promise).await.map_err(|e| ApiError {
        message: format!("Get token failed: {:?}", e),
    })?;

    serde_wasm_bindgen::from_value(result).map_err(|e| ApiError {
        message: format!("Failed to parse token: {:?}", e),
    })
}

/// Clear auth token via Tauri command
pub async fn clear_auth_token() -> Result<(), ApiError> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({})).map_err(|e| ApiError {
        message: format!("Failed to convert args: {:?}", e),
    })?;

    let promise = invoke("clear_auth_token", args);
    JsFuture::from(promise).await.map_err(|e| ApiError {
        message: format!("Clear token failed: {:?}", e),
    })?;

    Ok(())
}
