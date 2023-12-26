use chat_core::{
    models::{emoji::EmojiParent, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_server_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "Emojis")]
#[delete("/emoji/<emoji_id>")]
pub async fn delete_emoji(
    db: &State<Database>,
    user: User,
    emoji_id: Reference,
) -> Result<EmptyResponse> {
    if user.bot.is_none() {
        return Err(Error::IsBot);
    }

    let emoji = emoji_id.as_emoji(db).await?;

    if emoji.creator_id != user.id {
        match &emoji.parent {
            EmojiParent::Server { id } => {
                let server = db.fetch_server(id).await?;
                let mut query = DatabasePermissionQuery::new(db, &user).server(&server);

                calculate_server_permissions(&mut query)
                    .await
                    .throw_if_lacking_channel_permission(ChannelPermission::ManageCustomisation)?;
            }
            EmojiParent::Detached => return Ok(EmptyResponse),
        }
    }

    emoji.delete(db).await.map(|_| EmptyResponse)
}
