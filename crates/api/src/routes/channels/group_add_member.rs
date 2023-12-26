use chat_core::{
    models::{Channel, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "Groups")]
#[put("/<group_id>/recipients/<member_id>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    group_id: Reference,
    member_id: Reference,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let mut channel = group_id.as_channel(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

    match &channel {
        Channel::Group { .. } => {
            let member = member_id.as_user(db).await?;
            if !user.is_friends_with(&member.id) {
                return Err(Error::NotFriends);
            }

            channel
                .add_user_to_group(db, &member, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
