use chat_core::{
    models::{Invite, User},
    permissions::defn::ChannelPermission,
    perms, Db, Ref, Result,
};
use rocket_empty::EmptyResponse;

#[openapi(tag = "Invites")]
#[delete("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<EmptyResponse> {
    let invite = target.as_invite(db).await?;

    if user.id == invite.creator() {
        db.delete_invite(invite.code()).await
    } else {
        match invite {
            Invite::Server { code, server, .. } => {
                let server = db.fetch_server(&server).await?;

                perms(&user)
                    .server(&server)
                    .throw_permission(db, ChannelPermission::ManageServer)
                    .await?;
                db.delete_invite(&code).await
            }
            _ => unreachable!(),
        }
    }
    .map(|_| EmptyResponse)
}
