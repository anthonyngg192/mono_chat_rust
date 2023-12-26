use chat_core::{
    models::{Channel, User},
    permissions::defn::ChannelPermission,
    util::r#ref::Ref,
    Db, Error, Result,
};
use rocket_empty::EmptyResponse;

#[openapi(tag = "Groups")]
#[delete("/<target>/recipients/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let channel = target.as_channel(db).await?;

    match &channel {
        Channel::Group {
            owner, recipients, ..
        } => {
            if &user.id != owner {
                return Error::from_permission(ChannelPermission::ManageChannel);
            }

            let member = member.as_user(db).await?;
            if user.id == member.id {
                return Err(Error::CannotRemoveYourself);
            }

            if !recipients.iter().any(|x| *x == member.id) {
                return Err(Error::NotInGroup);
            }

            channel
                .remove_user_from_group(db, &member.id, Some(&user.id), false)
                .await
                .map(|_| EmptyResponse)
        }
        _ => Err(Error::InvalidOperation),
    }
}
