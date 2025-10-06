use std::{sync::Arc, time::Duration};

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId},
    prelude::Mutex,
};

use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};
use tokio::time::sleep;

use crate::components::music_buttons::create_music_buttons;

pub struct TrackEndNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub call: Arc<Mutex<Call>>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        // Continue only if this is a Track event
        let EventContext::Track(_) = ctx else {
            return None;
        };

        // Attempt to grab the next song that will be playing
        let handler = self.call.lock().await;
        let queue = handler.queue().current_queue();
        let next_song = queue.first();

        drop(handler);

        // Artificial delay added here before sending message notifying
        // of the next song to play. Often times, this message is sent before
        // the response from other commands making the messages appear
        // out of order. This is a quick/dirty fix for that.
        sleep(Duration::from_secs(2)).await;

        match next_song {
            // A song was found, notify that it will be playing next
            // Note: In Songbird 0.5.0, metadata is not directly accessible from TrackHandle
            // We would need to store it separately when enqueueing tracks
            Some(_song) => {
                let embed = CreateEmbed::new()
                    .description("**Now playing** next track!")
                    .color(Color::DARK_GREEN);

                let message = CreateMessage::new()
                    .embed(embed)
                    .components(create_music_buttons());

                let _ = self.channel_id.send_message(&self.http, message).await;
            }
            // No song was picked up, the queue is most likely done
            None => {
                let embed = CreateEmbed::new()
                    .description("Queue has **ended!**")
                    .color(Color::DARK_GREEN);

                let message = CreateMessage::new().embed(embed);

                let _ = self.channel_id.send_message(&self.http, message).await;
            }
        }

        None
    }
}
