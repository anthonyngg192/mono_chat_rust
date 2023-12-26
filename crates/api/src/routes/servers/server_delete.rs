use chat_core::{
    models::{server_member::RemovalIntention, User},
    Db, EmptyResponse, Ref, Result,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema, FromForm)]
pub struct OptionsServerDelete {
    leave_silently: Option<bool>,
}

#[openapi(tag = "Server Information")]
#[delete("/<target>?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsServerDelete,
) -> Result<EmptyResponse> {
    let server = target.as_server(db).await?;
    let member = db.fetch_member(&target.id, &user.id).await?;

    if server.owner == user.id {
        server.delete(db).await
    } else {
        server
            .remove_member(
                db,
                member,
                RemovalIntention::Leave,
                options.leave_silently.unwrap_or_default(),
            )
            .await
    }
    .map(|_| EmptyResponse)
}
