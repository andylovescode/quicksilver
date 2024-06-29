use crate::{
    data::items::InventoryItem,
    data::rng::Chance,
    data::user::DBUser,
    utils::calculate_length_to_xp
};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, RoleId, UserId};
use std::collections::{HashMap, HashSet};
use crate::data::user::DBUserError;
use crate::systems::autoconfig::{ServerConfigChannelId, ServerConfigRoleId};

#[derive(Serialize, Deserialize, Debug)]
pub enum DBEvent {
    Counter { user: UserId },
    CoinFlip { chance: Chance },
    UserSendMessage { user: UserId, length: usize },
    AdminGive { user: UserId, item: InventoryItem },
    AdminBurn { user: UserId, item: InventoryItem },
    ChannelForget { server: GuildId, id: ServerConfigChannelId },
    ChannelAdd { server: GuildId, id: ServerConfigChannelId, discord_id: ChannelId },
    RoleForget { server: GuildId, id: ServerConfigRoleId },
    RoleAdd { server: GuildId, id: ServerConfigRoleId, discord_id: RoleId }
}

#[derive(Debug)]
pub enum SideChannel {
    CoinFlip { success: bool },
    AdminBurnFail { user_error: DBUserError },
    None
}

impl DBEvent {
    pub fn reduce_state(&self, state: DBState) -> (DBState, SideChannel) {
        match self {
            DBEvent::Counter { user } => state.mutated(|s| {
                s.counter += 1;
                s.people_who_counted.insert(*user);

                SideChannel::None
            }),
            DBEvent::CoinFlip { chance } => state.mutated(|s| {
                if chance.eval(0.5) {
                    s.flips_in_a_row += 1
                } else {
                    s.flips_in_a_row = 0
                }

                SideChannel::CoinFlip { success: chance.eval(0.5) }
            }),
            DBEvent::UserSendMessage { user, length } => state.mutated(|s| {
                if user.clone() == s.last_typed_user {
                    return SideChannel::None
                }

                s.last_typed_user = user.clone();

                // Figure out how much XP we need
                let xp = calculate_length_to_xp(length);

                // And give it to the user
                let mut db_user = s.get_user_or_create(user);

                db_user.gain_xp(xp);

                s.update_user(user, db_user);

                SideChannel::None
            }),
            DBEvent::AdminGive { user, item } => state.mutated(|s| {
                let mut db_user = s.get_user_or_create(user);

                db_user.give_item(*item);

                s.update_user(user, db_user);

                SideChannel::None
            }),
            DBEvent::AdminBurn { user, item } => state.mutated(|s| {
                let mut db_user = s.get_user_or_create(user);

                let result = db_user.drop_item(*item);

                s.update_user(user, db_user);

                match result {
                    Ok(_) => SideChannel::None,
                    Err(err) => SideChannel::AdminBurnFail { user_error: err }
                }
            }),
            DBEvent::ChannelAdd { server, id, discord_id } => state.mutated(| s | {
                let mut db_server = s.get_server_or_create(server);

                db_server.channels.insert(id.clone(), discord_id.clone());

                s.update_server(server, db_server);

                SideChannel::None
            }),
            DBEvent::ChannelForget { server, id } => state.mutated(| s | {
                let mut db_server = s.get_server_or_create(server);

                db_server.channels.remove(id);

                s.update_server(server, db_server);

                SideChannel::None
            }),
            DBEvent::RoleAdd { server, id, discord_id } => state.mutated(| s | {
                let mut db_server = s.get_server_or_create(server);

                db_server.roles.insert(id.clone(), discord_id.clone());

                s.update_server(server, db_server);

                SideChannel::None
            }),
            DBEvent::RoleForget { server, id } => state.mutated(| s | {
                let mut db_server = s.get_server_or_create(server);

                db_server.roles.remove(id);

                s.update_server(server, db_server);

                SideChannel::None
            })
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct DBState {
    pub counter: u64,
    pub people_who_counted: HashSet<UserId>,

    pub flips_in_a_row: u32,

    pub users: HashMap<UserId, DBUser>,
    pub servers: HashMap<GuildId, DBServer>,

    pub last_typed_user: UserId
}

#[derive(Clone, Default, Debug)]
pub struct DBServer {
    pub channels: HashMap<ServerConfigChannelId, ChannelId>,
    pub roles: HashMap<ServerConfigRoleId, RoleId>
}

impl DBState {
    pub fn get_user_or_default(&self, id: &UserId) -> DBUser {
        if self.users.contains_key(id) {
            self.users[id].clone()
        } else {
            DBUser::default()
        }
    }

    pub fn get_user_or_create(&mut self, id: &UserId) -> DBUser {
        if !self.users.contains_key(id) {
            self.users.insert(*id, DBUser::default());
        }

        self.users[id].clone()
    }

    pub fn update_user(&mut self, id: &UserId, user: DBUser) {
        self.users.insert(*id, user);
    }

    pub fn get_server_or_default(&self, id: &GuildId) -> DBServer {
        if self.servers.contains_key(id) {
            self.servers[id].clone()
        } else {
            DBServer::default()
        }
    }

    pub fn get_server_or_create(&mut self, id: &GuildId) -> DBServer {
        if !self.servers.contains_key(id) {
            self.servers.insert(*id, DBServer::default());
        }

        self.servers[id].clone()
    }

    pub fn update_server(&mut self, id: &GuildId, user: DBServer) {
        self.servers.insert(*id, user);
    }
}

impl DBState {
    pub fn mutated<T>(&self, callback: T) -> (Self, SideChannel)
    where
        T: Fn(&mut Self) -> SideChannel,
    {
        let mut fork = self.clone();

        let side_channel = callback(&mut fork);

        (fork, side_channel)
    }
}
