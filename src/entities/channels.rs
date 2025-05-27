#[derive(sqlx::FromRow)]
pub struct Channel {
    id: i64,
    pub name: String,
    pub description: String,
    pub public_read: bool,
    pub public_write: bool,
    pub status: bool,
    #[deprecated]
    temp: bool,
    #[deprecated]
    hidden: bool,
}
