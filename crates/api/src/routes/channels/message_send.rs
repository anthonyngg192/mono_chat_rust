use chat_core::{
    models::{
        message::{DataMessageSend, Interactions, MessageAuthor},
        Message, User,
    },
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::{idempotency::IdempotencyKey, reference::Reference},
    Database, Error, Result,
};
use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

#[openapi(tag = "Messaging")]
#[post("/<target>/messages", data = "<data>")]
pub async fn message_send(
    db: &State<Database>,
    user: User,
    target: Reference,
    data: Json<DataMessageSend>,
    idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();

    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let channel = target.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);

    let permissions = calculate_channel_permissions(&mut query).await;
    let _ = permissions.throw_if_lacking_channel_permission(ChannelPermission::SendMessage);

    if let Some(masq) = &data.masquerade {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::Masquerade)?;

        if masq.colour.is_some() {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::ManageRole)?;
        }

        if data.embeds.as_ref().is_some_and(|v| !v.is_empty()) {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::SendEmbeds)?;
        }

        if data.attachments.as_ref().is_some_and(|v| !v.is_empty()) {
            permissions.throw_if_lacking_channel_permission(ChannelPermission::UploadFiles)?;
        }

        if let Some(interactions) = &data.interactions {
            let interactions: Interactions = interactions.clone();
            interactions.validate(db, &permissions).await?;
        }
    }
    let author: User = user.clone().into(db, Some(&user)).await;
    Ok(Json(
        Message::create_from_api(
            db,
            channel,
            data,
            MessageAuthor::User(&author),
            idempotency,
            permissions.has_channel_permission(ChannelPermission::SendEmbeds),
        )
        .await?,
    ))
}
