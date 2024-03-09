use iso8601_timestamp::{Duration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{config, utils::convert_to_offset_datetime};

use super::twitch::TwitchEvent;


#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct RecurrenceRule {
    pub start: Timestamp,
    pub by_weekday: Option<Vec<u8>>,
    pub interval: Option<u8>,
    pub frequency: Option<u8>,
}

pub trait NextDate {
    fn next_date(&self, start: Timestamp) -> Option<Timestamp>;
}

impl NextDate for RecurrenceRule {
    fn next_date(&self, start: Timestamp) -> Option<Timestamp> {
        let mut next_date = start;

        loop {
            next_date = next_date.checked_add(Duration::seconds(24 * 60 * 60)).unwrap();

            if next_date < Timestamp::now_utc() {
                continue;
            }

            if let Some(ref days) = self.by_weekday {
                if days.contains(&(next_date.date().weekday().number_from_monday() - 1)) {
                    return Some(next_date);
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub struct DiscordEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scheduled_start_time: Timestamp,
    pub scheduled_end_time: Timestamp,
    pub recurrence_rule: Option<RecurrenceRule>,
    pub creator_id: u128,
}

#[derive(Serialize, Clone)]
pub struct EntityMetadata {
    pub location: String,
}

#[derive(Serialize, Clone)]
pub struct CreateDiscordEvent {
    pub name: String,
    pub description: String,
    pub privacy_level: u32,
    pub entity_type: u32,
    pub entity_metadata: EntityMetadata,
    pub scheduled_start_time: Timestamp,
    pub scheduled_end_time: Timestamp,
    pub recurrence_rule: Option<RecurrenceRule>,
}


impl From<TwitchEvent> for CreateDiscordEvent {
    fn from(val: TwitchEvent) -> Self {
        CreateDiscordEvent {
            name: format!("{} | {}", val.name, val.categories),
            description: format!("{}\n\n\n\n#{}", val.description.clone(), val.uid),
            privacy_level: 2,
            entity_type: 3,
            entity_metadata: EntityMetadata {
                location: "https://twitch.tv/hafmc".to_string()
            },
            scheduled_start_time: convert_to_offset_datetime(val.start_at),
            scheduled_end_time: convert_to_offset_datetime(val.end_at),
            recurrence_rule: match val.repeat_rule {
                Some(rule) => {
                    match rule {
                        super::twitch::RepeatRule::Weekly(day) => {
                            Some(RecurrenceRule {
                                start: convert_to_offset_datetime(val.start_at),
                                frequency: Some(2),
                                interval: Some(1),
                                by_weekday: Some(vec![(day.number_from_monday() - 1) as u8]),
                            })
                        },
                    }
                },
                None => None,
            },
        }
    }
}


#[derive(Serialize, Clone)]
pub struct UpdateDiscordEvent {
    pub name: String,
    pub description: String,
    pub scheduled_start_time: Timestamp,
    pub scheduled_end_time: Timestamp,
    pub recurrence_rule: Option<RecurrenceRule>,
}


pub async fn get_discord_events() -> Result<Vec<DiscordEvent>, Box<dyn std::error::Error + Send + Sync>> {
    let events = reqwest::Client::new()
        .get(format!("https://discord.com/api/v10/guilds/{}/scheduled-events", config::CONFIG.guild_id))
        .header("Authorization", format!("Bot {}", config::CONFIG.bot_token))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(events)
}

pub async fn create_discord_event(event: CreateDiscordEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    reqwest::Client::new()
        .post(format!("https://discord.com/api/v10/guilds/{}/scheduled-events", config::CONFIG.guild_id))
        .json(&event)
        .header("Authorization", format!("Bot {}", config::CONFIG.bot_token))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn delete_discord_event(event_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    reqwest::Client::new()
        .delete(format!("https://discord.com/api/v10/guilds/{}/scheduled-events/{event_id}", config::CONFIG.guild_id))
        .header("Authorization", format!("Bot {}", config::CONFIG.bot_token))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn edit_discord_event(event_id: String, event: UpdateDiscordEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    reqwest::Client::new()
        .patch(format!("https://discord.com/api/v10/guilds/{}/scheduled-events/{event_id}", config::CONFIG.guild_id))
        .json(&event)
        .header("Authorization", format!("Bot {}", config::CONFIG.bot_token))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}


// Comparators

pub fn is_repeated(start: Timestamp, target: Timestamp, rule: &RecurrenceRule) -> bool {
    if let Some(ref days) = rule.by_weekday {
        let target_day = target.date().weekday().number_from_monday() - 1;

        return days.contains(&target_day) && start.time() == target.time();
    }

    false
}

pub fn compare_events(e: &CreateDiscordEvent, d_e: &DiscordEvent) -> bool {
    if e.name != d_e.name { return false };
    if e.description != d_e.description { return false };

    match e.recurrence_rule {
        Some(ref rule) => {
            match d_e.recurrence_rule {
                Some(ref d_rule) => {
                    if rule.by_weekday != d_rule.by_weekday { return false };
                    if rule.interval != d_rule.interval { return false };
                    if rule.frequency != d_rule.frequency { return false };
                    if !is_repeated(rule.start, d_rule.start, rule) { return false };
                },
                None => return false,
            }
        },
        None => {
            if d_e.recurrence_rule.is_some() { return false };
        },
    }

    if e.scheduled_start_time != d_e.scheduled_start_time {
        match &e.recurrence_rule {
            Some(rule) => {
                if !is_repeated(e.scheduled_start_time, d_e.scheduled_start_time, rule) {
                    return false;
                }
            },
            None => return false,
        }
    };

    if e.scheduled_end_time != d_e.scheduled_end_time {
        match &e.recurrence_rule {
            Some(rule) => {
                if !is_repeated(e.scheduled_end_time, d_e.scheduled_end_time, rule) {
                    return false;
                }
            },
            None => return false,
        }
    }

    true
}
