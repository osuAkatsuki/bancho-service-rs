use crate::entities::sessions::Session as SessionEntity;
use crate::models::privileges::Privileges;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: i64,
    pub privileges: Privileges,
    pub create_ip_address: IpAddr,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn is_publicly_visible(&self) -> bool {
        self.privileges.is_publicly_visible()
    }

    pub fn has_all_privileges(&self, privileges: Privileges) -> bool {
        privileges.is_empty() || self.privileges.contains(privileges)
    }

    pub fn has_any_privilege(&self, privileges: Privileges) -> bool {
        privileges.is_empty() || self.privileges.intersects(privileges)
    }

    pub fn as_entity(&self) -> SessionEntity {
        self.clone().into()
    }
}

impl Into<SessionEntity> for Session {
    fn into(self) -> SessionEntity {
        SessionEntity {
            session_id: self.session_id,
            user_id: self.user_id,
            privileges: self.privileges.bits(),
            create_ip_address: self.create_ip_address,
            updated_at: self.updated_at,
        }
    }
}

impl From<SessionEntity> for Session {
    fn from(value: SessionEntity) -> Self {
        Self {
            session_id: value.session_id,
            user_id: value.user_id,
            privileges: Privileges::from_bits_retain(value.privileges),
            create_ip_address: value.create_ip_address,
            updated_at: value.updated_at,
        }
    }
}
