//! Shared deserialization shapes for IPC responses.
//!
//! The native backend returns most endpoints as typed envelopes
//! (see `src-tauri/src/lib.rs`). The frontend deserializes the IPC value
//! into the matching shape here instead of walking a `serde_json::Value`
//! by string keys.

use serde::Deserialize;

/// `{ "data": T }` — the most common response shape.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiEnvelope<T> {
    #[serde(default = "Option::default")]
    pub data: Option<T>,
}

/// `{ "data": [ { "data": T }, ... ] }` — used by the few endpoints
/// (profile, 30-day summary) that come back double-wrapped.
#[derive(Debug, Clone, Deserialize)]
pub struct NestedDataEnvelope<T> {
    #[serde(default = "Vec::new")]
    pub data: Vec<DataItem<T>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataItem<T> {
    pub data: T,
}

impl<T> NestedDataEnvelope<T> {
    /// First inner `data` value, if any.
    pub fn first(self) -> Option<T> {
        self.data.into_iter().next().map(|item| item.data)
    }
}
