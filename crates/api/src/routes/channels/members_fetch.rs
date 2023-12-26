use chat_core::{
    models::{Channel, User},
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;

#[openapi(tag = "Groups")]
#[get("/<target>/members")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<User>>> {
    let channel = target.as_channel(db).await?;
    perms(&user)
        .channel(&channel)
        .throw_permission(db, ChannelPermission::ViewChannel)
        .await?;

    if let Channel::Group { recipients, .. } = channel {
        Ok(Json(
            db.fetch_users(&recipients)
                .await?
                .into_iter()
                .map(|x| x.with_relationship(&user))
                .collect::<Vec<User>>(),
        ))
    } else {
        Err(Error::InvalidOperation)
    }
}
