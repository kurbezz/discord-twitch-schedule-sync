use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};

use crate::config;


#[derive(Deserialize)]
pub struct DiscordEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scheduled_start_time: Timestamp,
    pub scheduled_end_time: Timestamp,
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
}

#[derive(Serialize, Clone)]
pub struct UpdateDiscordEvent {
    pub scheduled_start_time: Timestamp,
    pub scheduled_end_time: Timestamp,
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
