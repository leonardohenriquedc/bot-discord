use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::CreateEmbed,
    client::Context,
    model::colour::Color,
};

use songbird::input::YoutubeDl;

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
                .description("Please provide a URL to play!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    let url = match resolved_value {
        CommandDataOptionValue::String(value) => value.clone(),
        _ => {
            response_embed = response_embed
                .description("Please provide a valid URL!")
                .color(Color::DARK_RED);

            respond_to_followup(command, &ctx.http, response_embed, false).await;

            return;
        }
    };

    play_url(&ctx, &command, url).await;
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("play-url")
        .description("Play the audio from a Youtube video URL")
        .add_option(
            serenity::builder::CreateCommandOption::new(
                CommandOptionType::String,
                "url",
                "A Youtube video URL",
            )
            .required(true),
        )
}

async fn play_url(ctx: &Context, command: &CommandInteraction, url: String) {
    let mut response_embed = CreateEmbed::default();

    // Validate its a valid Youtube URL
    if !is_valid_youtube_url(&url) {
        response_embed = response_embed
            .description("Please provide a valid **/watch** Youtube URL")
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;

        return;
    }

    // Grab the voice client registered with Serentiy's shard key-value store
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    // Grab the active Call for the command's guild
    if let Some(call) = manager.get(guild_id) {
        let mut handler = call.lock().await;

        // If a song is currently playing, we'll add the new song to the queue
        let should_enqueue = match handler.queue().current() {
            Some(_) => true,
            None => false,
        };

        // Get the audio source for the URL
        let source = YoutubeDl::new(
            ctx.data
                .read()
                .await
                .get::<crate::utils::type_map::HttpKey>()
                .unwrap()
                .clone(),
            url,
        );

        // Play/enqueue song
        let _track = handler.enqueue_input(source.clone().into()).await;

        let mut input: songbird::input::Input = source.into();

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

fn is_valid_youtube_url(url: &String) -> bool {
    (url.contains("youtube.com") && (url.contains("/watch"))) || url.contains("youtu.be")
}

#[cfg(test)]
mod tests {
    use crate::commands::play_url::is_valid_youtube_url;

    use super::format_description;

    #[test]
    fn it_formats_description_queued() {
        let title = String::from("Heat Waves");

        let formatted = format_description(title.clone(), true);
        assert_eq!(format!("**Queued** {}!", title), formatted);
    }

    #[test]
    fn it_formats_description_playing() {
        let title = String::from("Heat Waves");

        let formatted = format_description(title.clone(), false);
        assert_eq!(format!("**Playing** {}!", title), formatted);
    }

    #[test]
    fn it_validates_valid_youtube_urls() {
        let valid_watch_url = String::from("https://www.youtube.com/watch?id=12345");
        let valid_share_url = String::from("https://youtu.be/e7qtC_e8Jxc?si=mtCnq8iVc253P89M");

        assert_eq!(true, is_valid_youtube_url(&valid_watch_url));
        assert_eq!(true, is_valid_youtube_url(&valid_share_url));
    }

    #[test]
    fn it_validates_invalid_youtube_urls() {
        let invalid_url = String::from("https://www.you.tube.com/watch?id=12345");
        let another_invalid_url =
            String::from("https://www.youtube.com/results?search_query=title");

        assert_eq!(false, is_valid_youtube_url(&invalid_url));
        assert_eq!(false, is_valid_youtube_url(&another_invalid_url));
    }
}
