//! Cover the realistic shapes of the LoginResponse wire format.

use hotworx_api::LoginResponse;

#[test]
fn first_factor_success_includes_token() {
    let json = r#"{"msg":"success","token":"abc123","two_factor":null,"error":null,"status":"ok"}"#;
    let resp: LoginResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.token.as_deref(), Some("abc123"));
    assert!(resp.is_success());
    assert!(!resp.requires_otp());
}

#[test]
fn two_factor_required_signals_otp_step() {
    let json =
        r#"{"msg":null,"token":"temp123","two_factor":"yes","error":null,"status":"pending"}"#;
    let resp: LoginResponse = serde_json::from_str(json).unwrap();
    assert!(resp.requires_otp());
    assert!(resp.is_success());
}

#[test]
fn server_error_clears_success() {
    let json = r#"{"msg":null,"token":null,"two_factor":null,"error":"Invalid credentials","status":"error"}"#;
    let resp: LoginResponse = serde_json::from_str(json).unwrap();
    assert!(!resp.is_success());
    assert_eq!(resp.error.as_deref(), Some("Invalid credentials"));
}
