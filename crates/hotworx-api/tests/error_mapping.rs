//! End-to-end tests against a wiremock server: exercises the request
//! shape (URL, headers, form body) and the HTTP-status-to-`HotworxError`
//! mapping that `HotworxClient` performs.

use hotworx_api::{HotworxClient, HotworxError};
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client_at(server: &MockServer) -> HotworxClient {
    HotworxClient::new("test-device")
        .with_token("test-token")
        .with_base_url(format!("{}/api/v1", server.uri()))
}

#[tokio::test]
async fn missing_token_short_circuits_to_auth_expired() {
    let client = HotworxClient::new("test-device");
    match client.get_dashboard(None).await {
        Err(HotworxError::AuthExpired) => {}
        other => panic!("expected AuthExpired, got {:?}", other),
    }
}

#[tokio::test]
async fn server_401_maps_to_auth_expired() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/getDashboard"))
        .respond_with(ResponseTemplate::new(401).set_body_string("token expired"))
        .mount(&server)
        .await;

    let client = client_at(&server);
    match client.get_dashboard(None).await {
        Err(HotworxError::AuthExpired) => {}
        other => panic!("expected AuthExpired, got {:?}", other),
    }
}

#[tokio::test]
async fn server_403_maps_to_auth_expired() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/getDashboard"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let client = client_at(&server);
    match client.get_dashboard(None).await {
        Err(HotworxError::AuthExpired) => {}
        other => panic!("expected AuthExpired, got {:?}", other),
    }
}

#[tokio::test]
async fn server_500_surfaces_as_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/getDashboard"))
        .respond_with(ResponseTemplate::new(500).set_body_string("backend on fire"))
        .mount(&server)
        .await;

    let client = client_at(&server);
    match client.get_dashboard(None).await {
        Err(HotworxError::Http { status, body }) => {
            assert_eq!(status, 500);
            assert_eq!(body, "backend on fire");
        }
        other => panic!("expected Http, got {:?}", other),
    }
}

#[tokio::test]
async fn dashboard_unwraps_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/getDashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "data": {
                "todays_pending_sessions": [],
                "todays_completed_sessions": [],
                "summary": {
                    "total_sessions": "42",
                    "total_cal_burned": "12345",
                    "continious_streak": "7"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client_at(&server);
    let dash = client.get_dashboard(Some("2026-05-04")).await.unwrap();
    let summary = dash.summary.expect("summary present");
    assert_eq!(summary.total_sessions.as_deref(), Some("42"));
    assert_eq!(summary.continious_streak.as_deref(), Some("7"));
}

#[tokio::test]
async fn login_sends_hashed_password_and_app_headers() {
    let server = MockServer::start().await;
    let hashed = hotworx_api::password_hash("hunter2");

    Mock::given(method("POST"))
        .and(path("/api/v1/loginwithpassword"))
        .and(header("Content-Type", "application/x-www-form-urlencoded"))
        .and(header("User-Agent", "okhttp/4.12.0"))
        .and(header("application-version", "6.5.5"))
        .and(header("device-id", "test-device"))
        .and(header("sec-ch-ua-platform", "Android"))
        .and(body_string_contains(format!("password={}", hashed)))
        .and(body_string_contains("email_address=alice%40example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "msg": "success",
            "token": "abc123",
            "status": "ok"
        })))
        .mount(&server)
        .await;

    let client =
        HotworxClient::new("test-device").with_base_url(format!("{}/api/v1", server.uri()));
    let resp = client
        .login_with_password("alice@example.com", "hunter2")
        .await
        .unwrap();
    assert_eq!(resp.token.as_deref(), Some("abc123"));
}

#[tokio::test]
async fn show_slots_accepts_bare_array() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/booking/showSlots"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "time_slot": "08:00", "session_name": "HOT YOGA", "sauna_no": "1" },
            { "time_slot": "08:30", "session_name": "HOT YOGA", "sauna_no": "2" }
        ])))
        .mount(&server)
        .await;

    let client = client_at(&server);
    let slots = client
        .show_slots("loc-1", "2026-05-04", "HOT YOGA")
        .await
        .unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].time_slot.as_deref(), Some("08:00"));
}

#[tokio::test]
async fn show_slots_accepts_data_wrapped_form() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/booking/showSlots"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "slots": [{ "time_slot": "10:00", "sauna_no": "3" }]
            }
        })))
        .mount(&server)
        .await;

    let client = client_at(&server);
    let slots = client
        .show_slots("loc-1", "2026-05-04", "HOT YOGA")
        .await
        .unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].time_slot.as_deref(), Some("10:00"));
}
