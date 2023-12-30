use std::time::Duration;

use actix_web::{
    web::{self, Query},
    Responder,
};
use chat_core::types::january::{Embed, Image, ImageSize, Video};
use regex::Regex;
use serde::Deserialize;

use crate::{
    structs::metadata::Metadata,
    util::{
        request::{consume_size, fetch},
        result::Error,
    },
};

lazy_static! {
    static ref CACHE: moka::future::Cache<String, Result<Embed, Error>> =
        moka::future::Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(60))
            .build();
}

#[derive(Deserialize)]
pub struct Parameters {
    url: String,
}

async fn embed(url: String) -> Result<Embed, Error> {
    lazy_static! {
        static ref RE_TWITTER: Regex =
            Regex::new("^(?:https?://)?(?:www\\.)?twitter\\.com").unwrap();
    }

    let (resp, mime) = fetch(&url).await?;

    match (mime.type_(), mime.subtype()) {
        (_, mime::HTML) => {
            let mut metadata = Metadata::from(resp, url).await?;
            metadata.resolve_external().await;

            if metadata.is_none() {
                return Ok(Embed::None);
            }

            Ok(Embed::Website(chat_core::types::january::Metadata::from(
                metadata,
            )))
        }
        (mime::IMAGE, _) => {
            if let Ok((width, height)) = consume_size(resp, mime).await {
                Ok(Embed::Image(Image {
                    url,
                    width,
                    height,
                    size: ImageSize::Large,
                }))
            } else {
                Ok(Embed::None)
            }
        }
        (mime::VIDEO, _) => {
            if let Ok((width, height)) = consume_size(resp, mime).await {
                Ok(Embed::Video(Video { url, width, height }))
            } else {
                Ok(Embed::None)
            }
        }
        _ => Ok(Embed::None),
    }
}

pub async fn get(Query(info): Query<Parameters>) -> Result<impl Responder, Error> {
    let url = info.url;
    let result = CACHE
        .get_with(url.clone(), async { embed(url).await })
        .await;
    result.map(web::Json)
}
