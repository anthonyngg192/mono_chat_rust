use chat_core::{
    models::{channel::Webhook, webhook::CreateWebhookBody, Channel, User},
    permissions::{
        defn::{ChannelPermission, DEFAULT_WEBHOOK_PERMISSIONS},
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};
use ulid::Ulid;
use validator::Validate;

#[openapi(tag = "Webhooks")]
#[post("/<target>/webhooks", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    target: Reference,
    data: Json<CreateWebhookBody>,
) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    let channel = target.as_channel(db).await?;

    if !matches!(channel, Channel::TextChannel { .. } | Channel::Group { .. }) {
        return Err(Error::InvalidOperation);
    }

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageWebhooks)?;

    let webhook_id = Ulid::new().to_string();

    let avatar = match &data.avatar {
        Some(id) => Some(
            db.find_and_use_attachment(id, "avatars", "user", &webhook_id)
                .await?,
        ),
        None => None,
    };

    let webhook = Webhook {
        id: webhook_id,
        name: data.name,
        avatar,
        channel_id: channel.id().to_string(),
        permissions: *DEFAULT_WEBHOOK_PERMISSIONS,
        token: Some(nanoid::nanoid!(64)),
    };

    webhook.create(db).await?;

    Ok(Json(webhook))
}
