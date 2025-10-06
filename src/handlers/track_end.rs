use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId},
    prelude::Mutex,
};

use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};

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

        let handler = self.call.lock().await;
        let queue = handler.queue().current_queue();

        if queue.is_empty() {
            // No songs left in the queue, notify the channel
            let embed = CreateEmbed::new()
                .description("Queue has **ended!**")
                .color(Color::DARK_GREEN);

            let message = CreateMessage::new().embed(embed);

            let _ = self.channel_id.send_message(&self.http, message).await;
        }

        None
    }
}
