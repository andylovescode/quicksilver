use crate::config::get_bot_id;
use crate::data::rng::Random;
use crate::data::state::{DBEvent, DBServer};
use crate::data::Database;
use crate::systems::autoconfig::ServerConfigChannel::{Category, Text, Voice};
use crate::Context;
use eyre::Result;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ChannelId, ChannelType, Colour, CreateChannel, EditChannel, EditRole, GuildChannel, GuildId,
    PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
};
use std::collections::HashMap;
use thiserror::Error;

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
    role_order: Vec<ServerConfigRoleId>,
}

pub struct ServerConfigRole {
    name: String,
    color: Colour,
    permissions: Permissions,
}

#[derive(Clone)]
pub struct ServerConfigPermissionOverwrite {
    allow: Permissions,
    deny: Permissions,
    role: ServerConfigRoleId,
}

impl ServerConfigPermissionOverwrite {
    fn as_overwrite(&self, server: &DBServer) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: self.allow,
            deny: self.deny,
            kind: PermissionOverwriteType::Role(server.roles[&self.role]),
        }
    }
}

#[derive(Error, Debug)]
pub enum AutoconfigError {
    #[error("an option returned none")]
    OptionIsNone,
}

#[derive(Clone)]
pub struct ServerConfigPermissions {
    overrides: Vec<ServerConfigPermissionOverwrite>,
    base: Permissions,
}

trait ConsistentOrder {
    fn consistent_order(&self) -> Vec<PermissionOverwrite>;
}

impl ConsistentOrder for Vec<PermissionOverwrite> {
    fn consistent_order(&self) -> Vec<PermissionOverwrite> {
        let mut copy = self.clone();

        copy.sort_by(|a, b| {
            let num_a = match a.kind {
                PermissionOverwriteType::Role(r) => r.get(),
                PermissionOverwriteType::Member(m) => m.get(),
                _ => todo!(),
            };

            let num_b = match b.kind {
                PermissionOverwriteType::Role(r) => r.get(),
                PermissionOverwriteType::Member(m) => m.get(),
                _ => todo!(),
            };

            num_a.cmp(&num_b)
        });

        copy
    }
}

pub struct ServerConfigTextLike {
    name: String,
    description: String,
    permissions: ServerConfigPermissions,
}

impl ServerConfigTextLike {
    fn check_dirty(&self, channel: &GuildChannel, server: &DBServer) -> bool {
        if self.name.clone() != channel.name {
            return true;
        }
        if Some(self.description.clone()) != channel.topic {
            return true;
        }
        if overrides(&channel.guild_id, server, &self.permissions).consistent_order()
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
            .permissions(overrides(guild, server, &self.permissions))
    }
}

fn overrides(
    guild_id: &GuildId,
    server: &DBServer,
    perms: &ServerConfigPermissions,
) -> Vec<PermissionOverwrite> {
    let mut overrides = perms
        .overrides
        .iter()
        .map(|x| x.as_overwrite(server))
        .collect::<Vec<PermissionOverwrite>>();

    let guild_num = guild_id.get();

    overrides.push(PermissionOverwrite {
        kind: PermissionOverwriteType::Role(RoleId::new(guild_num)),
        allow: perms.base,
        deny: Permissions::empty(),
    });

    overrides
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
    fn kind(&self) -> ChannelType {
        match self {
            Text { .. } => ChannelType::Text,
            ServerConfigChannel::Rules(_) => ChannelType::Text,
            ServerConfigChannel::News(_) => ChannelType::News,
            ServerConfigChannel::Voice { .. } => ChannelType::Voice,
            Category { .. } => ChannelType::Category,
        }
    }

    fn build(&self, guild_id: &GuildId, server: &DBServer) -> EditChannel {
        (match self {
            Text(tl) => tl.build(guild_id, server),
            ServerConfigChannel::Rules(tl) => tl.build(guild_id, server),
            ServerConfigChannel::News(tl) => tl.build(guild_id, server),
            ServerConfigChannel::Voice { name, permissions } => EditChannel::new()
                .name(name)
                .permissions(overrides(guild_id, server, permissions)),
            Category { name, children: _ } => EditChannel::new().name(name),
        })
        .kind(self.kind())
    }

    fn check_dirty(&self, channel: &GuildChannel, server: &DBServer) -> bool {
        match self {
            Text(tl) => return tl.check_dirty(channel, server),
            ServerConfigChannel::Rules(tl) => return tl.check_dirty(channel, server),
            ServerConfigChannel::News(tl) => return tl.check_dirty(channel, server),
            ServerConfigChannel::Voice { name, permissions } => {
                if name.clone() != channel.name {
                    return true;
                }
                if overrides(&channel.guild_id, server, permissions).consistent_order()
                    != channel.permission_overwrites.consistent_order()
                {
                    return true;
                }
            }
            Category { name, children: _ } => {
                if name.clone() != channel.name {
                    return true;
                }
            }
        }

        false
    }
}

