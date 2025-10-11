use std::sync::Arc;

use serde::Deserialize;
use serenity::{
    all::{
        ButtonStyle, CommandDataOptionValue, CommandInteraction, CommandOptionType,
        ComponentInteraction,
    },
    builder::{CreateActionRow, CreateButton, CreateEmbed},
    client::Context,
    model::colour::Color,
};
use tracing::{debug, error};

use crate::{
    handlers::track_play::TrackPlayHandler,
    utils::{
        response::{respond_to_error_button, respond_to_followup},
        track_utils::TrackMetadata,
        type_map::get_http_client,
    },
};
use songbird::{input::YoutubeDl, tracks::Track, Event, TrackEvent};

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    id: String,
    title: String,
    duration: Option<f64>,
    #[serde(default)]
    thumbnails: Vec<Thumbnail>,
}

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer search command: {}", err);
        return;
    }

    let mut response_embed = CreateEmbed::default();

    let command_value = command.data.options.first();

    let resolved_value = match command_value {
        Some(data) => &data.value,
        _ => {
            response_embed = response_embed
                .description("Please provide a search query!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;
            return;
        }
    };

    let query = match resolved_value {
        CommandDataOptionValue::String(value) => value.clone(),
        _ => {
            response_embed = response_embed
                .description("Please provide a valid search query!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;
            return;
        }
    };

    debug!("Searching YouTube for: {}", query);

    // Run yt-dlp to search YouTube
    let output = match tokio::process::Command::new("yt-dlp")
        .args(&[
            "--default-search",
            "ytsearch5",
            "--dump-json",
            "--no-playlist",
            "--flat-playlist",
            &query,
        ])
        .output()
        .await
    {
        Ok(output) => output,
        Err(err) => {
            error!("Failed to execute yt-dlp: {}", err);
            response_embed = response_embed
                .description("Failed to search YouTube. Please try again later.")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;
            return;
        }
    };

    if !output.status.success() {
        error!(
            "yt-dlp command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        response_embed = response_embed
            .description("Failed to search YouTube. Please try again later.")
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
        return;
    }

    // Parse the JSON output - yt-dlp returns one JSON object per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("Stdout from yt-dlp query: {}", stdout);

    let results: Vec<SearchResult> = stdout
        .lines()
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }
            match serde_json::from_str::<SearchResult>(line) {
                Ok(result) => Some(result),
                Err(err) => {
                    error!("Failed to parse search result: {}", err);
                    error!("Line that failed: {}", line);
                    None
                }
            }
        })
        .take(5)
        .collect();

    debug!("Parsed {} results from query", results.len());

    if results.is_empty() {
        response_embed = response_embed
            .description(format!("No results found for \"{}\"", query))
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
        return;
    }

    // Create embeds for each search result
    let embeds: Vec<CreateEmbed> = results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            let duration_str = result
                .duration
                .map(|d| {
                    let total_seconds = d as u64;
                    let minutes = total_seconds / 60;
                    let seconds = total_seconds % 60;
                    format!("{}:{:02}", minutes, seconds)
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let mut embed = CreateEmbed::default()
                .title(format!("{}. {}", idx + 1, result.title))
                .description(format!("Duration: {}", duration_str))
                .url(format!("https://www.youtube.com/watch?v={}", result.id))
                .color(Color::BLUE);

            // Add thumbnail if available (use the last one which is usually highest quality)
            if let Some(thumbnail) = result.thumbnails.last() {
                embed = embed.thumbnail(&thumbnail.url);
            }

            embed
        })
        .collect();

    // Create buttons for each result
    let buttons: Vec<CreateButton> = results
        .iter()
        .enumerate()
        .map(|(idx, result)| {
            CreateButton::new(format!("search_play_{}", result.id))
                .label(format!("Option {}", idx + 1))
                .style(ButtonStyle::Primary)
        })
        .collect();

    // Discord allows up to 5 buttons per action row, we have max 5 results
    let action_rows: Vec<CreateActionRow> = buttons
        .chunks(5)
        .map(|chunk| CreateActionRow::Buttons(chunk.to_vec()))
        .collect();

    if let Err(err) = command
        .edit_response(
            &ctx.http,
            serenity::builder::EditInteractionResponse::new()
                .embeds(embeds)
                .components(action_rows),
        )
        .await
    {
        error!("Failed to send search results: {}", err);
    }
}

