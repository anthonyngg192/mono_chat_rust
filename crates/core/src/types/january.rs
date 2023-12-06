use std::{collections::HashSet, sync::Arc};

use async_lock::Semaphore;
use async_std::task::spawn;
use futures::future::join_all;
use linkify::{LinkFinder, LinkKind};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{models::media::attachment::File, util::result::Error, util::result::Result};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum ImageSize {
    Large,
    Preview,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Image {
    pub url: String,
    pub width: isize,
    pub height: isize,
    pub size: ImageSize,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Video {
    pub url: String,
    pub width: isize,
    pub height: isize,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Special {
    None,
    GIF,
    YouTube {
        id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    original_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    special: Option<Special>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Image>,

    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<Video>,

    #[serde(skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Embed {
    Website(Metadata),
    Image(Image),
    Video(Video),
    Text(Text),
    None,
}

static RE_CODE: Lazy<Regex> = Lazy::new(|| Regex::new("```(?:.|\n)+?```|`(?:.|\n)+?`").unwrap());
static RE_IGNORED: Lazy<Regex> = Lazy::new(|| Regex::new("(<http.+>)").unwrap());

impl Embed {
    pub async fn generate(
        content: String,
        host: &str,
        max_embeds: usize,
        semaphore: Arc<Semaphore>,
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
        let client = reqwest::Client::new();
        let mut tasks = Vec::new();

        let url = format!("{host}/embed");

        for link in links {
            let client = client.clone();
            let url = url.clone();
            let semaphore = semaphore.clone();

            tasks.push(spawn(async move {
                let guard = semaphore.acquire().await;

                let response = client.get(url).query(&[("url", link)]).send().await.ok()?;
                drop(guard);

                if response.status().is_success() {
                    response.json::<Embed>().await.ok()
                } else {
                    None
                }
            }))
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
}
