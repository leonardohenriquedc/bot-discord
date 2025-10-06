use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::CreateEmbed,
    client::Context,
    model::colour::Color,
};
use tracing::error;

use songbird::input::YoutubeDl;

use crate::utils::{
    response::respond_to_followup, track_utils::enqueue_track, type_map::get_http_client,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer play-url command: {}", err);
        return;
    }

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

    // Validate its a valid Youtube URL
    if !is_valid_youtube_url(&url) {
        response_embed = response_embed
            .description("Please provide a valid **/watch** Youtube URL")
            .color(Color::DARK_RED);

        respond_to_followup(command, &ctx.http, response_embed, false).await;

        return;
    }

    let http_client = get_http_client(ctx).await;

    // Get the audio source for the URL
    let source = YoutubeDl::new(http_client, url);

    enqueue_track(ctx, command, source.into()).await;
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
fn is_valid_youtube_url(url: &String) -> bool {
    (url.contains("youtube.com") && (url.contains("/watch"))) || url.contains("youtu.be")
}

#[cfg(test)]
mod tests {
    use crate::commands::play_url::is_valid_youtube_url;

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
