use serenity::all::{Color, CommandInteraction, Context, CreateEmbed};
use songbird::{input::Input, tracks::Track};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

use crate::{handlers::track_play::TrackPlayHandler, utils::response::respond_to_followup};

#[derive(Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub duration: Option<Duration>,
}

pub async fn enqueue_track(ctx: &Context, command: &CommandInteraction, mut source: Input) {
    let mut response_embed = CreateEmbed::default();

    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;

        // Get metadata from the source
        debug!("Fetching track metadata for guild {}", guild_id);
        let metadata = match source.aux_metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                warn!("Failed to fetch track metadata: {}. Using defaults.", err);
                Default::default()
            }
        };

        let track_title = metadata
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();
        let track_duration = metadata.duration.clone();

        info!("Enqueueing track: '{}' in guild {}", track_title, guild_id);

        // Create custom metadata to attach to the track
        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
            thumbnail_url: track_thumbnail.clone(),
            duration: track_duration,
        });

        // Create track with attached metadata
        let track_with_data = Track::new_with_data(source, custom_metadata);

        // Play/enqueue song
        let track = handler.enqueue(track_with_data).await;

        let _ = track.add_event(
            songbird::Event::Track(songbird::TrackEvent::Playable),
            TrackPlayHandler {
                channel_id: command.channel_id,
                http: ctx.http.clone(),
                title: track_title.clone(),
                thumbnail: track_thumbnail.clone().unwrap_or_default(),
            },
        );

        let response_description = format!("**Queued** {}!", track_title);

        response_embed = response_embed
            .description(response_description)
            .color(Color::DARK_GREEN);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    } else {
        error!(
            "Bot is not in a voice channel in guild {}. Cannot enqueue track.",
            guild_id
        );

        response_embed = response_embed
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    }
}
