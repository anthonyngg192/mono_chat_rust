use chat_core::{
    models::User, permissions::defn::ChannelPermission, perms, Db, EmptyResponse, Ref, Result,
};

#[openapi(tag = "Interactions")]
#[put("/<target>/messages/<msg>/reactions/<emoji>")]
pub async fn react_message(
    db: &Db,
    user: User,
    target: Ref,
    msg: Ref,
    emoji: Ref,
) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::React)
        .await?;

    // Fetch relevant message
    let message = msg.as_message_in(db, channel.id()).await?;

    // Add the reaction
    message
        .add_reaction(db, &user, &emoji.id)
        .await
        .map(|_| EmptyResponse)
}
