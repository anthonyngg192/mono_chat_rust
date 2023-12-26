use chat_core::{
    models::User, permissions::defn::ChannelPermission, perms, Db, EmptyResponse, Error, Ref,
    Result,
};

#[openapi(tag = "Server Permissions")]
#[delete("/<target>/roles/<role_id>")]
pub async fn req(db: &Db, user: User, target: Ref, role_id: String) -> Result<EmptyResponse> {
    let mut server = target.as_server(db).await?;
    let mut permissions = perms(&user).server(&server);

    permissions
        .throw_permission(db, ChannelPermission::ManageRole)
        .await?;

    let member_rank = permissions.get_member_rank().unwrap_or(i64::MIN);

    if let Some(role) = server.roles.remove(&role_id) {
        if role.rank <= member_rank {
            return Err(Error::NotElevated);
        }

        role.delete(db, &server.id, &role_id)
            .await
            .map(|_| EmptyResponse)
    } else {
        Err(Error::NotFound)
    }
}
