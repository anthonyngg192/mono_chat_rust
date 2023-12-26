use crate::{
    events::client::EventV1,
    models::channel::{FieldsWebhook, PartialWebhook, Webhook},
    Database, Error, Result,
};

#[allow(clippy::disallowed_methods)]
impl Webhook {
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_webhook(self).await?;

        // Avoid leaking the token to people who receive the event
        let mut webhook = self.clone();
        webhook.token = None;

        EventV1::WebhookCreate(webhook)
            .p(self.channel_id.clone())
            .await;

        Ok(())
    }

    pub fn assert_token(&self, token: &str) -> Result<()> {
        if self.token.as_deref() == Some(token) {
            Ok(())
        } else {
            Err(Error::InvalidCredentials)
        }
    }

    pub async fn update(
        &mut self,
        db: &Database,
        mut partial: PartialWebhook,
        remove: Vec<FieldsWebhook>,
    ) -> Result<()> {
        for field in &remove {
            self.remove_field(field)
        }

        self.apply_options(partial.clone());

        db.update_webhook(&self.id, &partial, &remove).await?;

        partial.token = None; // Avoid leaking the token to people who receive the event

        EventV1::WebhookUpdate {
            id: self.id.clone(),
            data: partial,
            remove: remove.into_iter().collect(),
        }
        .p(self.channel_id.clone())
        .await;

        Ok(())
    }

    pub fn remove_field(&mut self, field: &FieldsWebhook) {
        match field {
            FieldsWebhook::Avatar => self.avatar = None,
        }
    }

    pub async fn delete(&self, db: &Database) -> Result<()> {
        db.delete_webhook(&self.id).await?;

        EventV1::WebhookDelete {
            id: self.id.clone(),
        }
        .p(self.channel_id.clone())
        .await;

        Ok(())
    }
}
