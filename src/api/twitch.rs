use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use reqwest;
use icalendar::{self, parser::Property};
use crate::config;
use chrono::Weekday;


#[derive(Clone)]
pub enum RepeatRule {
    Weekly(chrono::Weekday)
}


#[derive(Clone)]
pub struct TwitchEvent {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub categories: String,
    pub repeat_rule: Option<RepeatRule>
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
            let repeat_rule = if component.properties.len() > 7 {
                let repeat_rule_str = &component.properties[7].val.to_string();

                if repeat_rule_str.starts_with("FREQ=WEEKLY") {
                    let day_str = repeat_rule_str.split(';').nth(1).unwrap().split('=').nth(1).unwrap();
                    let day = match day_str {
                        "MO" => Weekday::Mon,
                        "TU" => Weekday::Tue,
                        "WE" => Weekday::Wed,
                        "TH" => Weekday::Thu,
                        "FR" => Weekday::Fri,
                        "SA" => Weekday::Sat,
                        "SU" => Weekday::Sun,
                        _ => panic!("Invalid day of week")
                    };

                    Some(RepeatRule::Weekly(day))
                } else {
                    None
                }
            } else {
                None
            };

            let event = TwitchEvent {
                start_at: parse_property_datetime(&component.properties[2]),
                end_at: parse_property_datetime(&component.properties[3]),
                name: component.properties[4].val.to_string(),
                description: component.properties[5].val.to_string(),
                categories: component.properties[6].val.to_string(),
                repeat_rule: repeat_rule.clone()
            };

            if repeat_rule.is_some() && event.start_at < Utc::now() {
                let mut next_event = event.clone();
                next_event.start_at = match repeat_rule.unwrap() {
                    RepeatRule::Weekly(day) => {
                        let mut next_date = event.start_at.date_naive();

                        while next_date.weekday() != day || next_date < Utc::now().date_naive() {
                            next_date = next_date.succ_opt().unwrap();
                        }

                        Utc.from_utc_datetime(&next_date.and_hms_opt(event.start_at.hour(), event.start_at.minute(), event.start_at.second()).unwrap())
                    }
                };

                next_event.end_at = next_event.start_at + (event.end_at - event.start_at);
                next_event
            } else {
                event
            }
        })
        .filter(|event| event.start_at > Utc::now())
        .collect();

    Ok(events)
}
