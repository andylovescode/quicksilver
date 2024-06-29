use crate::{
	data::state::DBServer,
	systems::autoconfig::{apply_config, apply_config::ConsistentOrder},
};
use serde::{Deserialize, Serialize};
use serenity::all::{
	ChannelType, Colour, EditChannel, GuildChannel, GuildId, PermissionOverwrite,
	PermissionOverwriteType, Permissions,
};
use std::collections::HashMap;

pub fn channel(id: &str) -> ServerConfigChannelId { ServerConfigChannelId(id.to_string()) }

pub fn role(id: &str) -> ServerConfigRoleId { ServerConfigRoleId(id.to_string()) }

pub struct ServerConfig {
	pub(crate) children: Vec<ServerConfigChannelId>,
	pub(crate) channels: HashMap<ServerConfigChannelId, ServerConfigChannel>,
	pub(crate) roles: HashMap<ServerConfigRoleId, ServerConfigRole>,
	pub(crate) role_order: Vec<ServerConfigRoleId>,
}

pub struct ServerConfigRole {
	pub(crate) name: String,
	pub(crate) color: Colour,
	pub(crate) permissions: Permissions,
}

#[derive(Clone)]
pub struct ServerConfigPermissionOverwrite {
	pub(crate) allow: Permissions,
	pub(crate) deny: Permissions,
	pub(crate) role: ServerConfigRoleId,
}

impl ServerConfigPermissionOverwrite {
	pub(crate) fn as_overwrite(&self, server: &DBServer) -> PermissionOverwrite {
		PermissionOverwrite {
			allow: self.allow,
			deny: self.deny,
			kind: PermissionOverwriteType::Role(server.roles[&self.role]),
		}
	}
}

#[derive(Clone)]
pub struct ServerConfigPermissions {
	pub(crate) overrides: Vec<ServerConfigPermissionOverwrite>,
	pub(crate) base: Permissions,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ServerConfigChannelId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ServerConfigRoleId(pub String);

pub struct ServerConfigTextLike {
	pub(crate) name: String,
	pub(crate) description: String,
	pub(crate) permissions: ServerConfigPermissions,
}

impl ServerConfigTextLike {
	fn check_dirty(&self, channel: &GuildChannel, server: &DBServer) -> bool {
		if self.name.clone() != channel.name {
			return true;
		}
		if Some(self.description.clone()) != channel.topic {
			return true;
		}
		if apply_config::overrides(&channel.guild_id, server, &self.permissions).consistent_order()
			!= channel.permission_overwrites.consistent_order()
		{
			return true;
		}
		false
	}

	fn build(&self, guild: &GuildId, server: &DBServer) -> EditChannel {
		EditChannel::new()
			.name(self.name.clone())
			.topic(self.description.clone())
			.permissions(apply_config::overrides(guild, server, &self.permissions))
	}
}

pub enum ServerConfigChannel {
	Text(ServerConfigTextLike),
	Rules(ServerConfigTextLike),
	News(ServerConfigTextLike),
	Voice {
		name: String,
		permissions: ServerConfigPermissions,
	},
	Category {
		name: String,
		children: Vec<ServerConfigChannelId>,
	},
}

impl ServerConfigChannel {
	pub(crate) fn kind(&self) -> ChannelType {
		match self {
			ServerConfigChannel::Text { .. } => ChannelType::Text,
			ServerConfigChannel::Rules(_) => ChannelType::Text,
			ServerConfigChannel::News(_) => ChannelType::News,
			ServerConfigChannel::Voice { .. } => ChannelType::Voice,
			ServerConfigChannel::Category { .. } => ChannelType::Category,
		}
	}

	pub(crate) fn build(&self, guild_id: &GuildId, server: &DBServer) -> EditChannel {
		(match self {
			ServerConfigChannel::Text(tl) => tl.build(guild_id, server),
			ServerConfigChannel::Rules(tl) => tl.build(guild_id, server),
			ServerConfigChannel::News(tl) => tl.build(guild_id, server),
			ServerConfigChannel::Voice { name, permissions } => EditChannel::new()
				.name(name)
				.permissions(apply_config::overrides(guild_id, server, permissions)),
			ServerConfigChannel::Category { name, children: _ } => EditChannel::new().name(name),
		})
		.kind(self.kind())
	}

	pub(crate) fn check_dirty(&self, channel: &GuildChannel, server: &DBServer) -> bool {
		match self {
			ServerConfigChannel::Text(tl) => return tl.check_dirty(channel, server),
			ServerConfigChannel::Rules(tl) => return tl.check_dirty(channel, server),
			ServerConfigChannel::News(tl) => return tl.check_dirty(channel, server),
			ServerConfigChannel::Voice { name, permissions } => {
				if name.clone() != channel.name {
					return true;
				}
				if apply_config::overrides(&channel.guild_id, server, permissions)
					.consistent_order()
					!= channel.permission_overwrites.consistent_order()
				{
					return true;
				}
			}
			ServerConfigChannel::Category { name, children: _ } => {
				if name.clone() != channel.name {
					return true;
				}
			}
		}

		false
	}
}
