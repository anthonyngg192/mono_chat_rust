use chat_core::{
    models::User, permissions::defn::ChannelPermission, perms, Db, EmptyResponse, Ref, Result,
};

#[openapi(tag = "Server Members")]
#[delete("/<server>/bans/<target>")]
pub async fn req(db: &Db, user: User, server: Ref, target: Ref) -> Result<EmptyResponse> {
    let server = server.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, ChannelPermission::BanMembers)
        .await?;

    let ban = target.as_ban(db, &server.id).await?;
    db.delete_ban(&ban.id).await.map(|_| EmptyResponse)
}
