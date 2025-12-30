use crate::api::client::{ApiClient, ApiError};
use crate::models::booking::{
    BookSessionRequest, BookSessionResponse, DeleteSessionRequest, GetLevelTwoRequest,
    ShowSlotsRequest, SlotsResponse,
};
use crate::models::location::LocationsResponse;

impl ApiClient {
    /// Get available booking locations
    pub async fn get_booking_locations(&self) -> Result<LocationsResponse, ApiError> {
        self.get("booking/getBookingLocations_v2").await
    }

    /// Get available time slots for a date/location
    pub async fn show_slots(
        &self,
        booking_date: &str,
        location_id: &str,
        session_type: &str,
    ) -> Result<SlotsResponse, ApiError> {
        let request = ShowSlotsRequest {
            booking_date: booking_date.to_string(),
            location_id: location_id.to_string(),
            view_type: "day".to_string(),
            time: "all".to_string(),
            session_type: session_type.to_string(),
        };

        self.post("booking/showSlots", &request).await
    }

    /// Get booking level two details
    pub async fn get_level_two(
        &self,
        booking_date: &str,
        location_id: &str,
    ) -> Result<SlotsResponse, ApiError> {
        let request = GetLevelTwoRequest {
            booking_date: booking_date.to_string(),
            location_id: location_id.to_string(),
            view_type: "day".to_string(),
        };

        self.post("booking/getLevelTwo_v2", &request).await
    }

    /// Book a session
    pub async fn book_session(
        &self,
        sauna_no: &str,
        time_slot: &str,
        booking_date: &str,
        session_type: &str,
        location_id: &str,
    ) -> Result<BookSessionResponse, ApiError> {
        let request = BookSessionRequest {
            sauna_no: sauna_no.to_string(),
            time_slot: time_slot.to_string(),
            booking_date: booking_date.to_string(),
            session_type: session_type.to_string(),
            selected_location_id: location_id.to_string(),
            message_popup: None,
        };

        self.post("booking/bookSession_v2", &request).await
    }

    /// Cancel/delete a session
    pub async fn delete_session(
        &self,
        session_record_id: &str,
        lead_record_id: &str,
    ) -> Result<BookSessionResponse, ApiError> {
        let request = DeleteSessionRequest {
            session_record_id: session_record_id.to_string(),
            lead_record_id: lead_record_id.to_string(),
        };

        self.post("booking/deleteSession", &request).await
    }
}
