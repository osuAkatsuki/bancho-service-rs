use crate::entities::badges::{Badge as BadgeEntity, UserBadge as UserBadgeEntity};

#[derive(Debug)]
pub struct Badge {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub colour: String,
}

impl From<BadgeEntity> for Badge {
    fn from(entity: BadgeEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            icon: entity.icon,
            colour: entity.colour,
        }
    }
}

#[derive(Debug)]
pub struct UserBadge {
    pub id: i32,
    pub user_id: i64,
    pub badge_id: i32,
}

impl From<UserBadgeEntity> for UserBadge {
    fn from(entity: UserBadgeEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user,
            badge_id: entity.badge,
        }
    }
}
