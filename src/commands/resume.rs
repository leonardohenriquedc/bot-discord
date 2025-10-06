use serenity::{
    all::{CommandInteraction, ComponentInteraction},
    client::Context,
};
use songbird::tracks::PlayMode;

use crate::utils::response::{
    respond_to_button, respond_to_command, respond_to_error, respond_to_error_button,
};

pub async fn run(ctx: &Context, command: &CommandInteraction) {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let current_song = handler.queue().current();

        // Attempt to grab the current play state of the current song
        let song_state = match &current_song {
            Some(track) => match track.get_info().await {
                Ok(state) => state.playing,
                Err(why) => {
                    println!("Error getting song state: {why}");

                    respond_to_error(command, &ctx.http, format!("Error resuming song!")).await;

                    return;
                }
            },
            None => {
                respond_to_command(
                    command,
                    &ctx.http,
                    format!("There is no song to resume!"),
                    false,
                )
                .await;

                return;
            }
        };

        // If the song is paused, resume it
        match song_state {
            PlayMode::Pause => match current_song {
                Some(song) => match song.play() {
                    Ok(_) => {
                        respond_to_command(command, &ctx.http, format!("Song **resumed!**"), false)
                            .await;
                    }
                    Err(why) => {
                        println!("Error resuming song: {why}");

                        respond_to_error(command, &ctx.http, format!("Error resuming song!")).await;
                    }
                },
                None => {
                    respond_to_command(
                        command,
                        &ctx.http,
                        format!("There is nothing to resume!"),
                        false,
                    )
                    .await;
                }
            },
            _ => {
                respond_to_command(
                    command,
                    &ctx.http,
                    format!("The song is currently playing!"),
                    false,
                )
                .await;
            }
        };
    } else {
        respond_to_error(
            command,
            &ctx.http,
            format!("Error resuming song! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub async fn handle_button(ctx: &Context, command: &ComponentInteraction) {
    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialization.");

    let guild_id = command.guild_id.unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        let current_song = handler.queue().current();

        // Attempt to grab the current play state of the current song
        let song_state = match &current_song {
            Some(track) => match track.get_info().await {
                Ok(state) => state.playing,
                Err(why) => {
                    println!("Error getting song state: {why}");

                    respond_to_error_button(command, &ctx.http, format!("Error resuming song!"))
                        .await;

                    return;
                }
            },
            None => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("There is no song to resume!"),
                    false,
                )
                .await;

                return;
            }
        };

        // If the song is paused, resume it
        match song_state {
            PlayMode::Pause => match current_song {
                Some(song) => match song.play() {
                    Ok(_) => {
                        respond_to_button(command, &ctx.http, format!("Song **resumed!**"), false)
                            .await;
                    }
                    Err(why) => {
                        println!("Error resuming song: {why}");

                        respond_to_error_button(
                            command,
                            &ctx.http,
                            format!("Error resuming song!"),
                        )
                        .await;
                    }
                },
                None => {
                    respond_to_button(
                        command,
                        &ctx.http,
                        format!("There is nothing to resume!"),
                        false,
                    )
                    .await;
                }
            },
            _ => {
                respond_to_button(
                    command,
                    &ctx.http,
                    format!("The song is currently playing!"),
                    false,
                )
                .await;
            }
        };
    } else {
        respond_to_error_button(
            command,
            &ctx.http,
            format!("Error resuming song! Ensure Poor Jimmy is in a voice channel with **/join**"),
        )
        .await;
    }
}

pub fn register() -> serenity::builder::CreateCommand {
    serenity::builder::CreateCommand::new("resume").description("Resume the currently paused song")
}
