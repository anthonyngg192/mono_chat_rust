use chat_core::{
    models::{message::PartialMessage, User},
    permissions::defn::ChannelPermission,
    perms, Db, EmptyResponse, Ref, Result,
};

#[openapi(tag = "Interactions")]
#[delete("/<target>/messages/<msg>/reactions")]
pub async fn clear_reactions(db: &Db, user: User, target: Ref, msg: Ref) -> Result<EmptyResponse> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::ManageMessages)
        .await?;

    // Fetch relevant message
    let mut message = msg.as_message_in(db, channel.id()).await?;

    // Clear reactions
    message
        .update(
            db,
            PartialMessage {
                reactions: Some(Default::default()),
                ..Default::default()
            },
        )
        .await
        .map(|_| EmptyResponse)
}
