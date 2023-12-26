use nanoid::nanoid;
use ulid::Ulid;

use crate::{
    database::Database,
    models::{
        bot::{FieldsBot, PartialBot, PublicBot},
        user::{BotInformation, PartialUser},
        Bot, User,
    },
    sys_config::config,
    Error, Result,
};

impl Bot {
    pub fn remove(&mut self, field: &FieldsBot) {
        match field {
            FieldsBot::Token => self.token = nanoid!(64),
            FieldsBot::InteractionsUrl => {
                self.interactions_url.take();
            }
        }
    }

    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.fetch_user(&self.id).await?.mark_deleted(db).await?;
        db.delete_bot(&self.id).await
    }

    pub fn into_public_bot(self, user: User) -> PublicBot {
        #[cfg(debug_assertions)]
        assert_eq!(self.id, user.id);

        PublicBot {
            id: self.id,
            username: user.username,
            avatar: user.avatar.map(|x| x.id).unwrap_or_default(),
            description: user
                .profile
                .map(|profile| profile.content.unwrap())
                .unwrap_or_default(),
        }
    }

    pub async fn create<D>(db: &Database, username: String, owner: &User, data: D) -> Result<Bot>
    where
        D: Into<Option<PartialBot>>,
    {
        if owner.bot.is_some() {
            return Err(Error::IsBot);
        }

        let config = config().await;
        if db.get_number_of_bots_by_user(&owner.id).await? >= config.features.limits.default.bots {
            return Err(Error::ReachedMaximumBots);
        }

        let id = Ulid::new().to_string();

        let _ = User::create(
            db,
            username,
            Some(id.to_string()),
            Some(PartialUser {
                bot: Some(BotInformation {
                    owner: id.to_string(),
                }),
                ..Default::default()
            }),
        )
        .await;

        let mut bot = Bot {
            id,
            owner: owner.id.to_string(),
            token: nanoid::nanoid!(64),
            ..Default::default()
        };

        if let Some(data) = data.into() {
            bot.apply_options(data);
        }

        db.insert_bot(&bot).await?;
        Ok(bot)
    }

    pub async fn update(
        &mut self,
        db: &Database,
        mut partial: PartialBot,
        remove: Vec<FieldsBot>,
    ) -> Result<()> {
        if remove.contains(&FieldsBot::Token) {
            partial.token = Some(nanoid::nanoid!(64));
        }

        for field in &remove {
            self.remove_field(field);
        }

        db.update_bot(&self.id, &partial, remove).await?;

        self.apply_options(partial);
        Ok(())
    }

    pub fn remove_field(&mut self, field: &FieldsBot) {
        match field {
            FieldsBot::Token => self.token = nanoid::nanoid!(64),
            FieldsBot::InteractionsUrl => {
                self.interactions_url = Some(String::new());
            }
        }
    }
}
