use eyre::Result;
use serenity::all::{User};
use crate::{
    data::items::InventoryItem,
    Context,
    Error,
    data::state::DBEvent,
    utils::{Admin, GetDB}
};

#[poise::command(slash_command)]
pub async fn admin_give(
    ctx: Context<'_>,
    user: User,
    item: InventoryItem
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if !ctx.author().is_admin() {
        ctx.say("You are not an admin.").await?;
        return Ok(());
    }

    let mut db = ctx.db("admin give").await;

    db.add(DBEvent::AdminGive {
        user: user.id,
        item
    })?;

    ctx.say("Granted").await?;

    Ok(())
}