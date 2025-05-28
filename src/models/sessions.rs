use crate::entities::sessions::Session as SessionEntity;
use crate::models::privileges::Privileges;
use chrono::{DateTime, Utc};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: i64,
    pub username: String,
    pub privileges: Privileges,
    pub create_ip_address: IpAddr,
    pub private_dms: bool,
    pub silence_end: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn is_publicly_visible(&self) -> bool {
        self.privileges.is_publicly_visible()
    }

    pub fn is_silenced(&self) -> bool {
        self.silence_end
            .is_some_and(|silence_end| silence_end > Utc::now())
    }

    pub fn silence_left(&self) -> i64 {
        self.silence_end
            .map(|silence_end| {
                let time_left = silence_end - Utc::now();
                time_left.num_seconds()
            })
            .unwrap_or(0)
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
            username: self.username,
            privileges: self.privileges.bits(),
            create_ip_address: self.create_ip_address,
            private_dms: self.private_dms,
            silence_end: self.silence_end,
            updated_at: self.updated_at,
        }
    }
}

impl From<SessionEntity> for Session {
    fn from(value: SessionEntity) -> Self {
        Self {
            session_id: value.session_id,
            user_id: value.user_id,
            username: value.username,
            privileges: Privileges::from_bits_retain(value.privileges),
            create_ip_address: value.create_ip_address,
            private_dms: value.private_dms,
            silence_end: value.silence_end,
            updated_at: value.updated_at,
        }
    }
}
