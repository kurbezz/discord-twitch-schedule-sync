use std::time::Duration;

use api::{discord::{create_discord_event, delete_discord_event, get_discord_events, CreateDiscordEvent, DiscordEvent, EntityMetadata}, twitch::get_twitch_events};
use tokio::time;
use utils::convert_to_offset_datetime;

pub mod config;
pub mod api;
pub mod utils;


async fn sync() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let twitch_events: Vec<CreateDiscordEvent> = get_twitch_events()
        .await?
        .iter()
        .map(|e| {
            CreateDiscordEvent {
                name: e.name.clone(),
                description: e.description.clone(),
                privacy_level: 2,
                entity_type: 3,
                entity_metadata: EntityMetadata {
                    location: "https://twitch.tv/hafmc".to_string()
                },
                scheduled_start_time: convert_to_offset_datetime(e.start_at),
                scheduled_end_time: convert_to_offset_datetime(e.end_at)
            }
        })
        .collect();
    let discord_events = get_discord_events().await?;

    // Create events
    let to_create: Vec<&CreateDiscordEvent> = twitch_events
        .iter()
        .filter(|e| {
            let exist = discord_events
                .iter()
                .any(|d_e| {
                    return e.name == d_e.name &&
                        e.description == d_e.description &&
                        e.scheduled_start_time == d_e.scheduled_start_time &&
                        e.scheduled_end_time == d_e.scheduled_end_time;
                });

            return !exist;
        })
        .collect();

    for event in to_create {
        create_discord_event(event.clone()).await?;
    }

    // Delete events
    let to_delete: Vec<&DiscordEvent> = discord_events
        .iter()
        .filter(|d_e| {
            let exist = twitch_events
                .iter()
                .any(|e| {
                    return e.name == d_e.name &&
                        e.description == d_e.description &&
                        e.scheduled_start_time == d_e.scheduled_start_time &&
                        e.scheduled_end_time == d_e.scheduled_end_time;
                });

            return !exist;
        })
        .collect();

    for event in to_delete {
        delete_discord_event(event.id.clone()).await?;
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    let mut interval = time::interval(Duration::from_secs(5 * 60));

    loop {
        match sync().await {
            Ok(_) => print!("Updated!"),
            Err(e) => eprintln!("{}", e),
        }

        interval.tick().await;
    }
}
