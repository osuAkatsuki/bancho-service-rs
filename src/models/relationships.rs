use crate::entities::relationships::Relationship as RelationshipEntity;

#[derive(Debug)]
pub struct Relationship {
    // The ID of the user that added the friend
    pub follower_id: i64,
    // The ID of the friend
    pub friend_id: i64,
}

impl From<RelationshipEntity> for Relationship {
    fn from(value: RelationshipEntity) -> Self {
        Self {
            follower_id: value.user1,
            friend_id: value.user2,
        }
    }
}
