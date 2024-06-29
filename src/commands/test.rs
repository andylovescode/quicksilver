use eyre::Result;
use serenity::all::{User};
use crate::{
    Context,
    Error,
    data::items::InventoryItem,
    data::state::DBEvent,
    utils::{Admin, GetDB},
    data::state::SideChannel
};
use crate::systems::autoconfig::ServerConfigRoleId;

#[poise::command(slash_command)]
pub async fn test(
    ctx: Context<'_>
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if !ctx.author().is_admin() {
        ctx.say("You are not an admin.").await?;
        return Ok(());
    }

    let mut db = ctx.db("test").await;

    db.update_config(&ctx, &ctx.guild_id().unwrap()).await?;

    let member = ctx.author_member().await.unwrap();

    member.add_role(ctx, db.state().servers[&ctx.guild_id().unwrap()].roles[&ServerConfigRoleId("admin".to_string())]).await?;

    ctx.say("did the thing").await?;

    Ok(())
}