use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, JsonSchema)]
pub struct CreateWebhookBody {
    #[validate(length(min = 1, max = 32))]
    pub name: String,

    #[validate(length(min = 1, max = 128))]
    pub avatar: Option<String>,
}
