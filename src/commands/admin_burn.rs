use crate::{
	data::{
		items::InventoryItem,
		state::{DBEvent, SideChannel},
	},
	utils::{Admin, GetDB},
	Context, Error,
};
use eyre::Result;
use serenity::all::User;

#[poise::command(slash_command)]
pub async fn admin_burn(ctx: Context<'_>, user: User, item: InventoryItem) -> Result<(), Error> {
	ctx.defer_ephemeral().await?;

	if !ctx.author().is_admin() {
		ctx.say("You are not an admin.").await?;
		return Ok(());
	}

	let mut db = ctx.db("admin burn").await;

	let err = db.add(DBEvent::AdminBurn {
		user: user.id,
		item,
	})?;

	match err {
		SideChannel::AdminBurnFail { user_error } => {
			ctx.say(format!("Error: {user_error}")).await?;
		}
		SideChannel::None => {
			ctx.say("Burned").await?;
		}
		state => panic!("Expected AdminBurnFail | None but got {:?}", state),
	}

	Ok(())
}
