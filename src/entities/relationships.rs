#[derive(Debug, sqlx::FromRow)]
pub struct Relationship {
    pub id: i64,
    pub user1: i64,
    pub user2: i64,
}
