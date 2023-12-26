use chat_core::{
    models::User, permissions::defn::ChannelPermission, perms, util::r#ref::Ref, Db, Error, Result,
};
use rocket_empty::EmptyResponse;

#[openapi(tag = "Messaging")]
#[put("/<target>/ack/<message>")]
pub async fn req(db: &Db, user: User, target: Ref, message: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission(db, ChannelPermission::ViewChannel)
        .await?;

    channel
        .ack(&user.id, &message.id)
        .await
        .map(|_| EmptyResponse)
}
