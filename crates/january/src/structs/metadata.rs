use std::collections::HashMap;

use chat_core::types::january::{Image, ImageSize, Special, Video};
use regex::Regex;
use reqwest::Response;
use scraper::Selector;
use serde::Serialize;
use validator::Validate;

use crate::util::{
    request::{consume_fragment, consume_size, fetch},
    result::Error,
};

#[derive(Clone, Validate, Debug, Serialize)]
pub struct Metadata {
    pub url: String,
    pub original_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub special: Option<Special>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<Video>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub opengraph_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
}

impl Metadata {
    pub async fn from(resp: Response, original_url: String) -> Result<Metadata, Error> {
        let fragment = consume_fragment(resp).await?;

        let metadata_selector = Selector::parse("meta").map_err(|_| Error::MetaSelectionFailed)?;

        let mut meta = HashMap::new();

        for el in fragment.select(&metadata_selector) {
            let node = el.value();

            if let (Some(property), Some(content)) = (
                node.attr("property").or_else(|| node.attr("name")),
                node.attr("content"),
            ) {
                meta.insert(property.to_string(), content.to_string());
            }
        }

        let link_selector = Selector::parse("link").map_err(|_| Error::MetaSelectionFailed)?;
        let mut link = HashMap::new();
        for el in fragment.select(&link_selector) {
            let node = el.value();

            if let (Some(property), Some(content)) = (node.attr("rel"), node.attr("href")) {
                link.insert(property.to_string(), content.to_string());
            }
        }

        let metadata = Metadata {
            title: meta
                .remove("og:title")
                .or_else(|| meta.remove("twitter:title"))
                .or_else(|| meta.remove("title")),
            description: meta
                .remove("og:description")
                .or_else(|| meta.remove("twitter:description"))
                .or_else(|| meta.remove("description")),
            image: meta
                .remove("og:image")
                .or_else(|| meta.remove("og:image:secure_url"))
                .or_else(|| meta.remove("twitter:image"))
                .or_else(|| meta.remove("twitter:image:src"))
                .map(|mut url| {
                    // If relative URL, prepend root URL. Also if root URL ends with a slash, remove it.
                    if let Some(ch) = url.chars().next() {
                        if ch == '/' {
                            url = format!("{}{}", &original_url.trim_end_matches('/'), &url);
                        }
                    }
                    let mut size = ImageSize::Preview;
                    if let Some(card) = meta.remove("twitter:card") {
                        if &card == "summary_large_image" {
                            size = ImageSize::Large;
                        }
                    }
                    Image {
                        url,
                        width: meta
                            .remove("og:image:width")
                            .unwrap_or_else(|| "0".to_string())
                            .parse()
                            .unwrap_or(0),
                        height: meta
                            .remove("og:image:height")
                            .unwrap_or_else(|| "0".to_string())
                            .parse()
                            .unwrap_or(0),
                        size,
                    }
                }),
            video: meta
                .remove("og:video")
                .or_else(|| meta.remove("og:video:url"))
                .or_else(|| meta.remove("og:video:secure_url"))
                .map(|mut url| {
                    // If relative URL, prepend root URL. Also if root URL ends with a slash, remove it.
                    if let Some(ch) = url.chars().next() {
                        if ch == '/' {
                            url = format!("{}{}", &original_url.trim_end_matches('/'), &url);
                        }
                    }
                    Video {
                        url,
                        width: meta
                            .remove("og:video:width")
                            .unwrap_or_else(|| "0".to_string())
                            .parse()
                            .unwrap_or(0),
                        height: meta
                            .remove("og:video:height")
                            .unwrap_or_else(|| "0".to_string())
                            .parse()
                            .unwrap_or(0),
                    }
                }),
            icon_url: link
                .remove("apple-touch-icon")
                .or_else(|| link.remove("icon"))
                .map(|mut v| {
                    // If relative URL, prepend root URL.
                    if let Some(ch) = v.chars().next() {
                        if ch == '/' {
                            v = format!("{}{}", &original_url.trim_end_matches('/'), v);
                        }
                    }

                    v
                }),
            colour: meta.remove("theme-color"),
            opengraph_type: meta.remove("og:type"),
            site_name: meta.remove("og:site_name"),
            url: meta
                .remove("og:url")
                .unwrap_or_else(|| original_url.clone()),
            original_url,
            special: None,
        };
        metadata
            .validate()
            .map_err(|error| Error::FailedValidation { error })?;

        Ok(metadata)
    }

    async fn resolve_image(&mut self) -> Result<(), Error> {
        if let Some(image) = &mut self.image {
            if image.width != 0 && image.height != 0 {
                return Ok(());
            }
            let (resp, mime) = fetch(&image.url).await?;
            let (width, height) = consume_size(resp, mime).await?;

            image.width = width;
            image.height = height;
        }
        Ok(())
    }

    pub async fn generate_special(&mut self) -> Result<Special, Error> {
        lazy_static! {
            static ref RE_YOUTUBE: Regex = Regex::new("^(?:(?:https?:)?//)?(?:(?:www|m)\\.)?(?:(?:youtube\\.com|youtu.be))(?:/(?:[\\w\\-]+\\?v=|embed/|v/)?)([\\w\\-]+)(?:\\S+)?$").unwrap();
            static ref RE_GIF: Regex = Regex::new("^(?:https?://)?(www\\.)?(gifbox\\.me/view|yiffbox\\.me/view|tenor\\.com/view|giphy\\.com/gifs|gfycat\\.com|redgifs\\.com/watch)/[\\w\\d-]+").unwrap();
        }

        if let Some(captures) = RE_YOUTUBE.captures_iter(&self.url).next() {
            lazy_static! {
                static ref RE_TIMESTAMP: Regex =
                    Regex::new("(?:\\?|&)(?:t|start)=([\\w]+)").unwrap();
            }

            if let Some(video) = &self.video {
                if let Some(timestamp_captures) = RE_TIMESTAMP.captures_iter(&video.url).next() {
                    return Ok(Special::YouTube {
                        id: captures[1].to_string(),
                        timestamp: Some(timestamp_captures[1].to_string()),
                    });
                }

                return Ok(Special::YouTube {
                    id: captures[1].to_string(),
                    timestamp: None,
                });
            }
        } else if RE_GIF.is_match(&self.original_url) {
            return Ok(Special::GIF);
        }

        Ok(Special::None)
    }

    pub async fn resolve_external(&mut self) {
        if let Ok(special) = self.generate_special().await {
            self.colour = Some("#FF424F".to_string());
            self.special = Some(special);
        }

        if self.resolve_image().await.is_err() {
            self.image = None;
        }
    }

    pub fn is_none(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.image.is_none()
            && self.video.is_none()
    }
}

impl From<Metadata> for chat_core::types::january::Metadata {
    fn from(value: Metadata) -> Self {
        chat_core::types::january::Metadata {
            url: Some(value.url),
            original_url: Some(value.original_url),
            special: value.special,
            title: value.title,
            description: value.description,
            image: value.image,
            video: value.video,
            site_name: value.site_name,
            icon_url: value.icon_url,
            colour: value.colour,
        }
    }
}
