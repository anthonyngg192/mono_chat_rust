use chat_core::{
    models::{Invite, User},
    permissions::defn::ChannelPermission,
    perms, Db, Ref, Result,
};

use rocket::serde::json::Json;

#[openapi(tag = "Server Members")]
#[get("/<target>/invites")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Invite>>> {
    let server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, ChannelPermission::ManageServer)
        .await?;

    db.fetch_invites_for_server(&server.id).await.map(Json)
}
