use chat_core::variables::delta::{
    APP_URL, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY, USE_EMAIL, USE_HCAPTCHA,
};
use chat_core::Result;

use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize, JsonSchema, Debug)]
pub struct CaptchaFeature {
    pub enabled: bool,
    pub key: String,
}

#[derive(Serialize, JsonSchema, Debug)]
pub struct Feature {
    pub enabled: bool,
    pub url: String,
}

#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
}

#[derive(Serialize, JsonSchema, Debug)]
pub struct ChatConfig {
    pub revolt: String,
    pub features: RevoltFeatures,
    pub ws: String,
    pub app: String,
}

#[openapi(tag = "Core")]
#[get("/")]
pub async fn root() -> Result<Json<ChatConfig>> {
    Ok(Json(ChatConfig {
        revolt: env!("CARGO_PKG_VERSION").to_string(),
        features: RevoltFeatures {
            captcha: CaptchaFeature {
                enabled: *USE_HCAPTCHA,
                key: HCAPTCHA_SITEKEY.to_string(),
            },
            email: *USE_EMAIL,
            invite_only: *INVITE_ONLY,
        },
        ws: EXTERNAL_WS_URL.to_string(),
        app: APP_URL.to_string(),
    }))
}
