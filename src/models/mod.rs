use bancho_protocol::structures::{Mode, Mods};

pub mod bancho;
pub mod channels;
pub mod hardware_logs;
pub mod location;
pub mod messages;
pub mod multiplayer;
pub mod presences;
pub mod privileges;
pub mod relationships;
pub mod sessions;
pub mod stats;
pub mod users;

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Gamemode {
    #[default]
    Standard = 0,
    Taiko = 1,
    Catch = 2,
    Mania = 3,
    StandardRelax = 4,
    TaikoRelax = 5,
    CatchRelax = 6,
    StandardAutopilot = 8,
}

impl Gamemode {
    pub fn from_mode_and_mods(mode: Mode, mods: Mods) -> Gamemode {
        if mods.has_any(Mods::Relax) {
            match mode {
                Mode::Standard => Gamemode::StandardRelax,
                Mode::Taiko => Gamemode::TaikoRelax,
                Mode::Catch => Gamemode::CatchRelax,
                Mode::Mania => Gamemode::Mania,
            }
        } else if mods.has_any(Mods::Autopilot) {
            match mode {
                Mode::Standard => Gamemode::StandardAutopilot,
                Mode::Taiko => Gamemode::Taiko,
                Mode::Catch => Gamemode::Catch,
                Mode::Mania => Gamemode::Mania,
            }
        } else {
            match mode {
                Mode::Standard => Gamemode::Standard,
                Mode::Taiko => Gamemode::Taiko,
                Mode::Catch => Gamemode::Catch,
                Mode::Mania => Gamemode::Mania,
            }
        }
    }

    // TODO: get rid of this
    pub fn rxap(&self) -> u8 {
        match self {
            Gamemode::Standard => 0,
            Gamemode::Taiko => 0,
            Gamemode::Catch => 0,
            Gamemode::Mania => 0,
            Gamemode::StandardRelax => 1,
            Gamemode::TaikoRelax => 1,
            Gamemode::CatchRelax => 1,
            Gamemode::StandardAutopilot => 2,
        }
    }

    pub fn to_bancho(&self) -> Mode {
        match self {
            Gamemode::Standard => Mode::Standard,
            Gamemode::Taiko => Mode::Taiko,
            Gamemode::Catch => Mode::Catch,
            Gamemode::Mania => Mode::Mania,
            Gamemode::StandardRelax => Mode::Standard,
            Gamemode::TaikoRelax => Mode::Taiko,
            Gamemode::CatchRelax => Mode::Catch,
            Gamemode::StandardAutopilot => Mode::Standard,
        }
    }
}

impl TryFrom<u8> for Gamemode {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use std::io::{Error, ErrorKind};

        match value {
            0 => Ok(Gamemode::Standard),
            1 => Ok(Gamemode::Taiko),
            2 => Ok(Gamemode::Catch),
            3 => Ok(Gamemode::Mania),
            4 => Ok(Gamemode::StandardRelax),
            5 => Ok(Gamemode::TaikoRelax),
            6 => Ok(Gamemode::CatchRelax),
            8 => Ok(Gamemode::StandardAutopilot),
            _ => Err(Error::new(ErrorKind::InvalidData, "invalid gamemode")),
        }
    }
}
