use crate::common::error::{AppError, ServiceResult};
use crate::entities::gamemodes::Gamemode;
use crate::entities::multiplayer::{
    MultiplayerMatch as Entity, MultiplayerMatchSlot as SlotEntity,
};
use crate::entities::sessions::SessionIdentity;
use crate::repositories::multiplayer::MULTIPLAYER_MAX_SIZE;
use bancho_protocol::structures::{
    Match, MatchSlot, MatchTeam, MatchTeamType, Mods, SlotStatus, WinCondition,
};

#[derive(Debug, Clone)]
pub struct MultiplayerMatch {
    pub match_id: i64,
    pub name: String,
    pub password: String,
    pub in_progress: bool,
    pub powerplay: bool,
    pub mods: Mods,
    pub beatmap_name: String,
    pub beatmap_md5: String,
    pub beatmap_id: i32,
    pub host_user_id: i64,
    pub tournament_owner_id: Option<i64>,
    pub mode: Gamemode,
    pub win_condition: WinCondition,
    pub team_type: MatchTeamType,
    pub freemod_enabled: bool,
    pub random_seed: i32,
    pub last_game_id: Option<i64>,
}

#[derive(Copy, Clone)]
pub struct MultiplayerMatchSlot {
    pub status: SlotStatus,
    pub team: MatchTeam,
    pub mods: Mods,
    pub user: Option<SessionIdentity>,
    pub loaded: bool,
    pub skipped: bool,
    pub failed: bool,
    pub completed: bool,
}

impl MultiplayerMatch {
    pub fn controller_user_id(&self) -> i64 {
        self.tournament_owner_id.unwrap_or(self.host_user_id)
    }

    pub fn as_entity(&self) -> Entity {
        self.clone().into()
    }

    pub fn ingame_match_id(&self) -> u16 {
        // We have match identifiers that require 64 bits
        // osu! only accepts 16 bits to represent your match identifier
        // thus we take the lower 16 bits of our 64 bit identifier
        (self.match_id & 0xFFFF) as _
    }

    pub fn invite_message(&self) -> String {
        let safe_password = self.password.replace(" ", "_");
        format!(
            "\x01ACTION has invited you to their multiplayer match: [osump://{}/{} {}]",
            self.ingame_match_id(),
            safe_password,
            self.name,
        )
    }

    pub fn as_lobby_match(&self, slots: MultiplayerMatchSlots) -> Match<'_> {
        let mut data = self.as_bancho(slots);
        if self.tournament_owner_id.is_some() && !data.password.is_empty() {
            data.password = "\x01";
        }
        data
    }

    pub fn as_bancho(&self, slots: [MultiplayerMatchSlot; 16]) -> Match<'_> {
        let freemods = match self.freemod_enabled {
            true => Some(slots.to_mods()),
            false => None,
        };
        Match {
            id: self.ingame_match_id(),
            in_progress: self.in_progress,
            powerplay: self.powerplay,
            mods: self.mods,
            name: &self.name,
            password: &self.password,
            beatmap_name: &self.beatmap_name,
            beatmap_md5: &self.beatmap_md5,
            beatmap_id: self.beatmap_id,
            slots: slots.as_bancho(),
            host: self.host_user_id as _,
            mode: self.mode.as_bancho(),
            win_condition: self.win_condition,
            team_type: self.team_type,
            freemod_enabled: self.freemod_enabled,
            random_seed: self.random_seed,
            freemods,
        }
    }
}

impl Into<Entity> for MultiplayerMatch {
    fn into(self) -> Entity {
        Entity {
            match_id: self.match_id,
            name: self.name,
            password: self.password,
            in_progress: self.in_progress,
            powerplay: self.powerplay,
            mods: self.mods.bits(),
            beatmap_name: self.beatmap_name,
            beatmap_md5: self.beatmap_md5,
            beatmap_id: self.beatmap_id,
            host_user_id: self.host_user_id,
            tournament_owner_id: self.tournament_owner_id,
            mode: self.mode as _,
            win_condition: self.win_condition as _,
            team_type: self.team_type as _,
            freemod_enabled: self.freemod_enabled,
            random_seed: self.random_seed,
            last_game_id: self.last_game_id,
        }
    }
}

impl TryFrom<Entity> for MultiplayerMatch {
    type Error = AppError;

