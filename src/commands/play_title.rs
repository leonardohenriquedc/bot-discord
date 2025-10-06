use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::CreateEmbed,
    client::Context,
    model::colour::Color,
};
use songbird::input::{Compose, YoutubeDl};

use crate::utils::response::respond_to_followup;

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    command.defer(&ctx.http).await.expect(
        "Deferring a command response shouldn't fail. Possible change in API requirements/response",
    );

    let mut response_embed = CreateEmbed::default();

    let command_value = command.data.options.first();

    let resolved_value = match command_value {
        Some(data) => &data.value,
        _ => {
            response_embed = response_embed
                .description("Please provide a title to search!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let title = match resolved_value {
        CommandDataOptionValue::String(value) => value.clone(),
        _ => {
            response_embed = response_embed
                .description("Please provide a valid title!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    play_title(&ctx, &command, title).await;
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("play-title")
        .description("Play the audio from a Youtube video searching by title")
        .add_option(
            serenity::builder::CreateCommandOption::new(
                CommandOptionType::String,
                "title",
                "A Youtube video title",
            )
            .required(true),
        )
}

async fn play_title(ctx: &Context, command: &CommandInteraction, title: String) {
    let mut response_embed = CreateEmbed::default();

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    // Grab the active Call for the command's guild
    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;

        let should_enqueue = match handler.queue().current() {
            Some(_) => true,
            None => false,
        };

        // Get the audio source for the URL
        let source = YoutubeDl::new_search(
            ctx.data
                .read()
                .await
                .get::<crate::utils::type_map::HttpKey>()
                .unwrap()
                .clone(),
            title,
        );

        // Play/enqueue song
        let _track = handler.enqueue_input(source.clone().into());

        let mut input: songbird::input::Input = source.clone().into();

        // Get metadata from the source
        let metadata = input.aux_metadata().await.unwrap_or_default();
        let track_title = metadata
            .title
            .clone()
            .unwrap_or_else(|| String::from("Song"));
        let track_thumbnail = metadata.thumbnail.clone();

        let response_description = format_description(track_title, should_enqueue);

        response_embed = response_embed
            .description(response_description)
            .color(Color::DARK_GREEN);

        if !should_enqueue {
            if let Some(url) = track_thumbnail {
                response_embed = response_embed.image(url);
            }
        }

        respond_to_followup(command, &ctx.http, response_embed, true).await;
    } else {
        response_embed = response_embed
            .description(
                "Error playing song! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;
    }
}

fn format_description(source_title: String, should_enqueue: bool) -> String {
    if should_enqueue {
        return format!("**Queued** {}!", source_title);
    } else {
        return format!("**Playing** {}!", source_title);
    }
}
