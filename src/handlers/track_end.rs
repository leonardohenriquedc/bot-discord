use std::sync::Arc;

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{colour::Color, prelude::ChannelId},
    prelude::Mutex,
};
use tracing::{debug, error};

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
            debug!("Queue ended in channel {}", self.channel_id);
            // No songs left in the queue, notify the channel
            let embed = CreateEmbed::new()
                .description("Queue has **ended!**")
                .color(Color::DARK_GREEN);

            let message = CreateMessage::new().embed(embed);

            if let Err(err) = self.channel_id.send_message(&self.http, message).await {
                error!("Failed to send queue end notification to channel {}: {}", self.channel_id, err);
            }
        }

        None
    }
}
