use crate::{
	data::Database,
	systems::autoconfig::{
		data,
		data::{
			ServerConfig, ServerConfigChannel, ServerConfigPermissionOverwrite,
			ServerConfigPermissions, ServerConfigRole, ServerConfigTextLike,
		},
	},
};
use serenity::all::{Colour, GuildId, Permissions};
use std::collections::HashMap;

macro_rules! role {
	($config:expr, $id:expr, $opts:expr) => {{
		$config.roles.insert(data::role($id), $opts);

		$config.role_order.push(data::role($id));
	}};
}

macro_rules! channel {
	($config:expr, $id: expr, $opts: expr) => {{
		let opts = $opts;

		$config.channels.insert(data::channel($id), opts);

		data::channel($id)
	}};
}

pub struct Place<'a> {
	name: &'a str,
	id: &'a str,
}

static PLACES: &[Place] = &[
	Place {
		name: "The Forest",
		id: "forest",
	},
	Place {
		name: "The Deep Forest",
		id: "deep-forest",
	},
];

impl Database {
	pub fn get_config(&self, _: &GuildId) -> ServerConfig {
		let mut config = ServerConfig {
			children: vec![],
			channels: HashMap::new(),
			roles: HashMap::new(),
			role_order: vec![],
		};

		// Roles
		role!(
			config,
			"admin",
			ServerConfigRole {
				name: "Admin".to_string(),
				permissions: Permissions::all(),
				color: Colour(0xFF0000)
			}
		);

		role!(
			config,
			"operator",
			ServerConfigRole {
				name: "Operator".to_string(),
				permissions: Permissions::all(),
				color: Colour(0x00FFFF)
			}
		);

		for place in PLACES {
			role!(
				config,
				&format!("place/{}", place.id),
				ServerConfigRole {
					name: format!("📍 {}", place.name),
					color: Colour(0x000000),
					permissions: Permissions::empty()
				}
			);
		}

		// Channels
		config.children.push(channel!(
			config,
			"chats",
			ServerConfigChannel::Category {
				name: "~ EVERYWHERE ~".to_string(),
				children: (1..5)
					.map(|x| channel!(
						config,
						&format!("chats/global-{}", x),
						ServerConfigChannel::Text(ServerConfigTextLike {
							name: format!("global-chat-{}", x),
							description: "A global chat".to_string(),
							permissions: ServerConfigPermissions {
								base: Permissions::default(),
								overrides: vec![]
							}
						})
					))
					.collect(),
			}
		));

		for place in PLACES {
			let visible_only_here = ServerConfigPermissions {
				base: Permissions::default() & !Permissions::VIEW_CHANNEL,
				overrides: vec![ServerConfigPermissionOverwrite {
					role: data::role(&format!("place/{}", place.id)),
					allow: Permissions::VIEW_CHANNEL,
					deny: Permissions::empty(),
				}],
			};

			config.children.push(channel!(
				config,
				&format!("places/{}", place.id),
				ServerConfigChannel::Category {
					name: format!("~ {} ~", place.name),
					children: vec![
						channel!(
							config,
							&format!("places/{}/text", place.id),
							ServerConfigChannel::Text(ServerConfigTextLike {
								name: format!("{}-text", place.id),
								description: "A place in Alternate Reality".to_string(),
								permissions: visible_only_here.clone()
							})
						),
						channel!(
							config,
							&format!("places/{}/vc", place.id),
							ServerConfigChannel::Voice {
								name: format!("{}-vc", place.id),
								permissions: visible_only_here.clone()
							}
						),
					]
				}
			));
		}

		config
	}
}
