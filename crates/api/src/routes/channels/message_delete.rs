use chat_core::{
    models::User, permissions::defn::ChannelPermission, perms, Db, EmptyResponse, Error, Ref,
    Result,
};

#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/<msg>", rank = 2)]
pub async fn req(db: &Db, user: User, target: Ref, msg: Ref) -> Result<EmptyResponse> {
    let message = msg.as_message(db).await?;
    if message.channel != target.id {
        return Err(Error::NotFound);
    }

    if message.author != user.id {
        perms(&user)
            .channel(&target.as_channel(db).await?)
            .throw_permission(db, ChannelPermission::ManageMessages)
            .await?;
    }

    message.delete(db).await.map(|_| EmptyResponse)
}
