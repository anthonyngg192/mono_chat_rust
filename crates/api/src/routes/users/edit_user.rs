use chat_core::{
    models::{
        user::{FieldsUser, PartialUser, UserStatus},
        File, User,
    },
    util::r#ref::Ref,
    Database, Error, Result,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub static RE_DISPLAY_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^\u200B\n\r]+$").unwrap());

#[derive(Validate, Serialize, Deserialize, Debug, JsonSchema)]
pub struct UserProfileData {
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    background: Option<String>,
}

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataEditUser {
    #[validate(length(min = 2, max = 32), regex = "RE_DISPLAY_NAME")]
    display_name: Option<String>,

    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,

    #[validate]
    status: Option<UserStatus>,

    #[validate]
    profile: Option<UserProfileData>,

    badges: u32,
    flags: u32,

    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsUser>>,
}

#[openapi(tag = "User Information")]
#[patch("/<target>", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    mut user: User,
    target: Ref,
    data: Json<DataEditUser>,
) -> Result<Json<User>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    if target.id != "@me" && target.id != user.id {
        let target_user = target.as_user(db).await?;
        let is_bot_owner = target_user
            .bot
            .map(|bot| bot.owner == user.id)
            .unwrap_or_default();

        if !is_bot_owner && !user.privileged {
            return Err(Error::NotPrivileged);
        }
    }
    if !user.privileged {
        return Err(Error::NotPrivileged);
    }

    if data.display_name.is_none()
        && data.status.is_none()
        && data.profile.is_none()
        && data.avatar.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(user));
    }

    if let Some(fields) = &data.remove {
        if fields.contains(&FieldsUser::Avatar) {
            if let Some(avatar) = &user.avatar {
                db.mark_attachment_as_deleted(&avatar.id).await?;
            }
        }

        if fields.contains(&FieldsUser::ProfileBackground) {
            if let Some(profile) = &user.profile {
                if let Some(background) = &profile.background {
                    db.mark_attachment_as_deleted(&background.id).await?;
                }
            }
        }

        for field in fields {
            user.remove(field);
        }
    }

    let mut partial: PartialUser = PartialUser {
        display_name: data.display_name,
        badges: Some(data.badges),
        flags: Some(data.flags),
        ..Default::default()
    };

    if let Some(avatar) = data.avatar {
        partial.avatar = Some(File::use_avatar(db, &avatar, &user.id).await?);
    }

    if let Some(status) = data.status {
        let mut new_status = user.status.take().unwrap_or_default();
        if let Some(text) = status.text {
            new_status.text = Some(text);
        }

        if let Some(presence) = status.presence {
            new_status.presence = Some(presence);
        }

        partial.status = Some(new_status);
    }

    if let Some(profile) = data.profile {
        let mut new_profile = user.profile.take().unwrap_or_default();
        if let Some(content) = profile.content {
            new_profile.content = Some(content);
        }

        if let Some(background) = profile.background {
            new_profile.background = Some(File::use_background(db, &background, &user.id).await?);
        }

        partial.profile = Some(new_profile);
    }

    user.update(db, partial, data.remove.unwrap_or_default())
        .await?;

    Ok(Json(user.foreign()))
}
