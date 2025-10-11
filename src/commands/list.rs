use serenity::{
    all::{Color, CommandInteraction, CreateEmbed},
    client::Context,
};
use tracing::error;

use crate::utils::{response::respond_to_followup, track_utils::TrackMetadata};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    if let Err(err) = command.defer(&ctx.http).await {
        error!("Failed to defer list command: {}", err);
        return;
    }

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        // Grab the queue and make sure its not empty
        let current_queue = handler.queue().current_queue();
        if current_queue.is_empty() {
            let embed = CreateEmbed::new()
                .description("The queue is **empty!**")
                .color(Color::DARK_GREEN);
            respond_to_followup(command, &ctx.http, embed, false).await;

            return;
        }

        // Transform the Vec of TrackHandles into a Vec of titles
        let queue_titles: Vec<String> = current_queue
            .iter()
            .map(|track| track.data::<TrackMetadata>().title.clone())
            .collect();

        // Build the response description string.
        let response_description = format_queue_description(queue_titles);

        let embed = CreateEmbed::new()
            .description(response_description)
            .color(Color::DARK_GREEN);
        respond_to_followup(command, &ctx.http, embed, false).await;
    } else {
        let embed = CreateEmbed::new()
            .description(
                "Error listing queue! Ensure Poor Jimmy is in a voice channel with **/join**",
            )
            .color(Color::DARK_RED);
        respond_to_followup(command, &ctx.http, embed, false).await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("list").description("Display the current queue of songs")
}

fn format_queue_description(list_of_titles: Vec<String>) -> String {
    let mut description = String::new();

    for (index, title) in list_of_titles.iter().enumerate() {
        description.push_str(format!("**{}:** {}\n", index + 1, title).as_str())
    }

    description
}
