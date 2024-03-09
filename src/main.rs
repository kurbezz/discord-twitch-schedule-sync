use std::time::Duration;

use api::{discord::{compare_events, create_discord_event, delete_discord_event, edit_discord_event, get_discord_events, CreateDiscordEvent, DiscordEvent, NextDate, RecurrenceRule, UpdateDiscordEvent}, twitch::get_twitch_events};
use iso8601_timestamp::Timestamp;
use tokio::time;

pub mod config;
pub mod api;
pub mod utils;


async fn sync() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let twitch_events: Vec<(String, CreateDiscordEvent)> = get_twitch_events()
        .await?
        .iter()
        .map(|e| (e.uid.clone(), e.clone().into()))
        .collect();

    let discord_events: Vec<(String, DiscordEvent)> = get_discord_events()
        .await?
        .into_iter()
        .filter(|e| e.creator_id == config::CONFIG.bot_id)
        .map(|e| (
            e.description.rsplit_once('#').unwrap_or(("", "")).1.to_string(),
            e
        ))
        .collect();

    // Create events
    let to_create: Vec<&CreateDiscordEvent> = twitch_events
        .iter()
        .filter(|(twitch_uid, _)| !discord_events
            .iter()
            .any(|(discord_uid, _)| discord_uid == twitch_uid)
        )
        .map(|(_, e)| e)
        .collect();

    for event in to_create {
        if event.scheduled_start_time <= Timestamp::now_utc() {
            if let Some(rule) = event.recurrence_rule.clone() {
                let mut next_event = event.clone();

                next_event.scheduled_start_time = rule.next_date(event.scheduled_start_time).unwrap();
                next_event.scheduled_end_time = next_event.scheduled_start_time.checked_add(
                    event.scheduled_end_time.duration_since(Timestamp::UNIX_EPOCH) - event.scheduled_start_time.duration_since(Timestamp::UNIX_EPOCH)
                ).unwrap();

                next_event.recurrence_rule = Some(RecurrenceRule {
                    start: next_event.scheduled_start_time,
                    ..next_event.recurrence_rule.unwrap()
                });

                create_discord_event(next_event.clone()).await?;
            }

            continue;
        }

        create_discord_event(event.clone()).await?;
    }

    // Delete events
    let to_delete: Vec<&DiscordEvent> = discord_events
        .iter()
        .filter(|(twitch_uid, _)| !twitch_events
            .iter()
            .any(|(discord_uid, _)| discord_uid == twitch_uid)
        )
        .map(|(_, e)| e.to_owned())
        .collect();

    for event in to_delete {
        delete_discord_event(event.id.clone()).await?;
    }

    // Edit events
    let to_edit: Vec<&(String, DiscordEvent)> = discord_events
        .iter()
        .filter(|(discord_uid, _)| twitch_events
            .iter()
            .any(|(twitch_uid, _)| discord_uid == twitch_uid)
        )
        .collect();

    for (dis_twitch_uid, event) in to_edit {
        let filtered_events: Vec<CreateDiscordEvent> = twitch_events
            .iter()
            .filter(|(twitch_uid, _)| twitch_uid == dis_twitch_uid)
            .map(|(_, e)| e.clone())
            .collect();

        let create_event = match filtered_events.first() {
            Some(e) => e,
            None => continue,
        };

        if compare_events(create_event, event) {
            continue;
        }

        let update_event = {
            let mut new_event = UpdateDiscordEvent {
                name: create_event.name.clone(),
                description: create_event.description.clone(),
                scheduled_start_time: create_event.scheduled_start_time,
                scheduled_end_time: create_event.scheduled_end_time,
                recurrence_rule: create_event.recurrence_rule.clone(),
            };

            if let Some(rule) = new_event.recurrence_rule.clone() {
                new_event.scheduled_start_time = rule.next_date(event.scheduled_start_time).unwrap();
                new_event.scheduled_end_time = new_event.scheduled_start_time.checked_add(
                    event.scheduled_end_time.duration_since(Timestamp::UNIX_EPOCH) - event.scheduled_start_time.duration_since(Timestamp::UNIX_EPOCH)
                ).unwrap();

                new_event.recurrence_rule = Some(RecurrenceRule {
                    start: new_event.scheduled_start_time,
                    ..new_event.recurrence_rule.unwrap()
                });
            }

            new_event
        };

        edit_discord_event(
            event.id.clone(),
            update_event
        ).await?;
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    let mut interval = time::interval(Duration::from_secs(5 * 60));

    loop {
        interval.tick().await;

        match sync().await {
            Ok(_) => print!("Updated!"),
            Err(e) => eprintln!("{}", e),
        }
    }
}
