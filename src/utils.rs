use chrono::{DateTime, Datelike, Timelike, Utc};
use iso8601_timestamp::{time::{Date, PrimitiveDateTime, Time}, Timestamp};


pub fn convert_to_offset_datetime(input: DateTime<Utc>) -> Timestamp {
    let month_index: u8 = input.month().try_into().unwrap();

    return PrimitiveDateTime::new(
        Date::from_calendar_date(
            input.year(),
            month_index.try_into().unwrap(),
            input.day().try_into().unwrap()
        ).unwrap(),
        Time::from_hms(
            input.hour().try_into().unwrap(),
            input.minute().try_into().unwrap(),
            input.second().try_into().unwrap()
        ).unwrap()
    ).into()
}
