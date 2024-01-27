use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use reqwest;
use icalendar::{self, parser::Property};

use crate::config;


pub struct TwitchEvent {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub categories: String,
}


fn parse_property_datetime(property: &Property) -> DateTime<Utc> {
    let timzone_str = &property.params[0].val.clone().unwrap().to_string()[1..];
    let tz: Tz = timzone_str.parse().unwrap();

    let dt = NaiveDateTime::parse_from_str(
        &property.val.to_string(),
        "%Y%m%dT%H%M%S"
    ).unwrap();

    tz
        .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), dt.second())
        .unwrap()
        .to_utc()
}


pub async fn get_twitch_events() -> Result<Vec<TwitchEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::Client::new()
        .get(format!("https://api.twitch.tv/helix/schedule/icalendar?broadcaster_id={}", config::CONFIG.broadcast_id))
        .send()
        .await?
        .error_for_status()?;

    let events_text = response.text().await.unwrap();

    let calendar = icalendar::parser::read_calendar(&events_text).unwrap();

    let events: Vec<TwitchEvent> = calendar
        .components
        .into_iter()
        .map(|component| {
            TwitchEvent {
                start_at: parse_property_datetime(&component.properties[2]),
                end_at: parse_property_datetime(&component.properties[3]),
                name: component.properties[4].val.to_string(),
                description: component.properties[5].val.to_string(),
                categories: component.properties[6].val.to_string()
            }
        })
        .filter(|event| event.start_at > Utc::now())
        .collect();

    Ok(events)
}
