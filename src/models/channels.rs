use crate::entities::channels::Channel as Entity;
use crate::models::privileges::Privileges;

pub struct Channel {
    pub name: String,
    pub description: String,
    pub read_privileges: Privileges,
    pub write_privileges: Privileges,
    pub status: bool,
}

// TODO: modify channels table to remove this
fn privileges_from(name: &str, public: bool) -> Privileges {
    match name {
        "#plus" | "#supporter" | "#premium" => Privileges::Donator | Privileges::AkatsukiPlus,
        "#staff" => Privileges::AdminChatMod,
        "#devlog" => Privileges::AdminManagePrivileges,
        _ => {
            if public {
                Privileges::None
            } else {
                Privileges::AdminCaker
            }
        }
    }
}

impl Channel {
    pub fn can_read(&self, privs: Privileges) -> bool {
        self.read_privileges.is_empty() || privs.intersects(self.read_privileges)
    }

    pub fn can_write(&self, privs: Privileges) -> bool {
        self.write_privileges.is_empty() || privs.intersects(self.write_privileges)
    }

    pub fn spectator() -> Self {
        Self {
            name: "#spectator".to_owned(),
            description: "Spectator Channel".to_owned(),
            read_privileges: Privileges::None,
            write_privileges: Privileges::None,
            status: true,
        }
    }

    pub fn multiplayer() -> Self {
        Self {
            name: "#multiplayer".to_owned(),
            description: "Multiplayer Channel".to_owned(),
            read_privileges: Privileges::None,
            write_privileges: Privileges::None,
            status: false,
        }
    }
}

impl From<Entity> for Channel {
    fn from(value: Entity) -> Self {
        let read_privileges = privileges_from(&value.name, value.public_read);
        let write_privileges = privileges_from(&value.name, value.public_write);

        Self {
            name: value.name,
            description: value.description,
            status: value.status,
            read_privileges,
            write_privileges,
        }
    }
}
