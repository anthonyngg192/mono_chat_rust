use chat_core::{
    models::{Member, User},
    perms, Db, Ref, Result,
};
use rocket::serde::json::Json;

#[openapi(tag = "Server Members")]
#[get("/<target>/members/<member>")]
pub async fn req(db: &Db, user: User, target: Ref, member: Ref) -> Result<Json<Member>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    member.as_member(db, &server.id).await.map(Json)
}
