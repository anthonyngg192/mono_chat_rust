use chat_core::{models::User, perms, Db, EmptyResponse, Error, Ref, Result};

#[openapi(tag = "Server Information")]
#[put("/<target>/ack")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    db.acknowledge_channels(&user.id, &server.channels)
        .await
        .map(|_| EmptyResponse)
}
