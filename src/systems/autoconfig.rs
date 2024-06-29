use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, ChannelType, Color, Colour, CreateChannel, EditChannel, EditRole, GuildChannel, GuildId, PermissionOverwrite, PermissionOverwriteType, Permissions};
use crate::Context;
use crate::data::Database;
use crate::systems::autoconfig::ServerConfigChannel::{Category, Text};
use eyre::Result;
use crate::data::rng::Random;
use crate::data::state::{DBEvent, DBServer};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ServerConfigChannelId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ServerConfigRoleId(pub String);

fn channel(id: &str) -> ServerConfigChannelId {
    ServerConfigChannelId(id.to_string())
}

fn role(id: &str) -> ServerConfigRoleId {
    ServerConfigRoleId(id.to_string())
}

pub struct ServerConfig {
    children: Vec<ServerConfigChannelId>,
    channels: HashMap<ServerConfigChannelId, ServerConfigChannel>,
    roles: HashMap<ServerConfigRoleId, ServerConfigRole>,
    role_order: Vec<ServerConfigRoleId>
}

pub struct ServerConfigRole {
    name: String,
    color: Colour,
    permissions: Permissions
}

#[derive(Clone)]
pub struct ServerConfigPermissionOverwrite {
    allow: Permissions,
    deny: Permissions,
    role: ServerConfigRoleId
}

impl ServerConfigPermissionOverwrite {
    fn as_overwrite(&self, server: &DBServer) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: self.allow,
            deny: self.deny,
            kind: PermissionOverwriteType::Role(server.roles[&self.role])
        }
    }
}

#[derive(Clone)]
pub struct ServerConfigPermissions {
    overrides: Vec<ServerConfigPermissionOverwrite>,
    base: Permissions
}

pub struct ServerConfigTextLike {
    name: String,
    description: String,
    permissions: ServerConfigPermissions
}

impl ServerConfigTextLike {
    fn check_dirty(&self, channel: &GuildChannel, server: &DBServer) -> bool {
        if self.name.clone() != channel.name {
            return true
        }
        if Some(self.description.clone()) != channel.topic {
            return true
        }
        if Some(self.permissions.base) != channel.permissions {
            return true
        }
        if self.permissions.overrides.iter().map(| x | x.as_overwrite(server)).collect::<Vec<PermissionOverwrite>>() != channel.permission_overwrites {
            return true
        }
        return false
    }

    fn build(&self, server: &DBServer) -> CreateChannel {
        CreateChannel::new(self.name.clone())
            .topic(self.description.clone())
            .permissions(self.permissions.overrides.iter().map(| x | x.as_overwrite(server)).collect::<Vec<PermissionOverwrite>>())
    }
}

pub enum ServerConfigChannel {
    Text(ServerConfigTextLike),
    Rules(ServerConfigTextLike),
    News(ServerConfigTextLike),
    Voice {
        name: String,
        permissions: ServerConfigPermissions
    },
    Category {
        name: String,
        children: Vec<ServerConfigChannelId>
    }
}

impl ServerConfigChannel {
    fn kind(&self) -> ChannelType {
        match self {
            Text { .. } => ChannelType::Text,
            ServerConfigChannel::Rules(_) => ChannelType::Text,
            ServerConfigChannel::News(_) => ChannelType::News,
            ServerConfigChannel::Voice { .. } => ChannelType::Voice,
            Category { .. } => ChannelType::Category
        }
    }
}

