#[derive(sqlx::FromRow)]
pub struct MinimalScore {
    pub score_id: i64,
    pub score: i64,
    #[sqlx(rename = "pp")]
    pub performance: f32,
    pub user_id: i64,
    pub beatmap_md5: String,
    pub mode: i8,
    pub time: i32,
}

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
