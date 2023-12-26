use chat_core::{
    models::{Member, User},
    perms, Db, Ref, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema, FromForm)]
pub struct OptionsFetchAllMembers {
    exclude_offline: Option<bool>,
}

#[derive(Serialize, JsonSchema)]
pub struct AllMemberResponse {
    members: Vec<Member>,
    users: Vec<User>,
}

#[openapi(tag = "Server Members")]
#[get("/<target>/members?<options..>")]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: OptionsFetchAllMembers,
) -> Result<Json<AllMemberResponse>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    let mut members = db.fetch_all_members(&server.id).await?;

    let mut user_ids = vec![];
    for member in &members {
        user_ids.push(member.id.user.clone());
    }

    let mut users = User::fetch_foreign_users(db, &user_ids).await?;

    members.sort_by(|a, b| a.id.user.cmp(&b.id.user));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    if let Some(true) = options.exclude_offline {
        let mut iter = users.iter();
        members.retain(|_| iter.next().unwrap().online);
        users.retain(|user| user.online);
    }

    Ok(Json(AllMemberResponse { members, users }))
}
