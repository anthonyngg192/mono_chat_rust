use chat_core::variables::delta::{
    APP_URL, AUTUMN_URL, EXTERNAL_WS_URL, HCAPTCHA_SITEKEY, INVITE_ONLY, JANUARY_URL, USE_AUTUMN,
    USE_EMAIL, USE_HCAPTCHA, USE_JANUARY, USE_VOSO, VOSO_URL, VOSO_WS_HOST,
};
use chat_core::Result;

use rocket::serde::json::Json;
use serde::Serialize;

/// # hCaptcha Configuration
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

/// # Voice Server Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct VoiceFeature {
    pub enabled: bool,
    pub url: String,
    pub ws: String,
}

/// # Feature Configuration
#[derive(Serialize, JsonSchema, Debug)]
pub struct RevoltFeatures {
    pub captcha: CaptchaFeature,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Feature,
    pub january: Feature,
    pub voso: VoiceFeature,
}

/// # Build Information
#[derive(Serialize, JsonSchema, Debug)]
pub struct BuildInformation {
    pub commit_sha: String,
    pub commit_timestamp: String,
    pub semver: String,
    pub origin_url: String,
    pub timestamp: String,
}

#[derive(Serialize, JsonSchema, Debug)]
pub struct ChatConfig {
    pub revolt: String,
    pub features: RevoltFeatures,
    pub ws: String,
    pub app: String,
    pub build: BuildInformation,
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
            autumn: Feature {
                enabled: *USE_AUTUMN,
                url: AUTUMN_URL.to_string(),
            },
            january: Feature {
                enabled: *USE_JANUARY,
                url: JANUARY_URL.to_string(),
            },
            voso: VoiceFeature {
                enabled: *USE_VOSO,
                url: VOSO_URL.to_string(),
                ws: VOSO_WS_HOST.to_string(),
            },
        },
        ws: EXTERNAL_WS_URL.to_string(),
        app: APP_URL.to_string(),
        build: BuildInformation {
            commit_sha: option_env!("VERGEN_GIT_SHA")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            commit_timestamp: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            semver: option_env!("VERGEN_GIT_SEMVER")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            origin_url: option_env!("GIT_ORIGIN_URL")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
            timestamp: option_env!("VERGEN_BUILD_TIMESTAMP")
                .unwrap_or_else(|| "<failed to generate>")
                .to_string(),
        },
    }))
}

#[cfg(test)]
#[cfg(feature = "FIXME: THIS TEST CAUSES cargo test TO SEG FAULT, I HAVE NO CLUE HOW")]
mod test {
    use crate::rocket;
    use rocket::http::Status;

    #[rocket::async_test]
    async fn hello_world() {
        let harness = crate::util::test::TestHarness::new().await;
        let response = harness.client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[rocket::async_test]
    async fn hello_world_concurrent() {
        let harness = crate::util::test::TestHarness::new().await;
        let response = harness.client.get("/").dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
}