    fn try_from(value: Entity) -> ServiceResult<Self> {
        Ok(Self {
            match_id: value.match_id,
            name: value.name,
            password: value.password,
            in_progress: value.in_progress,
            powerplay: value.powerplay,
            mods: Mods::from_bits_retain(value.mods),
            beatmap_name: value.beatmap_name,
            beatmap_md5: value.beatmap_md5,
            beatmap_id: value.beatmap_id,
            host_user_id: value.host_user_id,
            tournament_owner_id: value.tournament_owner_id,
            mode: Gamemode::try_from(value.mode)?,
            win_condition: WinCondition::try_from(value.win_condition)?,
            team_type: MatchTeamType::try_from(value.team_type)?,
            freemod_enabled: value.freemod_enabled,
            random_seed: value.random_seed,
            last_game_id: value.last_game_id,
        })
    }
}

pub type MultiplayerMatchSlots = [MultiplayerMatchSlot; MULTIPLAYER_MAX_SIZE];

impl MultiplayerMatchSlot {
    pub fn as_entity(&self) -> SlotEntity {
        self.clone().into()
    }

    pub fn from<const N: usize>(entities: [SlotEntity; N]) -> [MultiplayerMatchSlot; N] {
        std::array::from_fn(|i| MultiplayerMatchSlot {
            status: SlotStatus::from_bits_retain(entities[i].status),
            team: MatchTeam::from_u8(entities[i].team),
            mods: Mods::from_bits_retain(entities[i].mods),
            user: entities[i].user,
            loaded: entities[i].loaded,
            skipped: entities[i].skipped,
            failed: entities[i].failed,
            completed: entities[i].completed,
        })
    }
}

impl From<SlotEntity> for MultiplayerMatchSlot {
    fn from(value: SlotEntity) -> Self {
        Self {
            status: SlotStatus::from_bits_retain(value.status),
            team: MatchTeam::from_u8(value.team),
            mods: Mods::from_bits_retain(value.mods),
            user: value.user,
            loaded: value.loaded,
            skipped: value.skipped,
            failed: value.failed,
            completed: value.completed,
        }
    }
}

impl Into<SlotEntity> for MultiplayerMatchSlot {
    fn into(self) -> SlotEntity {
        SlotEntity {
            status: self.status.bits(),
            team: self.team as _,
            mods: self.mods.bits(),
            user: self.user,
            loaded: self.loaded,
            skipped: self.skipped,
            failed: self.failed,
            completed: self.completed,
        }
    }
}

pub trait MatchSlotExt<const N: usize> {
    fn as_bancho(&self) -> [MatchSlot; N];
    fn as_entity(&self) -> [SlotEntity; N];
    fn to_mods(&self) -> [Mods; N];
}

impl<const N: usize> MatchSlotExt<N> for [MultiplayerMatchSlot; N] {
    fn as_bancho(&self) -> [MatchSlot; N] {
        std::array::from_fn(|i| MatchSlot {
            status: self[i].status,
            team: self[i].team,
            user_id: self[i].user.map_or(0, |slot_user| slot_user.user_id) as _,
        })
    }

    fn as_entity(&self) -> [SlotEntity; N] {
        std::array::from_fn(|i| self[i].as_entity())
    }

    fn to_mods(&self) -> [Mods; N] {
        std::array::from_fn(|i| self[i].mods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tournament_lobby_packets_redact_passwords_and_keep_active_room_ids() {
        let room = MultiplayerMatch::try_from(Entity {
            match_id: 65_537,
            tournament_owner_id: Some(5),
            host_user_id: 7,
            password: "invite-only".to_owned(),
            ..Default::default()
        })
        .unwrap();
        let slots = MultiplayerMatchSlot::from([SlotEntity::default(); MULTIPLAYER_MAX_SIZE]);
        let lobby = room.as_lobby_match(slots);
        assert_eq!(lobby.id, 1);
        assert!(!lobby.password.is_empty());
        assert_ne!(lobby.password, room.password);
        assert_eq!(room.controller_user_id(), 5);
        assert!(room.invite_message().contains("osump://1/invite-only"));
    }

    #[test]
    fn existing_rooms_without_an_organizer_keep_their_host() {
        let row = Entity {
            host_user_id: 7,
            ..Default::default()
        };
        let mut stored = serde_json::to_value(row).unwrap();
        stored
            .as_object_mut()
            .unwrap()
            .remove("tournament_owner_id");
        let room =
            MultiplayerMatch::try_from(serde_json::from_value::<Entity>(stored).unwrap()).unwrap();
        assert_eq!(room.tournament_owner_id, None);
        assert_eq!(room.controller_user_id(), 7);
    }
}
