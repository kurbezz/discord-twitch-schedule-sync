use std::time::Duration;

use api::{discord::{compare_events, create_discord_event, delete_discord_event, get_discord_events, CreateDiscordEvent, DiscordEvent, NextDate, RecurrenceRule}, twitch::get_twitch_events};
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
    let discord_events = get_discord_events().await?;

    // Create events
    let to_create: Vec<&CreateDiscordEvent> = twitch_events
        .iter()
        .filter(|(_, e)| !discord_events
                .iter()
                .any(|d_e| compare_events(e, d_e))
        )
        .map(|(_, e)| e)
        .collect();

    for event in to_create {
        if event.scheduled_start_time <= Timestamp::now_utc() {
            match event.recurrence_rule.clone() {
                Some(rule) => {
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
                },
                None => {},
            }

            continue;
        }

        create_discord_event(event.clone()).await?;
    }

    // Delete events
    let to_delete: Vec<&DiscordEvent> = discord_events
        .iter()
        .filter(|d_e| !twitch_events
                .iter()
                .any(|(_, e)| compare_events(e, d_e))
        )
        .collect();

    for event in to_delete {
        delete_discord_event(event.id.clone()).await?;
    }

    // Edit events
    // let to_edit: Vec<&DiscordEvent> = discord_events
    //     .iter()
    //     .filter(|d_e| twitch_events
    //         .iter()
    //         .any(|e| compare_events(e, d_e))
    //     )
    //     .collect();

    // for event in to_edit {
    //     let filtered_events = twitch_events
    //         .iter()
    //         .filter(|e| compare_events(e, event))
    //         .collect::<Vec<&CreateDiscordEvent>>();

    //     if let Some(twitch_event) = filtered_events.get(0) {
    //         if twitch_event.scheduled_start_time != event.scheduled_start_time || twitch_event.scheduled_end_time != event.scheduled_end_time {
    //             edit_discord_event(
    //                 event.id.clone(),
    //                 UpdateDiscordEvent {
    //                     scheduled_start_time: twitch_event.scheduled_start_time,
    //                     scheduled_end_time: twitch_event.scheduled_end_time
    //                 }
    //             ).await?;
    //         }
    //     }
    // }

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
