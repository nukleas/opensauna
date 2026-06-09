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

/// The three bookable days (today + the next two), each as
/// `(YYYY-MM-DD, short label)` — e.g. `("2026-06-08", "Today")`. HOTWORX only
/// allows booking within a 3-day window, so a tiny pill row beats a native
/// date picker.
pub fn bookable_days() -> Vec<(String, String)> {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    (0..3)
        .map(|offset| {
            let d = js_sys::Date::new_0();
            d.set_date(d.get_date() + offset);
            let ymd = format!(
                "{:04}-{:02}-{:02}",
                d.get_full_year(),
                d.get_month() + 1,
                d.get_date()
            );
            let label = match offset {
                0 => "Today".to_string(),
                1 => "Tomorrow".to_string(),
                _ => WEEKDAYS[(d.get_day() as usize) % 7].to_string(),
            };
            (ymd, label)
        })
        .collect()
}
