use once_cell::sync::Lazy;
use std::env;

//Database
pub static MONGODB: Lazy<String> =
    Lazy::new(|| env::var("MONGODB").expect("Missing MONGODB environment variable."));

pub static MONGODB_DATABASE_NAME: Lazy<String> = Lazy::new(|| {
    env::var("MONGODB_DATABASE_NAME").expect("Missing MONGODB_DATABASE_NAME environment variable.")
});

// Application Settings
pub static PUBLIC_URL: Lazy<String> = Lazy::new(|| {
    env::var("MONO_CHAT_PUBLIC_URL").expect("Missing MONO_CHAT_PUBLIC_URL environment variable.")
});

pub static VAPID_PRIVATE_KEY: Lazy<String> = Lazy::new(|| {
    env::var("VAPID_PRIVATE_KEY").expect("Missing VAPID_PRIVATE_KEY environment variable.")
});

pub static VAPID_PUBLIC_KEY: Lazy<String> = Lazy::new(|| {
    env::var("VAPID_PUBLIC_KEY").expect("Missing VAPID_PUBLIC_KEY environment variable.")
});

pub static APP_URL: Lazy<String> = Lazy::new(|| {
    env::var("MONO_CHAT_APP_URL").expect("Missing MONO_CHAT_APP_URL environment variable.")
});

pub static FCM_API_KEY: Lazy<String> =
    Lazy::new(|| env::var("FCM_API_KEY").expect("Missing FCM_API_KEY environment variable."));

pub static EXTERNAL_WS_URL: Lazy<String> = Lazy::new(|| {
    env::var("MONO_CHAT_EXTERNAL_WS_URL")
        .expect("Missing MONO_CHAT_EXTERNAL_WS_URL environment variable.")
});

pub static AUTUMN_URL: Lazy<String> = Lazy::new(|| {
    env::var("AUTUMN_PUBLIC_URL").expect("Missing AUTUMN_PUBLIC_URL environment variable.")
});

pub static JANUARY_URL: Lazy<String> = Lazy::new(|| {
    env::var("JANUARY_PUBLIC_URL").expect("Missing JANUARY_PUBLIC_URL environment variable.")
});

pub static REDIS_URI: Lazy<String> =
    Lazy::new(|| env::var("REDIS_URI").expect("Missing REDIS_URI environment variable."));

pub static JANUARY_CONCURRENT_CONNECTIONS: Lazy<usize> =
    Lazy::new(|| env::var("JANUARY_CONCURRENT_CONNECTIONS").map_or(50, |v| v.parse().unwrap()));

pub static VOSO_URL: Lazy<String> =
    Lazy::new(|| env::var("VOSO_PUBLIC_URL").unwrap_or_else(|_| "https://example.com".to_string()));
pub static VOSO_WS_HOST: Lazy<String> =
    Lazy::new(|| env::var("VOSO_WS_HOST").unwrap_or_else(|_| "wss://example.com".to_string()));
pub static VOSO_MANAGE_TOKEN: Lazy<String> =
    Lazy::new(|| env::var("VOSO_MANAGE_TOKEN").unwrap_or_else(|_| "0".to_string()));

pub static HCAPTCHA_KEY: Lazy<String> = Lazy::new(|| {
    env::var("MONO_CHAT_HCAPTCHA_KEY")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string())
});
pub static HCAPTCHA_SITEKEY: Lazy<String> = Lazy::new(|| {
    env::var("MONO_CHAT_HCAPTCHA_SITEKEY")
        .unwrap_or_else(|_| "10000000-ffff-ffff-ffff-000000000001".to_string())
});

pub static AUTHIFIER_SHIELD_KEY: Lazy<Option<String>> =
    Lazy::new(|| env::var("MONO_CHAT_AUTHIFIER_SHIELD_KEY").ok());

// Application Flags
pub static INVITE_ONLY: Lazy<bool> =
    Lazy::new(|| env::var("MONO_CHAT_INVITE_ONLY").map_or(false, |v| v == "1"));
