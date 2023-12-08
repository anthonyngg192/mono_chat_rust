use crate::{
    database::Database,
    models::{user::RelationshipStatus, User},
    permissions::{
        defn::{UserPermissions, UserPerms},
        PermissionCalculator,
    },
};

use super::permission::calculate_user_permission;

impl PermissionCalculator<'_> {
    pub async fn calc_user(&mut self, db: &Database) -> UserPerms {
        if self.user.has() {
            let v = calculate_user_permission(self, db).await;
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

    if let Some(relations) = &a.relations {
        if let Some(relationship) = relations.iter().find(|x| x.id == b) {
            return relationship.status.clone();
        }
    }

    RelationshipStatus::None
}
