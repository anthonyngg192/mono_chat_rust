use crate::{
    database::Database,
    models::{user::RelationshipStatus, User},
    permissions::{
        defn::{UserPermissions, UserPerms},
        PermissionCalculator,
    },
    UserPermission,
};

impl PermissionCalculator<'_> {
    pub async fn calc_user(&mut self, db: &Database) -> UserPerms {
        if self.user.has() {
            let v = calculate_permission(self, db).await;
            self.cached_user_permission = Some(v);
            UserPermissions([v])
        } else {
            panic!("Expected `PermissionCalculator.user` to exist.")
        }
    }
}

pub fn get_relationship(a: &User, b: &str) -> RelationshipStatus {
    if a.id == b {
        return RelationshipStatus::User;
    }

    if let relations = &a.relations {
        if let Some(relationship) = relations.iter().find(|x| x.user_id == b) {
            return relationship.status.clone();
        }
    }

    RelationshipStatus::None
}

async fn calculate_permission(data: &mut PermissionCalculator<'_>, db: &crate::Database) -> u32 {
    let user = data.user.get().unwrap();

    if data.perspective.privileged {
        return u32::MAX;
    }

    if data.perspective.id == user.id {
        return u32::MAX;
    }

    let relationship = data
        .flag_known_relationship
        .cloned()
        .unwrap_or_else(|| get_relationship(data.perspective, &user.id));

    let mut permissions: u32 = 0;
    match relationship {
        RelationshipStatus::Friend => return u32::MAX,
        RelationshipStatus::Blocked | RelationshipStatus::BlockedOther => {
            return UserPermission::Access as u32
        }
        RelationshipStatus::Incoming | RelationshipStatus::Outgoing => {
            permissions = UserPermission::Access as u32;
        }
        _ => {}
    }

    if data.flag_has_mutual_connection
        || data
            .perspective
            .has_mutual_connection(db, &user.id)
            .await
            .unwrap_or(false)
    {
        permissions = UserPermission::Access + UserPermission::ViewProfile;

        if user.bot.is_some() || data.perspective.bot.is_some() {
            permissions += UserPermission::SendMessage as u32;
        }

        return permissions;
    }

    permissions
}
