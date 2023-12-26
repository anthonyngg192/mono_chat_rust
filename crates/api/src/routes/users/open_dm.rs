use chat_core::{
    models::{Channel, User},
    permissions::defn::UserPermission,
    perms, Database, Error, Ref, Result,
};

use rocket::{serde::json::Json, State};
use ulid::Ulid;

#[openapi(tag = "Direct Messaging")]
#[get("/<target>/dm")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<Channel>> {
    let target = target.as_user(db).await?;

    if target.id == user.id {
        return if let Ok(channel) = db.find_direct_message_channel(&user.id, &target.id).await {
            Ok(Json(channel))
        } else {
            let new_channel = Channel::SavedMessages {
                id: Ulid::new().to_string(),
                user: user.id,
            };

            new_channel.create(db).await?;
            Ok(Json(new_channel))
        };
    }

    if let Ok(channel) = db.find_direct_message_channel(&user.id, &target.id).await {
        Ok(Json(channel))
    } else if perms(&user)
        .user(&target)
        .calc_user(db)
        .await
        .get_send_message()
    {
        let new_channel = Channel::DirectMessage {
            id: Ulid::new().to_string(),
            active: false,
            recipients: vec![user.id, target.id],
            last_message_id: None,
        };

        new_channel.create(db).await?;
        Ok(Json(new_channel))
    } else {
        Error::from_user_permission(UserPermission::SendMessage)
    }
}