trait LazyOrder {
    async fn lazy_order(&self, ctx: &Context<'_>, children: &Vec<ChannelId>) -> Result<()>;
}

impl LazyOrder for GuildId {
    async fn lazy_order(&self, ctx: &Context<'_>, children: &Vec<ChannelId>) -> Result<()> {
        let mut should_be_ordered = false;

        let mut last_index = -1;

        for child in children {
            let pos = child.to_channel(ctx).await?.guild().unwrap().position;

            if pos as i32 > last_index {
                last_index = pos as i32;
            } else {
                should_be_ordered = true;
            }
        }

        if !should_be_ordered { return Ok(()) }

        let mut index = 0u64;

        self.reorder_channels(ctx, children.iter().map(| x | {
            index += 1;

            (x.clone(), index)
        })).await?;

        Ok(())
    }
}

macro_rules! role {
    ($config:expr, $id:expr, $opts:expr) => {
        $config.roles.insert(role($id), $opts);

        $config.role_order.push(role($id));
    };
}

macro_rules! channel {
    ($config:expr, $id: expr, $opts: expr) => {{
        let opts = $opts;

        $config.channels.insert(ServerConfigChannelId($id.to_string()), opts);

        ServerConfigChannelId($id.to_string())
    }};
}


impl Database {
    pub async fn update_config(&mut self, ctx: &Context<'_>, guild_id: &GuildId) -> Result<()> {
        self.update_server(&ctx, self.get_config(&guild_id), &guild_id).await?;

        Ok(())
    }

