//! Lock down the wire-compatible SHA-256 hashing.

use hotworx_api::password_hash;

#[test]
fn hash_matches_known_sha256_of_password() {
    assert_eq!(
        password_hash("password"),
        "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
    );
}

#[test]
fn hash_of_empty_string_matches_known_value() {
    assert_eq!(
        password_hash(""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn hash_is_64_hex_chars() {
    let h = password_hash("anything-goes-here");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_is_deterministic() {
    assert_eq!(password_hash("same"), password_hash("same"));
    assert_ne!(password_hash("same"), password_hash("different"));
}
