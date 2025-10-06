use serenity::all::{Color, CommandInteraction, Context, CreateEmbed};
use songbird::{input::Input, tracks::Track};
use std::sync::Arc;

use crate::{handlers::track_play::TrackPlayHandler, utils::response::respond_to_followup};

#[derive(Clone)]
pub struct TrackMetadata {
    pub title: String,
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
        let metadata = source.aux_metadata().await.unwrap_or_default();
        let track_title = metadata
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();

        // Create custom metadata to attach to the track
        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
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
        response_embed = response_embed
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    }
}
