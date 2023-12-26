use std::time::SystemTime;

use crate::{
    models::{message::MessageAuthor, Message},
    sys_config::config,
    variables::delta::{APP_URL, AUTUMN_URL, PUBLIC_URL},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushNotification {
    pub author: String,
    pub icon: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub body: String,
    pub tag: String,
    pub timestamp: u64,
    pub url: String,
}

impl PushNotification {
    pub fn new(msg: Message, author: Option<MessageAuthor<'_>>, channel_id: &str) -> Self {
        let icon = if let Some(author) = &author {
            if let Some(avatar) = &author.avatar() {
                format!("{}/avatars/{}", &*AUTUMN_URL, avatar)
            } else {
                format!("{}/users/{}/default_avatar", &*PUBLIC_URL, msg.author)
            }
        } else {
            format!("{}/assets/logo.png", &*APP_URL)
        };

        let image = msg.attachments.and_then(|attachments| {
            attachments
                .first()
                .map(|v| format!("{}/attachments/{}", &*AUTUMN_URL, v.id))
        });

        let body = if let Some(sys) = msg.system {
            sys.into()
        } else if let Some(text) = msg.content {
            text
        } else {
            "Empty Message".to_string()
        };

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            author: author
                .map(|x| x.username().to_string())
                .unwrap_or_else(|| "Revolt".to_string()),
            icon,
            image,
            body,
            tag: channel_id.to_string(),
            timestamp,
            url: format!("{}/channel/{}/{}", &*APP_URL, channel_id, msg.id),
        }
    }

    pub async fn from(msg: Message, author: Option<MessageAuthor<'_>>, channel_id: &str) -> Self {
        let config = config().await;

        let icon = if let Some(author) = &author {
            if let Some(avatar) = author.avatar() {
                format!("{}/avatars/{}", config.hosts.autumn, avatar)
            } else {
                format!("{}/users/{}/default_avatar", config.hosts.api, author.id())
            }
        } else {
            format!("{}/assets/logo.png", config.hosts.app)
        };

        let image = msg.attachments.and_then(|attachments| {
            attachments
                .first()
                .map(|v| format!("{}/attachments/{}", config.hosts.autumn, v.id))
        });

        let body = if let Some(sys) = msg.system {
            sys.into()
        } else if let Some(text) = msg.content {
            text
        } else {
            "Empty Message".to_string()
        };

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            author: author
                .map(|x| x.username().to_string())
                .unwrap_or_else(|| "Revolt".to_string()),
            icon,
            image,
            body,
            tag: channel_id.to_string(),
            timestamp,
            url: format!("{}/channel/{}/{}", config.hosts.app, channel_id, msg.id),
        }
    }
}