pub static USE_EMAIL: Lazy<bool> = Lazy::new(|| {
    env::var("MONO_CHAT_USE_EMAIL_VERIFICATION").map_or(
        env::var("MONO_CHAT_SMTP_HOST").is_ok()
            && env::var("MONO_CHAT_SMTP_USERNAME").is_ok()
            && env::var("MONO_CHAT_SMTP_PASSWORD").is_ok()
            && env::var("MONO_CHAT_SMTP_FROM").is_ok(),
        |v| v == *"1",
    )
});
pub static USE_HCAPTCHA: Lazy<bool> = Lazy::new(|| env::var("MONO_CHAT_HCAPTCHA_KEY").is_ok());
pub static USE_AUTUMN: Lazy<bool> = Lazy::new(|| env::var("AUTUMN_PUBLIC_URL").is_ok());
pub static USE_JANUARY: Lazy<bool> = Lazy::new(|| env::var("JANUARY_PUBLIC_URL").is_ok());

// SMTP Settings
pub static SMTP_HOST: Lazy<String> =
    Lazy::new(|| env::var("MONO_CHAT_SMTP_HOST").unwrap_or_else(|_| "".to_string()));
pub static SMTP_USERNAME: Lazy<String> =
    Lazy::new(|| env::var("MONO_CHAT_SMTP_USERNAME").unwrap_or_else(|_| "".to_string()));
pub static SMTP_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var("MONO_CHAT_SMTP_PASSWORD").unwrap_or_else(|_| "".to_string()));
pub static SMTP_FROM: Lazy<String> =
    Lazy::new(|| env::var("MONO_CHAT_SMTP_FROM").unwrap_or_else(|_| "".to_string()));

// Application Logic Settings
pub static MAX_GROUP_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_GROUP_SIZE")
        .unwrap_or_else(|_| "50".to_string())
        .parse()
        .unwrap()
});
pub static MAX_BOT_COUNT: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_BOT_COUNT")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap()
});
pub static MAX_EMBED_COUNT: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_EMBED_COUNT")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap()
});
pub static MAX_SERVER_COUNT: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_SERVER_COUNT")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap()
});
pub static MAX_ROLE_COUNT: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_ROLE_COUNT")
        .unwrap_or_else(|_| "200".to_string())
        .parse()
        .unwrap()
});

pub static MONO_CHAT_MAX_SERVER_EMOJI: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_SERVER_EMOJI")
        .unwrap_or_else(|_| "200".to_string())
        .parse()
        .unwrap()
});

pub static MONO_CHAT_MAX_MESSAGE_REACTS: Lazy<usize> = Lazy::new(|| {
    env::var("MONO_CHAT_MAX_MESSAGE_REACTS")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .unwrap()
});

pub fn preflight_checks() {
    format!("url = {}", *APP_URL);
    format!("public = {}", *PUBLIC_URL);
    format!("external = {}", *EXTERNAL_WS_URL);

    if !(*USE_EMAIL) {
        #[cfg(not(debug_assertions))]
        if !env::var("MONO_CHAT_UNSAFE_NO_EMAIL").map_or(false, |v| v == *"1") {
            panic!("Running in production without email is not recommended, set MONO_CHAT_UNSAFE_NO_EMAIL=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No SMTP settings specified! Remember to configure email.");
    }

    if !(*USE_HCAPTCHA) {
        #[cfg(not(debug_assertions))]
        if !env::var("MONO_CHAT_UNSAFE_NO_CAPTCHA").map_or(false, |v| v == *"1") {
            panic!("Running in production without CAPTCHA is not recommended, set MONO_CHAT_UNSAFE_NO_CAPTCHA=1 to override.");
        }

        #[cfg(debug_assertions)]
        warn!("No Captcha key specified! Remember to add hCaptcha key.");
    }
}

pub static IS_STAGING: Lazy<bool> =
    Lazy::new(|| env::var("MONO_CHAT_IS_STAGING").map_or(false, |v| v == "1"));
