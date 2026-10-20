//! Classification of `booking/bookSession_v2` bodies. HOTWORX returns HTTP
//! 200 for every one of these, so the body is the only signal.

use hotworx_api::{BookSessionResponse, BookingOutcome};

fn parse(json: &str) -> BookSessionResponse {
    serde_json::from_str(json).expect("valid booking body")
}

#[test]
fn success_with_session_id_is_accepted() {
    let r = parse(r#"{"success":"true","session_id":48213,"payment_status":"true"}"#);
    assert_eq!(
        r.outcome(),
        BookingOutcome::Accepted {
            session_id: Some("48213".into())
        }
    );
}

#[test]
fn legacy_data_envelope_is_accepted_with_record_id() {
    let r = parse(r#"{"status":"ok","data":{"session_record_id":"777","lead_record_id":"9"}}"#);
    assert_eq!(
        r.outcome(),
        BookingOutcome::Accepted {
            session_id: Some("777".into())
        }
    );
}

#[test]
fn empty_body_is_accepted_without_id() {
    // A bare 200 at a home studio: the official app treats this as booked.
    // We accept it but return no id so callers know to verify.
    let r = parse("{}");
    assert_eq!(r.outcome(), BookingOutcome::Accepted { session_id: None });
}

#[test]
fn explicit_success_false_is_rejected_with_server_text() {
    let r = parse(r#"{"success":false,"text":"You already have a session at this time."}"#);
    assert_eq!(
        r.outcome(),
        BookingOutcome::Rejected {
            message: "You already have a session at this time.".into()
        }
    );
}

#[test]
fn error_field_is_rejected() {
    let r = parse(r#"{"error":"Slot no longer available"}"#);
    assert_eq!(
        r.outcome(),
        BookingOutcome::Rejected {
            message: "Slot no longer available".into()
        }
    );
}

#[test]
fn message_popup_means_payment_required() {
    let r = parse(
        r#"{"message_popup":true,"payment_status":false,"card_available":false,
            "text":"A reciprocal fee of $10 applies at this location.",
            "add_card_url":"https://pay.example/add"}"#,
    );
    assert_eq!(
        r.outcome(),
        BookingOutcome::PaymentRequired {
            message: "A reciprocal fee of $10 applies at this location.".into(),
            add_card_url: Some("https://pay.example/add".into()),
        }
    );
}

#[test]
fn home_studio_body_with_payment_status_false_is_still_accepted() {
    // Home studios return payment_status "false" on good bookings; only the
    // explicit payment markers may flip a booking to PaymentRequired.
    let r =
        parse(r#"{"success":"true","session_id":123,"payment_status":"false","text":"Booked"}"#);
    assert_eq!(
        r.outcome(),
        BookingOutcome::Accepted {
            session_id: Some("123".into())
        }
    );
}

#[test]
fn unknown_success_string_is_not_a_rejection() {
    let r = parse(r#"{"success":"ok","session_id":5}"#);
    assert!(matches!(r.outcome(), BookingOutcome::Accepted { .. }));
    let r = parse(r#"{"success":""}"#);
    assert!(matches!(r.outcome(), BookingOutcome::Accepted { .. }));
    let r = parse(r#"{"success":"0","text":"Nope"}"#);
    assert!(matches!(r.outcome(), BookingOutcome::Rejected { .. }));
}

#[test]
fn card_on_file_still_needs_confirmation() {
    let r = parse(r#"{"card_available":true,"payment_status":false,"text":"Confirm $10 fee"}"#);
    assert!(matches!(
        r.outcome(),
        BookingOutcome::PaymentRequired { .. }
    ));
}

#[test]
fn server_message_falls_back_to_raw_json() {
    let r = parse(r#"{"something_new":1}"#);
    let msg = r.server_message();
    assert!(msg.starts_with('{'), "expected JSON fallback, got {msg}");
    assert!(
        msg.contains("\"status\""),
        "fallback should be the serialized body: {msg}"
    );
}
