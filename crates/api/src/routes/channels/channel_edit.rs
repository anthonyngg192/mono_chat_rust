use chat_core::{
    models::{
        channel::{FieldsChannel, PartialChannel},
        message::SystemMessage,
        Channel, File, User,
    },
    permissions::defn::ChannelPermission,
    perms,
    util::r#ref::Ref,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditChannel {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,

    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,

    owner: Option<String>,

    #[validate(length(min = 1, max = 128))]
    icon: Option<String>,

    archived: Option<bool>,

    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsChannel>>,
}

#[openapi(tag = "Channel Information")]
#[patch("/<target>", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    target: Ref,
    data: Json<DataEditChannel>,
) -> Result<Json<Channel>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut channel = target.as_channel(db).await?;

    perms(&user)
        .channel(&channel)
        .throw_permission_and_view_channel(db, ChannelPermission::AssignRoles)
        .await?;

    if data.name.is_none()
        && data.description.is_none()
        && data.icon.is_none()
        && data.owner.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(channel));
    }

    let mut partial: PartialChannel = Default::default();

    if let Some(new_owner) = data.owner {
        if let Channel::Group {
            owner, recipients, ..
        } = &mut channel
        {
            if owner != &user.id {
                return Err(Error::NotInGroup);
            }
            if !recipients.contains(&new_owner) {
                return Err(Error::NotInGroup);
            }

            // Transfer ownership
            partial.owner = Some(new_owner.to_string());
            let old_owner = std::mem::replace(owner, new_owner.to_string());

            // Notify clients
            SystemMessage::ChannelOwnershipChanged {
                from: old_owner,
                to: new_owner,
            }
        } else {
            return Err(Error::InvalidOperation);
        }
        .into_message(channel.id().to_string())
        .create(db, &channel, None)
        .await
        .ok();
    }

    match &mut channel {
        Channel::Group {
            id,
            name,
            description,
            icon,
            ..
        }
        | Channel::TextChannel {
            id,
            name,
            description,
            icon,
            ..
        }
        | Channel::VoiceChannel {
            id,
            name,
            description,
            icon,
            ..
        } => {
            if let Some(fields) = &data.remove {
                if fields.contains(&FieldsChannel::Icon) {
                    if let Some(icon) = &icon {
                        db.mark_attachment_as_deleted(&icon.id).await?;
                    }
                }

                for field in fields {
                    match field {
                        FieldsChannel::Description => {
                            description.take();
                        }
                        FieldsChannel::Icon => {
                            icon.take();
                        }
                        _ => {}
                    }
                }
            }

            if let Some(icon_id) = data.icon {
                partial.icon = Some(File::use_icon(db, &icon_id, id).await?);
                *icon = partial.icon.clone();
            }

            if let Some(new_name) = data.name {
                *name = new_name.clone();
                partial.name = Some(new_name);
            }

            if let Some(new_description) = data.description {
                partial.description = Some(new_description);
                *description = partial.description.clone();
            }

            if let Channel::Group { .. } = &channel {
                if let Some(name) = &partial.name {
                    SystemMessage::ChannelRenamed {
                        name: name.to_string(),
                        by: user.id.clone(),
                    }
                    .into_message(channel.id().to_string())
                    .create(db, &channel, None)
                    .await
                    .ok();
                }

                if partial.description.is_some() {
                    SystemMessage::ChannelDescriptionChanged {
                        by: user.id.clone(),
                    }
                    .into_message(channel.id().to_string())
                    .create(db, &channel, None)
                    .await
                    .ok();
                }

                if partial.icon.is_some() {
                    SystemMessage::ChannelIconChanged { by: user.id }
                        .into_message(channel.id().to_string())
                        .create(db, &channel, None)
                        .await
                        .ok();
                }
            }

            channel
                .update(db, partial, data.remove.unwrap_or_default())
                .await?;
        }
        _ => return Err(Error::InvalidOperation),
    };
    Ok(Json(channel))
}
