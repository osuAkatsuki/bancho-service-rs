#[derive(sqlx::FromRow)]
pub struct UserReport {
    pub id: i64,
    pub reason: String,
    // TODO: make the database store time as datetime instead of varchar(18) (?????)
    pub time: String,
    pub from_uid: i64,
    pub to_uid: i64,
}
