use rocket::request::FromParam;
use schemars::{
    schema::{InstanceType, Schema, SchemaObject, SingleOrVec},
    JsonSchema,
};
use serde::{Deserialize, Serialize};

use crate::{models::channel::Webhook, Result};
use crate::{
    models::{Bot, Channel, Emoji, Message, Server, User},
    Database,
};

#[derive(Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
}
impl Reference {
    pub fn from_unchecked(id: String) -> Reference {
        Reference { id }
    }

    pub async fn as_bot(&self, db: &Database) -> Result<Bot> {
        db.fetch_bot(&self.id).await
    }

    pub async fn as_emoji(&self, db: &Database) -> Result<Emoji> {
        db.fetch_emoji(&self.id).await
    }

    pub async fn as_channel(&self, db: &Database) -> Result<Channel> {
        db.fetch_channel(&self.id).await
    }

    pub async fn as_message(&self, db: &Database) -> Result<Message> {
        db.fetch_message(&self.id).await
    }

    pub async fn as_server(&self, db: &Database) -> Result<Server> {
        db.fetch_server(&self.id).await
    }

    pub async fn as_user(&self, db: &Database) -> Result<User> {
        db.fetch_user(&self.id).await
    }

    pub async fn as_webhook(&self, db: &Database) -> Result<Webhook> {
        db.fetch_webhook(&self.id).await
    }
}

impl<'r> FromParam<'r> for Reference {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(Reference::from_unchecked(param.into()))
    }
}

impl JsonSchema for Reference {
    fn schema_name() -> String {
        "Id".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            ..Default::default()
        })
    }
}
