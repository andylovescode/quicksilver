use crate::{utils::GetDB, Context, Error};
use poise::CreateReply;
use serenity::all::User;

/// Get your level
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, person: Option<User>) -> eyre::Result<(), Error> {
    ctx.defer().await?;

    let user = if let Some(player) = person {
        player
    } else {
        ctx.author().clone()
    };

    let db = ctx.db("status").await;

    let db_user = db.state().get_user_or_default(&user.id);

    ctx.send(CreateReply::default().attachment(db_user.attachment_image(&user).await?))
        .await?;

    Ok(())
}
