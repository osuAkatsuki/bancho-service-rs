use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Badge {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub colour: String,
}

#[derive(Debug, FromRow)]
pub struct UserBadge {
    pub id: i32,
    pub user: i64,
    pub badge: i32,
}
