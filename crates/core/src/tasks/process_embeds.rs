use crate::database::Database;
use crate::util::variables::delta::{JANUARY_CONCURRENT_CONNECTIONS, JANUARY_URL, MAX_EMBED_COUNT};
use crate::Error;
use crate::{
    models::{message::AppendMessage, Message},
    types::january::Embed,
    Result,
};
use async_lock::Semaphore;
use async_std::task::spawn;
use deadqueue::limited::Queue;
use futures::future::join_all;
use linkify::{LinkFinder, LinkKind};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;

use isahc::prelude::*;
/// Task information
#[derive(Debug)]
struct EmbedTask {
    channel: String,
    id: String,
    content: String,
}

static Q: Lazy<Queue<EmbedTask>> = Lazy::new(|| Queue::new(10_000));

pub async fn queue(channel: String, id: String, content: String) {
    Q.try_push(EmbedTask {
        channel,
        id,
        content,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

pub async fn worker(db: Database) {
    let semaphore = Arc::new(Semaphore::new(*JANUARY_CONCURRENT_CONNECTIONS));

    loop {
        let task = Q.pop().await;
        let db = db.clone();
        let semaphore = semaphore.clone();

        spawn(async move {
            let embeds =
                Embed::generate(task.content, &JANUARY_URL, *MAX_EMBED_COUNT, semaphore).await;

            if let Ok(embeds) = embeds {
                if let Err(err) = Message::append(
                    &db,
                    task.id,
                    task.channel,
                    AppendMessage {
                        embeds: Some(embeds),
                    },
                )
                .await
                {
                    error!("Encountered an error appending to message: {:?}", err);
                }
            }
        });
    }
}
static RE_CODE: Lazy<Regex> = Lazy::new(|| Regex::new("```(?:.|\n)+?```|`(?:.|\n)+?`").unwrap());
static RE_IGNORED: Lazy<Regex> = Lazy::new(|| Regex::new("(<http.+>)").unwrap());

pub async fn generate(
    content: String,
    host: &str,
    max_embeds: usize,
    semaphone: Arc<Semaphore>,
) -> Result<Vec<Embed>> {
    let content = RE_CODE.replace_all(&content, "");

    let content = RE_IGNORED.replace_all(&content, "");

    let content = content
        .split('\n')
        .map(|v| {
            if let Some(c) = v.chars().next() {
                if c == '>' {
                    return "";
                }
            }
            v
        })
        .collect::<Vec<&str>>()
        .join("\n");

    let mut finder = LinkFinder::new();

    finder.kinds(&[LinkKind::Url]);

    let links: Vec<String> = finder
        .links(&content)
        .map(|x| {
            x.as_str()
                .chars()
                .take_while(|&ch| ch != '#')
                .collect::<String>()
        })
        .collect::<HashSet<String>>()
        .into_iter()
        .take(max_embeds)
        .collect();

    if links.is_empty() {
        return Err(Error::LabelMe);
    }

    let mut tasks = Vec::new();

    for link in links {
        let semaphore = semaphone.clone();
        let host = host.to_string();

        tasks.push(spawn(async move {
            let guard = semaphore.acquire().await;

            if let Ok(mut response) = isahc::get_async(format!(
                "{host}/embed>url{}",
                url_escape::encode_component(&link)
            ))
            .await
            {
                drop(guard);
                response.json::<Embed>().await.ok()
            } else {
                None
            }
        }));
    }

    let embeds = join_all(tasks)
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<Embed>>();

    if !embeds.is_empty() {
        Ok(embeds)
    } else {
        Err(Error::LabelMe)
    }
}
