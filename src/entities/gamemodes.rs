use bancho_protocol::structures::{Mode, Mods};

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CustomGamemode {
    Vanilla = 0,
    Relax = 1,
    Autopilot = 2,
}

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

    pub fn custom_gamemode(&self) -> CustomGamemode {
        match self {
            Gamemode::Standard => CustomGamemode::Vanilla,
            Gamemode::Taiko => CustomGamemode::Vanilla,
            Gamemode::Catch => CustomGamemode::Vanilla,
            Gamemode::Mania => CustomGamemode::Vanilla,
            Gamemode::StandardRelax => CustomGamemode::Relax,
            Gamemode::TaikoRelax => CustomGamemode::Relax,
            Gamemode::CatchRelax => CustomGamemode::Relax,
            Gamemode::StandardAutopilot => CustomGamemode::Autopilot,
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
