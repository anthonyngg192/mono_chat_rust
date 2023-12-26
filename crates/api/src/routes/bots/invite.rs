use chat_core::{
    models::{bot::InviteBotDestination, Member, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{
            calculate_channel_permissions, calculate_server_permissions, DatabasePermissionQuery,
        },
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};
use rocket_empty::EmptyResponse;

#[openapi(tag = "Bots")]
#[post("/<target>/invite", data = "<dest>")]
pub async fn invite_bot(
    db: &State<Database>,
    user: User,
    target: Reference,
    dest: Json<InviteBotDestination>,
) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = target.as_bot(db).await?;

    if !bot.public && bot.owner != user.id {
        return Err(Error::BotIsPrivate);
    }

    let bot_user = db.fetch_user(&bot.id).await?;
    match dest.into_inner() {
        InviteBotDestination::Server { server } => {
            let server = db.fetch_server(&server).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
            calculate_server_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::ManageServer)?;

            Member::create(db, &server, &bot_user, None)
                .await
                .map(|_| EmptyResponse)
        }
        InviteBotDestination::Group { group } => {
            let mut channel = db.fetch_channel(&group).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
            calculate_channel_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::InviteOthers)?;

            channel
                .add_user_to_group(db, &bot_user, &user.id)
                .await
                .map(|_| EmptyResponse)
        }
    }
}
