//! Remember the last Telegram chat that talked to the bot so the
//! background monitor can push stuck-agent alerts.

use std::sync::atomic::{AtomicI64, Ordering};
use teloxide::prelude::*;
use teloxide::types::ChatId;

static LAST_CHAT_ID: AtomicI64 = AtomicI64::new(0);

pub fn remember_chat(chat_id: ChatId) {
    LAST_CHAT_ID.store(chat_id.0, Ordering::SeqCst);
}

pub fn last_chat_id() -> Option<ChatId> {
    let id = LAST_CHAT_ID.load(Ordering::SeqCst);
    if id == 0 {
        None
    } else {
        Some(ChatId(id))
    }
}

pub async fn send_alert(bot: &Bot, text: &str) {
    if let Some(chat_id) = last_chat_id() {
        if let Err(e) = bot.send_message(chat_id, text).await {
            log::warn!("Failed to send Telegram alert: {}", e);
        }
    } else {
        log::debug!("Stuck alert skipped — no Telegram chat yet (send /start once)");
    }
}