pub async fn handle_component(ctx: &Context, interaction: &ComponentInteraction) {
    if let Err(err) = interaction.defer(&ctx.http).await {
        error!("Failed to defer search component interaction: {}", err);
        return;
    }

    // Extract video ID from button custom_id (format: "search_play_{video_id}")
    let video_id = match interaction.data.custom_id.strip_prefix("search_play_") {
        Some(id) => id.to_string(),
        None => {
            error!("Invalid custom_id format: {}", interaction.data.custom_id);
            // Delete the search results message
            if let Err(err) = interaction.delete_response(&ctx.http).await {
                error!("Failed to delete search results message: {}", err);
            }
            respond_to_error_button(interaction, &ctx.http, "Invalid selection!".to_string()).await;
            return;
        }
    };

    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
    debug!("Playing selected video: {}", video_url);

    // Update the message to show we're processing the selection
    let loading_embed = CreateEmbed::default()
        .description("Adding track to queue...")
        .color(Color::BLUE);

    if let Err(err) = interaction
        .edit_response(
            &ctx.http,
            serenity::builder::EditInteractionResponse::new()
                .embeds(vec![loading_embed])
                .components(vec![]), // Remove buttons
        )
        .await
    {
        error!("Failed to update search results message: {}", err);
    }

    let http_client = get_http_client(ctx).await;
    let source = YoutubeDl::new(http_client, video_url);

    // Create a pseudo-command interaction for enqueue_track
    // We need to convert the ComponentInteraction to a CommandInteraction-like structure
    // Since enqueue_track expects a CommandInteraction, we'll need to use the interaction
    // Unfortunately, we can't directly convert, so we'll need to modify our approach

    // Let's directly enqueue without using the helper for now
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => {
            // Delete the loading message before sending error
            if let Err(err) = interaction.delete_response(&ctx.http).await {
                error!("Failed to delete loading message: {}", err);
            }
            respond_to_error_button(
                interaction,
                &ctx.http,
                "This command can only be used in a server!".to_string(),
            )
            .await;
            return;
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;
        let mut source_input: songbird::input::Input = source.into();

        // Get metadata
        let metadata = match source_input.aux_metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                error!("Failed to fetch track metadata: {}", err);
                Default::default()
            }
        };

        let track_title = metadata
            .title
            .clone()
            .unwrap_or_else(|| String::from("Unknown Track Title"));
        let track_thumbnail = metadata.thumbnail.clone();
        let track_duration = metadata.duration.clone();

        // Create custom metadata
        let custom_metadata = Arc::new(TrackMetadata {
            title: track_title.clone(),
            thumbnail_url: track_thumbnail.clone(),
            duration: track_duration,
        });

        // Create and enqueue track
        let track = handler
            .enqueue(Track::new_with_data(source_input, custom_metadata))
            .await;

        let _ = track.add_event(
            Event::Track(TrackEvent::Playable),
            TrackPlayHandler {
                channel_id: interaction.channel_id,
                http: ctx.http.clone(),
                title: track_title.clone(),
                thumbnail: track_thumbnail.clone().unwrap_or_default(),
            },
        );

        // Delete the loading message before sending the success message
        if let Err(err) = interaction.delete_response(&ctx.http).await {
            error!("Failed to delete loading message: {}", err);
        }

        let response_embed = CreateEmbed::default()
            .description(format!("**Queued** {}!", track_title))
            .color(Color::DARK_GREEN);

        if let Err(err) = interaction
            .create_followup(
                &ctx.http,
                serenity::builder::CreateInteractionResponseFollowup::new().embed(response_embed),
            )
            .await
        {
            error!("Failed to send followup message: {}", err);
        }
    } else {
        // Delete the loading message before sending the error message
        if let Err(err) = interaction.delete_response(&ctx.http).await {
            error!("Failed to delete loading message: {}", err);
        }

        let response_embed = CreateEmbed::default()
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        if let Err(err) = interaction
            .create_followup(
                &ctx.http,
                serenity::builder::CreateInteractionResponseFollowup::new().embed(response_embed),
            )
            .await
        {
            error!("Failed to send error message: {}", err);
        }
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("search")
        .description("Search YouTube and choose a video's audio to play")
        .add_option(
            serenity::builder::CreateCommandOption::new(
                CommandOptionType::String,
                "query",
                "Search query",
            )
            .required(true),
        )
}
