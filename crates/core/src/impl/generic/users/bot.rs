use nanoid::nanoid;

use crate::{
    database::Database,
    models::{bot::FieldsBot, Bot},
    Result,
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
}
