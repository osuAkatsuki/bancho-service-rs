#[derive(sqlx::FromRow)]
pub struct BanchoSetting {
    pub id: i32,
    pub name: String,
    pub value_int: i32,
    pub value_string: String,
}