    async fn update_server(&mut self, ctx: &Context<'_>, server_config: ServerConfig, guild_id: &GuildId) -> Result<()> {
        let server = self.state().get_server_or_default(&guild_id);

        // Step A.1: Ensure all declared roles exist
        for (id, _) in &server_config.roles {
            let exists = if server.roles.contains_key(&id) {
                let roles = guild_id.roles(ctx).await;

                roles.is_ok() && {
                    let roles = roles.unwrap();

                    roles.contains_key(&server.roles[id])
                }
            } else {
                false
            };

            if !exists {
                let role = ctx.http()
                    .get_guild(*guild_id).await?
                    .create_role(ctx.http(), EditRole::new().name("name pending")).await?;

                self.add(DBEvent::RoleAdd {
                    server: *guild_id,
                    id: id.clone(),
                    discord_id: role.id
                })?;
            }
        }

        // Step A.2: Mark used roles
        let mut used_roles = vec![];

        for (id, _) in &server_config.roles {
            let discord_id = server.roles[id];

            used_roles.push(discord_id);
        }

        // Step A.3: Delete unused roles
        let mut roles = guild_id.roles(ctx).await?;
        for (id, role) in &mut roles {
            if !used_roles.contains(&id) {
                let _ = role.delete(ctx).await;
            }
        }

        // Step A.4: Configure misconfigured roles
        for (id, config) in &server_config.roles {
            let mut role = roles[&server.roles[id]].clone();
            let dirty =
                role.colour != config.color ||
                role.name != config.name ||
                role.permissions != config.permissions;

            if dirty {
                role.edit(ctx, EditRole::new()
                    .name(&config.name)
                    .colour(config.color)
                    .permissions(config.permissions)
                ).await?;
            }
        }

        // Step A.5: Order roles
        {
            let mut idx = 0;

            for role in server_config.role_order {
                let mut discord = roles[&server.roles[&role]].clone();

                if discord.position != idx {
                    let _ = discord.edit(ctx, EditRole::new()
                        .position(idx)).await;
                }

                idx = discord.position + 1;
            }
        }

        // Step B.1: Ensure all declared channels exist
        for (id, _) in &server_config.channels {
            let exists = if server.channels.contains_key(&id) {
                let channel = ctx.http().get_channel(server.channels[id]).await;

                if channel.is_ok() {
                    let channel = channel.unwrap();

                    channel.guild().unwrap().kind == server_config.channels[&id].kind()
                } else {
                    false
                }
            } else {
                false
            };

            if !exists {
                let channel = ctx.http()
                    .get_guild(*guild_id).await?
                    .create_channel(ctx.http(),
                        CreateChannel::new(format!("uninitialized-{}", Random::new().get(0f32..1f32)))
                            .kind(server_config.channels[&id].kind())
                    ).await?;

                self.add(DBEvent::ChannelAdd {
                    server: *guild_id,
                    id: id.clone(),
                    discord_id: channel.id
                })?;
            }
        }

        // B.2. Put settings
        for (id, config) in &server_config.channels {
            let channel_id = server.channels[&id];

            let channel = ctx.http().get_channel(channel_id).await?;

            let guild_channel = channel.guild().unwrap();

            fn check_dirty(channel: &GuildChannel, config: &ServerConfigChannel, server: &DBServer) -> bool {
                match config {
                    Text(tl) => {
                        return tl.check_dirty(channel, server)
                    }
                    ServerConfigChannel::Rules(tl) => {
                        return tl.check_dirty(channel, server)
                    }
                    ServerConfigChannel::News(tl) => {
                        return tl.check_dirty(channel, server)
                    }
                    ServerConfigChannel::Voice { name, permissions } => {
                        if name.clone() != channel.name {
                            return true
                        }
                        if Some(permissions.base) != channel.permissions {
                            return true
                        }
                        if permissions.overrides.iter().map(| x | x.as_overwrite(server)).collect::<Vec<PermissionOverwrite>>() != channel.permission_overwrites {
                            return true
                        }
                    }
                    Category { name, children:_ } => {
                        if name.clone() != channel.name {
                            return true
                        }
                    }
                }

                return false
            }

            fn build<'a>(channel: &'a ServerConfigChannel, server: &'a DBServer) -> CreateChannel<'a> {
                (match channel {
                    Text(tl) => tl.build(&server),
                    ServerConfigChannel::Rules(tl) => tl.build(&server),
                    ServerConfigChannel::News(tl) => tl.build(&server),
                    ServerConfigChannel::Voice { name, permissions } => CreateChannel::new(name)
                        .permissions(permissions.overrides.iter().map(| x | x.as_overwrite(server)).collect::<Vec<PermissionOverwrite>>()),
                    Category { name, children:_ } => CreateChannel::new(name)
                }).kind(channel.kind())
            }

            if check_dirty(&guild_channel, &config, &server) {
                ctx.http().edit_channel(channel_id, &build(&config, &server), Some("Quicksilver autoconfig")).await?;
            }
        }

        // B.3. Find used channels
        let mut used_channels = vec![];

        for id in server_config.channels.keys() {
            let channel_id = server.channels[id];

            used_channels.push(channel_id);
        }

        // B.4. Burn it down
        for (id, channel) in guild_id.channels(ctx).await? {
            if !used_channels.contains(&id) {
                channel.delete(ctx).await?;
            }
        }

        // B.5. Arrange
        guild_id.lazy_order(ctx,
            &server_config.children.iter().map(| x | {
                server.channels[x]
            }).collect()
        ).await?;

        for (id, config) in &server_config.channels {
            if let Category { name:_, children } = config {
                let channel_id = server.channels[&id];

                for child in children.iter() {
                    let child_id = server.channels[&child];

                    let mut guild = child_id.to_channel(ctx).await?.guild().unwrap();

                    if guild.parent_id != Some(channel_id) {
                        guild.edit(ctx, EditChannel::new().category(channel_id)).await?;
                    }
                }

                guild_id.lazy_order(ctx,
                    &children.iter().map(| x | {
                        server.channels[x]
                    }).collect()
                ).await?;
            }
        }

        Ok(())
    }

    fn get_config(&self, _: &GuildId) -> ServerConfig {
        let mut config = ServerConfig {
            children: vec![],
            channels: HashMap::new(),
            roles: HashMap::new(),
            role_order: vec![],
        };

        // Roles
        role!(config, "admin", ServerConfigRole {
            name: "Admin".to_string(),
            permissions: Permissions::all(),
            color: Colour(0xFF0000)
        });

        role!(config, "operator", ServerConfigRole {
            name: "Operator".to_string(),
            permissions: Permissions::all(),
            color: Colour(0x00FFFF)
        });

        // Channels
        config.children.push(channel!(config, "chats", Category {
            name: "global-chats".to_string(),
            children: (1..5).map(| x | channel!(config, &format!("chats/global-{}", x), Text(ServerConfigTextLike {
                name: format!("global-chat-{}", x),
                description: "A global chat".to_string(),
                permissions: ServerConfigPermissions {
                    base: Permissions::default(),
                    overrides: vec![]
                }
            }))).collect(),
        }));

        config
    }
}