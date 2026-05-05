//! Date helpers used by the booking pages.
//!
//! HOTWORX only allows booking up to two days out, so we constrain the
//! `<input type="date">` min/max accordingly.

/// Today's date in `YYYY-MM-DD` format (local timezone, via the JS `Date`).
pub fn today() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// Latest bookable date — today + 2 days, in `YYYY-MM-DD` format.
pub fn max_booking_date() -> String {
    let now = js_sys::Date::new_0();
    now.set_date(now.get_date() + 2);
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}
