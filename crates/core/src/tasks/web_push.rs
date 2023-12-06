use crate::util::variables::delta::VAPID_PRIVATE_KEY;
use authifier::Database;
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;
use web_push::{
    ContentEncoding, SubscriptionInfo, SubscriptionKeys, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

#[derive(Debug)]
struct PushTask {
    recipients: Vec<String>,
    payload: String,
}

static Q: Lazy<Queue<PushTask>> = Lazy::new(|| Queue::new(10_000));

pub async fn queue(recipients: Vec<String>, payload: String) {
    if recipients.is_empty() {
        return;
    }

    Q.try_push(PushTask {
        recipients,
        payload,
    })
    .ok();

    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

pub async fn worker(db: Database) {
    let client = WebPushClient::new();
    let key = base64::decode_config(VAPID_PRIVATE_KEY.clone(), base64::URL_SAFE)
        .expect("valid `VAPID_PRIVATE_KEY`");

    loop {
        let task = Q.pop().await;

        if let Ok(sessions) = db.find_sessions_with_subscription(&task.recipients).await {
            for session in sessions {
                if let Some(sub) = session.subscription {
                    let subscription = SubscriptionInfo {
                        endpoint: sub.endpoint,
                        keys: SubscriptionKeys {
                            auth: sub.auth,
                            p256dh: sub.p256dh,
                        },
                    };

                    let mut builder = WebPushMessageBuilder::new(&subscription).unwrap();

                    match VapidSignatureBuilder::from_pem(std::io::Cursor::new(&key), &subscription)
                    {
                        Ok(sig_builder) => match sig_builder.build() {
                            Ok(signature) => {
                                builder.set_vapid_signature(signature);
                                builder
                                    .set_payload(ContentEncoding::AesGcm, task.payload.as_bytes());

                                match builder.build() {
                                    Ok(msg) => match client.send(msg).await {
                                        Ok(_) => {
                                            info!("Sent Web Push notification to {:?}.", session.id)
                                        }
                                        Err(err) => {
                                            error!("Hit error sending Web Push! {:?}", err)
                                        }
                                    },
                                    Err(err) => {
                                        error!(
                                            "Failed to build message for {}! {:?}",
                                            session.user_id, err
                                        )
                                    }
                                }
                            }
                            Err(err) => error!(
                                "Failed to build signature for {}! {:?}",
                                session.user_id, err
                            ),
                        },
                        Err(err) => error!(
                            "Failed to create signature builder for {}! {:?}",
                            session.user_id, err
                        ),
                    }
                }
            }
        }
    }
}
