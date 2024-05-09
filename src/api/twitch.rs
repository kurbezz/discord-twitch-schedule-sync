use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use reqwest;
use icalendar::{self, parser::{Component, Property}};
use crate::config;
use chrono::Weekday;


#[derive(Clone)]
pub enum RepeatRule {
    Weekly(chrono::Weekday)
}


#[derive(Clone)]
pub struct TwitchEvent {
    pub uid: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub categories: Option<String>,
    pub repeat_rule: Option<RepeatRule>
}


impl<'a> From<Component<'a>> for TwitchEvent {
    fn from(value: Component<'a>) -> Self {
        Self {
            uid: TwitchEvent::get_uid(&value),
            start_at: TwitchEvent::get_start_at(&value),
            end_at: TwitchEvent::get_end_at(&value),
            name: TwitchEvent::get_name(&value),
            description: TwitchEvent::get_description(&value),
            categories: TwitchEvent::get_categories(&value),
            repeat_rule: TwitchEvent::get_repeat_rule(&value)
        }
    }
}

impl TwitchEvent {
    fn get_uid(value: &Component) -> String {
        for property in &value.properties {
            if property.name == "UID" {
                return property.val.to_string();
            }
        }

        panic!("UID not found");
    }

    fn get_start_at(value: &Component) -> DateTime<Utc> {
        for property in &value.properties {
            if property.name == "DTSTART" {
                return parse_property_datetime(property);
            }
        }

        panic!("DTSTART not found");
    }

    fn get_end_at(value: &Component) -> DateTime<Utc> {
        for property in &value.properties {
            if property.name == "DTEND" {
                return parse_property_datetime(property);
            }
        }

        panic!("DTEND not found");
    }

    fn get_name(value: &Component) -> String {
        for property in &value.properties {
            if property.name == "SUMMARY" {
                return property.val.to_string();
            }
        }

        panic!("SUMMARY not found");
    }

    fn get_description(value: &Component) -> Option<String> {
        for property in &value.properties {
            if property.name == "DESCRIPTION" {
                return Some(property.val.to_string());
            }
        }

        None
    }

    fn get_categories(value: &Component) -> Option<String> {
        for property in &value.properties {
            if property.name == "CATEGORIES" {
                return Some(property.val.to_string());
            }
        }

        None
    }

    fn get_repeat_rule(value: &Component) -> Option<RepeatRule> {
        for property in &value.properties {
            if property.name == "RRULE" {
                let repeat_rule_str = property.val.to_string();

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

                    return Some(RepeatRule::Weekly(day));
                }

                return None;
            }
        }

        None
    }
}


fn parse_property_datetime(property: &Property) -> DateTime<Utc> {
    let timzone_str = &property.params[0].val.clone().unwrap().to_string()[1..];
    let tz: Tz = timzone_str.parse().unwrap();

    let dt = NaiveDateTime::parse_from_str(
        property.val.as_ref(),
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
            TwitchEvent::from(component)
        })
        .filter(|event| event.start_at > Utc::now() || event.repeat_rule.is_some())
        .collect();

    Ok(events)
}
