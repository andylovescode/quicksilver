use std::sync::Arc;

use serenity::{
    all::{Context, EventHandler, Message},
    async_trait
};
use tokio::sync::Mutex;

use crate::{
    data::Database,
    data::state::DBEvent::UserSendMessage,
    utils::AntiSpamCount
};

pub struct XPHandler {
    db: Arc<Mutex<Database>>,
}

impl XPHandler {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl EventHandler for XPHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut db = self.db.lock().await;

        let level_before = db.state().get_user_or_default(&msg.author.id).level;

        let _ = db.add(UserSendMessage {
            user: msg.author.id,
            length: msg.content.anti_spam_count(), // Secret Shenanigans
            // note: we do not store the full message 4 privacy
        });

        let level_after = db.state().get_user_or_default(&msg.author.id).level;

        if level_before != level_after {
            let _ = msg.reply_ping(ctx.http, format!("⬆️ Level up from {} to **{}**. {} xp until next level",
                                         level_before,
                                         level_after,
                                         db.state().get_user_or_default(&msg.author.id).xp_until_next_level
            )).await;
        }
    }
}