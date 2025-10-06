use serenity::all::{Command, Interaction, Ready};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::gateway::ActivityData;
use serenity::model::user::OnlineStatus;

use crate::commands;
use crate::utils::response::{respond_to_error, respond_to_error_button};

/// The primary handler for the bot that handles all
/// the events for the client
pub struct BotEventHandler;

#[async_trait]
impl EventHandler for BotEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let command_name = command.data.name.as_str();

            match command_name {
                "clear" => commands::clear::run(&ctx, &command).await,
                "help" => commands::help::run(&ctx, &command).await,
                "join" => commands::join::run(&ctx, &command).await,
                "leave" => commands::leave::run(&ctx, &command).await,
                "list" => commands::list::run(&ctx, &command).await,
                "loop" => commands::r#loop::run(&ctx, &command).await,
                "pause" => commands::pause::run(&ctx, &command).await,
                "ping" => commands::ping::run(&ctx, &command).await,
                "play-title" => commands::play_title::run(&ctx, &command).await,
                "play-url" => commands::play_url::run(&ctx, &command).await,
                "skip" => commands::skip::run(&ctx, &command).await,
                "resume" => commands::resume::run(&ctx, &command).await,
                _ => {
                    respond_to_error(&command, &ctx.http, format!("Unknown command!")).await;
                }
            };
        } else if let Interaction::Component(command) = interaction {
            let button_id = command.data.custom_id.as_str();

            match button_id {
                "clear" => commands::clear::handle_button(&ctx, &command).await,
                "loop" => commands::r#loop::handle_button(&ctx, &command).await,
                "pause" => commands::pause::handle_button(&ctx, &command).await,
                "resume" => commands::resume::handle_button(&ctx, &command).await,
                "skip" => commands::skip::handle_button(&ctx, &command).await,
                _ => {
                    respond_to_error_button(&command, &ctx.http, format!("Unknown command!")).await;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = vec![
            commands::clear::register(),
            commands::help::register(),
            commands::join::register(),
            commands::leave::register(),
            commands::list::register(),
            commands::r#loop::register(),
            commands::pause::register(),
            commands::ping::register(),
            commands::play_title::register(),
            commands::play_url::register(),
            commands::resume::register(),
            commands::skip::register(),
        ];

        Command::set_global_commands(&ctx.http, commands)
            .await
            .expect("Failed to register slash commands!");

        ctx.set_presence(Some(ActivityData::listening("/play")), OnlineStatus::Online);
    }
}
