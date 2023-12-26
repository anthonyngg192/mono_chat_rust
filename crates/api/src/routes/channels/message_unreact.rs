use chat_core::{models::User, permissions::defn::ChannelPermission, perms, Db, Ref, Result};
use rocket_empty::EmptyResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsUnreact {
    user_id: Option<String>,
    remove_all: Option<bool>,
}

#[openapi(tag = "Interactions")]
#[delete("/<target>/messages/<msg>/reactions/<emoji>?<options..>")]
pub async fn unreact_message(
    db: &Db,
    user: User,
    target: Ref,
    msg: Ref,
    emoji: Ref,
    options: OptionsUnreact,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    let mut permissions = perms(&user).channel(&channel);
    permissions
        .throw_permission_and_view_channel(db, ChannelPermission::React)
        .await?;

    let remove_all = options.remove_all.unwrap_or_default();
    if options.user_id.is_some() || remove_all {
        permissions
            .throw_permission(db, ChannelPermission::ManageMessages)
            .await?;
    }

    let message = msg.as_message_in(db, channel.id()).await?;

    if remove_all {
        return message
            .clear_reaction(db, &emoji.id)
            .await
            .map(|_| EmptyResponse);
    }

    message
        .remove_reaction(db, options.user_id.as_ref().unwrap_or(&user.id), &emoji.id)
        .await
        .map(|_| EmptyResponse)
}
