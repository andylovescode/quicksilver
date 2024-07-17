use crate::{data::places::Place, systems::autoconfig::data::role, utils::GetDB, Context, Error};
use thiserror::Error;

#[derive(Debug, Error)]
enum GotoError {
	#[error("not in a guild")]
	NotInAGuild,

	#[error("role does not exist")]
	RoleDoesNotExist,

	#[error("cannot get roles")]
	CannotGetRoles,
}

/// Increments a global counter
#[poise::command(slash_command)]
pub async fn goto(ctx: Context<'_>, place: Place) -> eyre::Result<(), Error> {
	ctx.defer_ephemeral().await?;

	let mut db = ctx.db("counter increment").await;

	db.update_config(&ctx, &ctx.guild_id().unwrap()).await?;

	let state = db.state();

	let db_server = state.get_server_or_default(&ctx.guild_id().ok_or(GotoError::NotInAGuild)?);

	let role_name = &role(&format!("places/{}", place.id()));

	let role = db_server
		.roles
		.get(role_name)
		.ok_or(GotoError::RoleDoesNotExist)?;

	let member = ctx.author_member().await.ok_or(GotoError::NotInAGuild)?;

	for (id, other_role) in &db_server.roles {
		if id.0.starts_with("place")
			&& role.get() != other_role.get()
			&& member
				.roles(ctx)
				.ok_or(GotoError::CannotGetRoles)?
				.iter()
				.map(|x| x.id)
				.collect::<Vec<_>>()
				.contains(other_role)
		{
			member.remove_role(&ctx, other_role).await?;
		}
	}

	member.add_role(&ctx, role).await?;

	ctx.say("Transported").await?;

	Ok(())
}
