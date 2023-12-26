use chat_core::{
    models::{Invite, User},
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;

#[openapi(tag = "Channel Invites")]
#[post("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Invite>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::InviteOthers)
        .await?;

    Invite::create(db, &user, &channel).await.map(Json)
}
