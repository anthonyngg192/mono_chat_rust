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

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct OptionsMessageSearch {
    #[validate(length(min = 1, max = 64))]
    query: String,

    #[validate(range(min = 1, max = 100))]
    limit: Option<i64>,

    #[validate(length(min = 26, max = 26))]
    before: Option<String>,

    #[validate(length(min = 26, max = 26))]
    after: Option<String>,

    #[serde(default = "MessageSort::default")]
    sort: MessageSort,

    include_users: Option<bool>,
}

#[openapi(tag = "Messaging")]
#[post("/<target>/search", data = "<options>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Json<OptionsMessageSearch>,
) -> Result<Json<BulkMessageResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let options = options.into_inner();

    let _ = options
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::ReadMessageHistory)
        .await?;

    let OptionsMessageSearch {
        query,
        limit,
        before,
        after,
        sort,
        include_users,
    } = options;

    let messages = db
        .fetch_messages(MessageQuery {
            filter: MessageFilter {
                channel: Some(channel.id().to_string()),
                query: Some(query),
                ..Default::default()
            },
            time_period: MessageTimePeriod::Absolute {
                before,
                after,
                sort: Some(sort),
            },
            limit,
        })
        .await?;

    BulkMessageResponse::transform(db, Some(&channel), messages, &user, include_users)
        .await
        .map(Json)
}
