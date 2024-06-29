use std::{path::Path, sync::Arc};

use crate::{
	commands::{
		admin_burn::admin_burn, admin_give::admin_give, coin::coinflip, counter::counter,
		inventory::inventory, status::status, test::test,
	},
	config::get_testing_guild,
	data::Database,
	systems::xp_leveling::XPHandler,
};
use eyre::Result;
use poise::{builtins::create_application_commands, serenity_prelude as serenity};
use serenity::Command;
use tokio::sync::Mutex;

pub mod commands;
pub mod config;
pub mod data;
pub mod systems;
pub mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Mutex<Database>>, Error>;

async fn eyre_main() -> Result<()> {
	// Create db
	let db = Arc::new(Mutex::new(Database::new(Path::new("./db.json").into())?));

	// We need message perms
	let intents = serenity::GatewayIntents::all();

	let db_for_poise = Arc::clone(&db);

	// And we want to create a bot
	let framework = poise::Framework::builder()
		// With these settings:
		.options(poise::FrameworkOptions {
			// With these commands:
			commands: vec![
				counter(),
				coinflip(),
				status(),
				inventory(),
				admin_give(),
				admin_burn(),
				test(),
			],

			// And default settings
			..Default::default()
		})
		// And when we set things up
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				let commands = create_application_commands(&framework.options().commands);

				get_testing_guild().set_commands(ctx, vec![]).await?;

				Command::set_global_commands(ctx, commands).await?;

				// And load our database
				Ok(db_for_poise)
			})
		})
		// And let's set things up
		.build();

	// And connect to discord
	let mut client = serenity::ClientBuilder::new(config::get_token(), intents)
		.framework(framework)
		.event_handler(XPHandler::new(Arc::clone(&db)))
		.await?;

	// And run it all
	client.start().await?;

	Ok(())
}

#[tokio::main]
async fn main() { eyre_main().await.unwrap(); }