trait LazyOrder {
    async fn lazy_order(
        &self,
        ctx: &Context<'_>,
        children: &[ChannelId],
        channels: &HashMap<ChannelId, GuildChannel>,
    ) -> Result<()>;
}

impl LazyOrder for GuildId {
    async fn lazy_order(
        &self,
        ctx: &Context<'_>,
        children: &[ChannelId],
        channels: &HashMap<ChannelId, GuildChannel>,
    ) -> Result<()> {
        let mut should_be_ordered = false;

        let mut last_index = -1;

        for child in children {
            let pos = channels[child].position;

            if pos as i32 > last_index {
                last_index = pos as i32;
            } else {
                should_be_ordered = true;
            }
        }

        if !should_be_ordered {
            return Ok(());
        }

        let mut index = 0u64;

        self.reorder_channels(
            ctx,
            children.iter().map(|x| {
                index += 1;

                (*x, index)
            }),
        )
        .await?;

        Ok(())
    }
}

macro_rules! role {
    ($config:expr, $id:expr, $opts:expr) => {{
        $config.roles.insert(role($id), $opts);

        $config.role_order.push(role($id));
    }};
}

macro_rules! channel {
    ($config:expr, $id: expr, $opts: expr) => {{
        let opts = $opts;

        $config.channels.insert(channel($id), opts);

        channel($id)
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
    pub async fn update_config(&mut self, ctx: &Context<'_>, guild_id: &GuildId) -> Result<()> {
        self.update_server(ctx, self.get_config(guild_id), guild_id)
            .await?;

        Ok(())
    }

    async fn update_server(
        &mut self,
        ctx: &Context<'_>,
        server_config: ServerConfig,
        guild_id: &GuildId,
    ) -> Result<()> {
        let mut server = self.state().get_server_or_default(guild_id);

        let mut roles = guild_id.roles(ctx).await?;

        // Step A.1: Ensure all declared roles exist
        for id in server_config.roles.keys() {
            let exists = server.roles.contains_key(id) && roles.contains_key(&server.roles[id]);

            if !exists {
                let role = guild_id
                    .create_role(ctx, EditRole::new().name("name pending"))
                    .await?;

                self.add(DBEvent::RoleAdd {
                    server: *guild_id,
                    id: id.clone(),
                    discord_id: role.id,
                })?;
            }
        }

        // Step A.2: Mark used roles
        let mut used_roles = vec![];

        for id in server_config.roles.keys() {
            let discord_id = server.roles[id];

            used_roles.push(discord_id);
        }

        // Step A.3: Delete unused roles
        let (_, my_pos) = guild_id
            .member(ctx, get_bot_id())
            .await?
            .highest_role_info(ctx)
            .ok_or(AutoconfigError::OptionIsNone)?;

        for (id, role) in &mut roles {
            if !used_roles.contains(id) && role.position > my_pos {
                for (role, sid) in &server.roles {
                    if *sid == *id {
                        self.add(DBEvent::RoleForget {
                            id: role.clone(),
                            server: *guild_id,
                        })?;
                    }
                }
                role.delete(ctx).await?;
            }
        }

        // Step A.4: Configure misconfigured roles
        for (id, config) in &server_config.roles {
            let mut role = roles[&server.roles[id]].clone();
            let dirty = role.colour != config.color
                || role.name != config.name
                || role.permissions != config.permissions;

            if dirty {
                role.edit(
                    ctx,
                    EditRole::new()
                        .name(&config.name)
                        .colour(config.color)
                        .permissions(config.permissions),
                )
                .await?;
            }
        }

        // Step A.5: Order roles
        {
            let mut idx = my_pos;

            for role in server_config.role_order {
                idx -= 1;

                let discord = roles[&server.roles[&role]].clone();

                if idx > my_pos || discord.position > my_pos {
                    println!(
                        "warn: role {} will go over, or is over current role, cancelled operation",
                        role.0
                    );
                    continue;
                }

                if discord.position != idx {
                    guild_id.edit_role_position(ctx, discord, idx).await?;
                }
            }
        }

        // Step B.1: Ensure all declared channels exist
        let mut channels = guild_id.channels(ctx).await?;

        for id in server_config.channels.keys() {
            let exists = server.channels.contains_key(id)
                && {
                    if !channels.contains_key(&server.channels[id]) {
                        channels = guild_id.channels(ctx).await?
                    };
                    true
                }
                && channels.contains_key(&server.channels[id])
                && channels[&server.channels[id]].kind == server_config.channels[id].kind(); // fixme: hell

            if !exists {
                let channel = guild_id
                    .create_channel(
                        ctx,
                        CreateChannel::new(format!(
                            "uninitialized-{}",
                            Random::new().get(0f32..1f32)
                        ))
                        .kind(server_config.channels[id].kind()),
                    )
                    .await?;

                self.add(DBEvent::ChannelAdd {
                    server: *guild_id,
                    id: id.clone(),
                    discord_id: channel.id,
                })?;

                server = self.state().get_server_or_default(guild_id); // fixme: this is f***ing evil

                channels.insert(server.channels[id], channel);
            }
        }

        // B.2. Put settings
        for (id, config) in &server_config.channels {
            let channel_id = server.channels[id];

            let guild = &channels[&channel_id];

            if config.check_dirty(guild, &server) {
                channel_id
                    .edit(ctx, config.build(guild_id, &server))
                    .await?;
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
                for (channel, sid) in &server.channels {
                    if *sid == id {
                        self.add(DBEvent::ChannelForget {
                            id: channel.clone(),
                            server: *guild_id,
                        })?;
                    }
                }
                channel.delete(ctx).await?;
            }
        }

        // B.5. Arrange
        guild_id
            .lazy_order(
                ctx,
                &server_config
                    .children
                    .iter()
                    .map(|x| server.channels[x])
                    .collect::<Vec<ChannelId>>(),
                &channels,
            )
            .await?;

        for (id, config) in &server_config.channels {
            if let Category { name: _, children } = config {
                let channel_id = server.channels[id];

                for child in children.iter() {
                    let child_id = server.channels[child];

                    let mut guild = channels[&child_id].clone();

                    if guild.parent_id != Some(channel_id) {
                        guild
                            .edit(ctx, EditChannel::new().category(channel_id))
                            .await?;
                    }
                }

                guild_id
                    .lazy_order(
                        ctx,
                        &children
                            .iter()
                            .map(|x| server.channels[x])
                            .collect::<Vec<ChannelId>>(),
                        &channels,
                    )
                    .await?;
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
            Category {
                name: "~ EVERYWHERE ~".to_string(),
                children: (1..5)
                    .map(|x| channel!(
                        config,
                        &format!("chats/global-{}", x),
                        Text(ServerConfigTextLike {
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
                    role: role(&format!("place/{}", place.id)),
                    allow: Permissions::VIEW_CHANNEL,
                    deny: Permissions::empty(),
                }],
            };

            config.children.push(channel!(
                config,
                &format!("places/{}", place.id),
                Category {
                    name: format!("~ {} ~", place.name),
                    children: vec![
                        channel!(
                            config,
                            &format!("places/{}/text", place.id),
                            Text(ServerConfigTextLike {
                                name: format!("{}-text", place.id),
                                description: "A place in Alternate Reality".to_string(),
                                permissions: visible_only_here.clone()
                            })
                        ),
                        channel!(
                            config,
                            &format!("places/{}/vc", place.id),
                            Voice {
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
