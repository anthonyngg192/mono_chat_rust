use chat_core::{
    models::{
        emoji::{DataCreateEmoji, EmojiParent},
        Emoji, File, User,
    },
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_server_permissions, DatabasePermissionQuery},
    },
    variables::delta::MONO_CHAT_MAX_SERVER_EMOJI,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};
use validator::Validate;

#[openapi(tag = "Emojis")]
#[put("/emoji/<id>", data = "<data>")]
pub async fn create_emoji(
    db: &State<Database>,
    user: User,
    id: String,
    data: Json<DataCreateEmoji>,
) -> Result<Json<Emoji>> {
    let data = data.into_inner();

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if user.bot.is_none() {
        return Err(Error::IsBot);
    }

    match &data.parent {
        EmojiParent::Server { id } => {
            let server = db.fetch_server(id).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
            calculate_server_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::ManageCustomisation)?;

            let emojis = db.fetch_emoji_by_parent_id(&server.id).await?;
            if emojis.len() > *MONO_CHAT_MAX_SERVER_EMOJI {
                return Err(Error::TooManyEmoji);
            }
        }
        EmojiParent::Detached => return Err(Error::InvalidOperation),
    }
    let attachment = File::use_emoji(db, &id, &id).await?;

    let emoji = Emoji {
        id,
        parent: data.parent,
        creator_id: user.id,
        name: data.name,
        animated: "image/gif" == &attachment.content_type,
        nsfw: data.nsfw,
    };

    emoji.create(db).await?;
    Ok(Json(emoji))
}
