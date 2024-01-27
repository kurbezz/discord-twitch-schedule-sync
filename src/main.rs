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
                name: format!("{} | {}", e.name, e.categories),
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
        .filter(|e| !discord_events
                .iter()
                .any(|d_e| e.name == d_e.name && e.description == d_e.description)
        )
        .collect();

    for event in to_create {
        create_discord_event(event.clone()).await?;
    }

    // Delete events
    let to_delete: Vec<&DiscordEvent> = discord_events
        .iter()
        .filter(|d_e| !twitch_events
                .iter()
                .any(|e| e.name == d_e.name && e.description == d_e.description)
        )
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
        interval.tick().await;

        match sync().await {
            Ok(_) => print!("Updated!"),
            Err(e) => eprintln!("{}", e),
        }
    }
}
