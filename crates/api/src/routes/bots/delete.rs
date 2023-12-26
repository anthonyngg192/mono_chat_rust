use chat_core::models::User;
use chat_core::util::reference::Reference;
use chat_core::Database;
use chat_core::{Error, Result};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "Bots")]
#[delete("/<target>")]
pub async fn delete_bot(
    db: &State<Database>,
    user: User,
    target: Reference,
) -> Result<EmptyResponse> {
    let bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    bot.delete(db).await.map(|_| EmptyResponse)
}
