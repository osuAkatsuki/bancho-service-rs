use crate::entities::channels::Channel as Entity;
use crate::models::privileges::Privileges;
use crate::repositories::streams::StreamName;

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

    pub fn get_update_stream_name(&self) -> StreamName {
        match self.name.as_str() {
            "#plus" | "#supporter" | "#premium" => StreamName::Donator,
            "#staff" => StreamName::Staff,
            "#devlog" => StreamName::Dev,
            _ => StreamName::Main,
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
