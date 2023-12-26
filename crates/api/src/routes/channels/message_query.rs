use chat_core::{
    models::{
        message::{
            BulkMessageResponse, MessageFilter, MessageQuery, MessageSort, MessageTimePeriod,
        },
        User,
    },
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result,
};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsQueryMessages {
    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,

    #[validate(length(min = 26, max = 26))]
    before: Option<String>,

    #[validate(length(min = 26, max = 26))]
    after: Option<String>,

    sort: Option<MessageSort>,
    #[validate(length(min = 26, max = 26))]
    nearby: Option<String>,

    include_users: Option<bool>,
}

#[openapi(tag = "Messaging")]
#[get("/<target>/messages?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsQueryMessages,
) -> Result<Json<BulkMessageResponse>> {
    let _ = options
        .validate()
        .map_err(|err| Error::FailedValidation { error: err });

    if let Some(MessageSort::Relevance) = options.sort {
        return Err(Error::InvalidOperation);
    }

    let channel = target.as_channel(db).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::ReadMessageHistory)
        .await?;

    let OptionsQueryMessages {
        limit,
        before,
        after,
        sort,
        nearby,
        include_users,
    } = options;
    let messages = db
        .fetch_messages(MessageQuery {
            filter: MessageFilter {
                channel: Some(channel.id().to_string()),
                ..Default::default()
            },
            time_period: if let Some(nearby) = nearby {
                MessageTimePeriod::Relative { nearby }
            } else {
                MessageTimePeriod::Absolute {
                    before,
                    after,
                    sort,
                }
            },
            limit,
        })
        .await?;

    BulkMessageResponse::transform(db, Some(&channel), messages, &user, include_users)
        .await
        .map(Json)
}
