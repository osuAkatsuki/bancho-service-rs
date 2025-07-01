#[derive(sqlx::FromRow)]
pub struct FirstPlaceScore {
    pub scoreid: i64,
    pub beatmap_md5: String,
    pub mode: u8,
    pub rx: u8,
}

#[derive(sqlx::FromRow)]
pub struct NewFirstPlace {
    pub id: i64,
    pub userid: i64,
}
