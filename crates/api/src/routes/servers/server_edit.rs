use std::collections::HashSet;

use chat_core::{
    models::{
        server::{Category, FieldsServer, PartialServer, SystemMessageChannels},
        File, Server, User,
    },
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditServer {
    #[validate(length(min = 1, max = 32))]
    name: Option<String>,

    #[validate(length(min = 0, max = 1024))]
    description: Option<String>,

    icon: Option<String>,
    banner: Option<String>,

    #[validate]
    categories: Option<Vec<Category>>,

    system_messages: Option<SystemMessageChannels>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,

    discoverable: Option<bool>,
    analytics: Option<bool>,

    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsServer>>,
}

#[openapi(tag = "Server Information")]
#[patch("/<target>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataEditServer>,
) -> Result<Json<Server>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = target.as_server(db).await?;
    let mut permissions = perms(&user).server(&server);
    permissions.calc(db).await?;

    if data.name.is_none()
        && data.description.is_none()
        && data.icon.is_none()
        && data.banner.is_none()
        && data.system_messages.is_none()
        && data.categories.is_none()
        && data.flags.is_none()
        && data.analytics.is_none()
        && data.discoverable.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(server));
    } else if data.name.is_some()
        || data.description.is_some()
        || data.icon.is_some()
        || data.banner.is_some()
        || data.system_messages.is_some()
        || data.analytics.is_some()
        || data.remove.is_some()
    {
        permissions
            .throw_permission(db, ChannelPermission::ManageServer)
            .await?;
    }

    if (data.flags.is_some() || data.discoverable.is_some()) && !user.privileged {
        return Err(Error::NotPrivileged);
    }

    if data.categories.is_some() {
        permissions
            .throw_permission(db, ChannelPermission::ManageChannel)
            .await?;
    }

    let DataEditServer {
        name,
        description,
        icon,
        banner,
        categories,
        system_messages,
        flags,
        discoverable,
        analytics,
        remove,
    } = data;

    let mut partial = PartialServer {
        name,
        description,
        categories,
        system_messages,
        flags,
        discoverable,
        analytics,
        ..Default::default()
    };

    if let Some(fields) = &remove {
        if fields.contains(&FieldsServer::Banner) {
            if let Some(banner) = &server.banner {
                db.mark_attachment_as_deleted(&banner.id).await?;
            }
        }

        if fields.contains(&FieldsServer::Icon) {
            if let Some(icon) = &server.icon {
                db.mark_attachment_as_deleted(&icon.id).await?;
            }
        }
    }

    if let Some(system_messages) = &partial.system_messages {
        for id in system_messages.clone().into_channel_ids() {
            if !server.channels.contains(&id) {
                return Err(Error::NotFound);
            }
        }
    }

    if let Some(categories) = &mut partial.categories {
        let mut channel_ids = HashSet::new();
        for category in categories {
            for channel in &category.channels {
                if channel_ids.contains(channel) {
                    return Err(Error::InvalidOperation);
                }

                channel_ids.insert(channel.to_string());
            }

            category
                .channels
                .retain(|item| server.channels.contains(item));
        }
    }

    if let Some(icon) = icon {
        partial.icon = Some(File::use_server_icon(db, &icon, &server.id).await?);
        server.icon = partial.icon.clone();
    }

    if let Some(banner) = banner {
        partial.banner = Some(File::use_banner(db, &banner, &server.id).await?);
        server.banner = partial.banner.clone();
    }

    server
        .update(db, partial, remove.unwrap_or_default())
        .await?;

    Ok(Json(server))
}
