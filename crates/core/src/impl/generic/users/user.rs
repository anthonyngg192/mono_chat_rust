use crate::{
    database::Database,
    events::client::EventV1,
    models::{
        user::{FieldsUser, PartialUser, Presence, RelationshipStatus, UserHint},
        User,
    },
    permissions::{
        defn::{UserPermission, UserPerms},
        perms,
        r#impl::{
            permission::{calculate_user_permissions, DatabasePermissionQuery},
            user::get_relationship,
        },
    },
    presence::presence_filter_online,
    Error, Result,
};
use futures::try_join;
use redis_kiss::{get_connection, AsyncCommands};
use ulid::Ulid;

impl User {
    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_user(&self.id, &partial, remove.clone()).await?;

        EventV1::UserUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p_user(self.id.clone(), db)
        .await;

        Ok(())
    }

    pub fn remove(&mut self, field: &FieldsUser) {
        match field {
            FieldsUser::Avatar => self.avatar = None,
            FieldsUser::StatusText => {
                if let Some(x) = self.status.as_mut() {
                    x.text = None;
                }
            }
            FieldsUser::StatusPresence => {
                if let Some(x) = self.status.as_mut() {
                    x.presence = None;
                }
            }
            FieldsUser::ProfileContent => {
                if let Some(x) = self.profile.as_mut() {
                    x.content = None;
                }
            }
            FieldsUser::ProfileBackground => {
                if let Some(x) = self.profile.as_mut() {
                    x.background = None;
                }
            }
        }
    }

    pub async fn has_mutual_connection(&self, db: &Database, user_b: &str) -> Result<bool> {
        Ok(!db
            .fetch_mutual_server_ids(&self.id, user_b)
            .await?
            .is_empty()
            || !db
                .fetch_mutual_channel_ids(&self.id, user_b)
                .await?
                .is_empty())
    }

    /*
    Resolve issue multiple GMT
     */
    #[must_use]
    pub fn foreign(mut self) -> User {
        self.profile = None;
        self.relations = vec![];

        if let Some(status) = &self.status {
            if let Some(presence) = &status.presence {
                if presence == &Presence::Invisible {
                    self.status = None;
                    self.online = false;
                }
            }
        }
        self
    }

    pub fn with_relationship(self, perspective: &User) -> User {
        let mut user = self.foreign();
        user.relationship = get_relationship(perspective, &user.id);
        user
    }

    pub fn relationship_with(&self, user_b: &str) -> RelationshipStatus {
        if self.id == user_b {
            return RelationshipStatus::User;
        }

        let relations = &self.relations;
        if let Some(relationship) = relations.iter().find(|x| x.user_id == user_b) {
            return relationship.status.clone();
        }

        RelationshipStatus::None
    }

    pub async fn fetch_foreign_users(db: &Database, user_ids: &[String]) -> Result<Vec<User>> {
        let online_ids = presence_filter_online(user_ids).await;
        Ok(db
            .fetch_users(user_ids)
            .await?
            .into_iter()
            .map(|mut user| {
                user.online = online_ids.contains(&user.id);
                user.foreign()
            })
            .collect::<Vec<User>>())
    }

    pub async fn mark_deleted(&mut self, db: &Database) -> Result<()> {
        self.update(
            db,
            PartialUser {
                username: Some(format!("Deleted User {}", self.id)),
                flags: Some(2),
                ..Default::default()
            },
            vec![
                FieldsUser::Avatar,
                FieldsUser::StatusText,
                FieldsUser::StatusPresence,
                FieldsUser::ProfileContent,
                FieldsUser::ProfileBackground,
            ],
        )
        .await
    }

    #[must_use]
    pub fn apply_permission(mut self, permission: &UserPerms) -> User {
        if !permission.get_view_profile() {
            self.status = None;
        }

        self
    }

    #[must_use]
    pub fn with_perspective(self, perspective: &User, permission: &UserPerms) -> User {
        self.with_relationship(perspective)
            .apply_permission(permission)
    }

    pub async fn with_auto_perspective(self, db: &Database, perspective: &User) -> User {
        let user = self.with_relationship(perspective);
        let permissions = perms(perspective).user(&user).calc_user(db).await;
        user.apply_permission(&permissions)
    }

    pub async fn can_acquire_server(&self, db: &Database) -> Result<bool> {
        Ok(db.fetch_server_count(&self.id).await? <= 100)
    }

    pub fn is_friends_with(&self, user_b: &str) -> bool {
        matches!(
            self.relationship_with(user_b),
            RelationshipStatus::Friend | RelationshipStatus::User
        )
    }

    pub async fn validate_username(db: &Database, username: String) -> Result<String> {
        let username = username.trim().to_string();
        if username.len() < 2 {
            return Err(Error::InvalidUsername);
        }

        let username_lowercase = username.to_lowercase();

        const BLOCKED_USERNAMES: &[&str] = &["admin", "revolt"];

        for username in BLOCKED_USERNAMES {
            if username_lowercase == *username {
                return Err(Error::InvalidUsername);
            }
        }

        const BLOCKED_SUBSTRINGS: &[&str] = &["```"];

        for substr in BLOCKED_SUBSTRINGS {
            if username_lowercase.contains(substr) {
                return Err(Error::InvalidUsername);
            }
        }

        if db.is_username_taken(&username).await? {
            return Err(Error::UsernameTaken);
        }

        Ok(username)
    }

    pub async fn update_username(&mut self, db: &Database, username: String) -> Result<()> {
        self.update(
            db,
            PartialUser {
                username: Some(User::validate_username(db, username).await?),
                ..Default::default()
            },
            vec![],
        )
        .await
    }

    pub async fn apply_relationship(
        &self,
        db: &Database,
        target: &mut User,
        local: RelationshipStatus,
        remote: RelationshipStatus,
    ) -> Result<()> {
        if try_join!(
            db.set_relationship(&self.id, &target.id, &local),
            db.set_relationship(&target.id, &self.id, &remote)
        )
        .is_err()
        {
            return Err(Error::DatabaseError {
                operation: "update_one",
                with: "user",
            });
        }

        EventV1::UserRelationship {
            id: target.id.clone(),
            user: self.clone().into(db, Some(&*target)).await,
        }
        .private(target.id.clone())
        .await;

        EventV1::UserRelationship {
            id: self.id.clone(),
            user: target.clone().into(db, Some(self)).await,
        }
        .private(self.id.clone())
        .await;

        // target.relationship.replace(local);
        Ok(())
    }

    pub async fn add_friend(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::User => Err(Error::NoEffect),
            RelationshipStatus::Friend => Err(Error::AlreadyFriends),
            RelationshipStatus::Outgoing => Err(Error::AlreadySentRequest),
            RelationshipStatus::Blocked => Err(Error::Blocked),
            RelationshipStatus::BlockedOther => Err(Error::BlockedByOther),
            RelationshipStatus::Incoming => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Friend,
                    RelationshipStatus::Friend,
                )
                .await
            }
            RelationshipStatus::None => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Outgoing,
                    RelationshipStatus::Incoming,
                )
                .await
            }
        }
    }

    pub async fn remove_friend(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::Friend
            | RelationshipStatus::Outgoing
            | RelationshipStatus::Incoming => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::None,
                    RelationshipStatus::None,
                )
                .await
            }
            _ => Err(Error::NoEffect),
        }
    }

    pub async fn block_user(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::User | RelationshipStatus::Blocked => Err(Error::NoEffect),
            RelationshipStatus::BlockedOther => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Blocked,
                    RelationshipStatus::Blocked,
                )
                .await
            }
            RelationshipStatus::None
            | RelationshipStatus::Friend
            | RelationshipStatus::Incoming
            | RelationshipStatus::Outgoing => {
                self.apply_relationship(
                    db,
                    target,
                    RelationshipStatus::Blocked,
                    RelationshipStatus::BlockedOther,
                )
                .await
            }
        }
    }

    pub async fn unblock_user(&self, db: &Database, target: &mut User) -> Result<()> {
        match get_relationship(self, &target.id) {
            RelationshipStatus::Blocked => match get_relationship(target, &self.id) {
                RelationshipStatus::Blocked => {
                    self.apply_relationship(
                        db,
                        target,
                        RelationshipStatus::BlockedOther,
                        RelationshipStatus::Blocked,
                    )
                    .await
                }
                RelationshipStatus::BlockedOther => {
                    self.apply_relationship(
                        db,
                        target,
                        RelationshipStatus::None,
                        RelationshipStatus::None,
                    )
                    .await
                }
                _ => Err(Error::InternalError),
            },
            _ => Err(Error::NoEffect),
        }
    }

    pub fn has_blocked(&self, user: &str) -> bool {
        matches!(
            get_relationship(self, user),
            RelationshipStatus::Blocked | RelationshipStatus::BlockedOther
        )
    }

    #[async_recursion]
    pub async fn from_token(db: &Database, token: &str, hint: UserHint) -> Result<User> {
        match hint {
            UserHint::Bot => db.fetch_user(&db.fetch_bot_by_token(token).await?.id).await,
            UserHint::User => db.fetch_user_by_token(token).await,
            UserHint::Any => {
                if let Ok(user) = User::from_token(db, token, UserHint::User).await {
                    Ok(user)
                } else {
                    User::from_token(db, token, UserHint::Bot).await
                }
            }
        }
    }

    pub async fn create<I, D>(
        // self,
        db: &Database,
        username: String,
        account_id: I,
        data: D,
    ) -> Result<User>
    where
        I: Into<Option<String>>,
        D: Into<Option<PartialUser>>,
    {
        let username = User::validate_username(db, username).await?;
        let mut user = User {
            id: account_id.into().unwrap_or_else(|| Ulid::new().to_string()),
            username,
            ..Default::default()
        };

        if let Some(data) = data.into() {
            user.apply_options(data);
        }

        db.insert_user(&user).await?;
        Ok(user)
    }

    pub async fn into<'a, P>(self, db: &Database, perspective: P) -> User
    where
        P: Into<Option<&'a User>>,
    {
        let perspective = perspective.into();
        let (relationship, can_see_profile) = if self.bot.is_some() {
            (RelationshipStatus::None, true)
        } else if let Some(perspective) = perspective {
            let mut query = DatabasePermissionQuery::new(db, perspective).user(&self);

            if perspective.id == self.id {
                (RelationshipStatus::User, true)
            } else {
                (
                    perspective
                        .relations
                        .clone()
                        .into_iter()
                        .find(|relationship| relationship.user_id == self.id)
                        .map(|relations| relations.status.clone())
                        .unwrap(),
                    calculate_user_permissions(&mut query)
                        .await
                        .has_user_permission(UserPermission::ViewProfile),
                )
            }
        } else {
            (RelationshipStatus::None, false)
        };

        User {
            username: self.username,
            display_name: self.display_name,
            avatar: self.avatar,
            relations: if let Some(User { id, .. }) = perspective {
                if id == &self.id {
                    self.relations
                } else {
                    vec![]
                }
            } else {
                vec![]
            },
            badges: self.badges,
            status: if can_see_profile { self.status } else { None },
            profile: if can_see_profile { self.profile } else { None },
            flags: self.flags,
            privileged: self.privileged,
            bot: self.bot,
            relationship,
            online: can_see_profile && is_online(&self.id).await,
            id: self.id,
        }
    }

    pub async fn into_self(self) -> User {
        User {
            username: self.username,
            display_name: self.display_name,
            avatar: self.avatar,
            relations: self.relations,
            badges: self.badges,
            status: self.status,
            profile: self.profile,
            flags: self.flags,
            privileged: self.privileged,
            bot: self.bot,
            relationship: RelationshipStatus::User,
            online: is_online(&self.id).await,
            id: self.id,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for User {
    fn default() -> Self {
        Self {
            id: Default::default(),
            username: Default::default(),
            display_name: Default::default(),
            avatar: Default::default(),
            relations: Default::default(),
            badges: Default::default(),
            status: Default::default(),
            profile: Default::default(),
            flags: Default::default(),
            privileged: Default::default(),
            bot: Default::default(),
            online: Default::default(),
            relationship: RelationshipStatus::None,
        }
    }
}

pub async fn is_online(user_id: &str) -> bool {
    if let Ok(mut conn) = get_connection().await {
        conn.exists(user_id).await.unwrap_or(false)
    } else {
        false
    }
}
