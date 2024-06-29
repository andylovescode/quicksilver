use crate::{
	systems::autoconfig::data::ServerConfigRoleId,
	utils::{Admin, GetDB},
	Context, Error,
};
use eyre::Result;

#[poise::command(slash_command)]
pub async fn test(ctx: Context<'_>) -> Result<(), Error> {
	ctx.defer_ephemeral().await?;

	if !ctx.author().is_admin() {
		ctx.say("You are not an admin.").await?;
		return Ok(());
	}

	let mut db = ctx.db("test").await;

	db.update_config(&ctx, &ctx.guild_id().unwrap()).await?;

	let member = ctx.author_member().await.unwrap();

	member
		.add_role(
			ctx,
			db.state().servers[&ctx.guild_id().unwrap()].roles
				[&ServerConfigRoleId("admin".to_string())],
		)
		.await?;

	ctx.say("did the thing").await?;

	Ok(())
}
