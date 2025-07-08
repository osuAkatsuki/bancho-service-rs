use crate::common::error::{AppError, ServiceResult};
use crate::settings::AppSettings;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

fn make_url(endpoint: &str) -> String {
    let settings = AppSettings::get();
    format!("{}{endpoint}", settings.beatmaps_service_base_url)
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Deserialize)]
pub struct AkatsukiBeatmap {
    pub beatmap_id: i32,
    pub beatmapset_id: i32,
    pub beatmap_md5: String,
    pub song_name: String,
    pub file_name: String,
    pub ar: f32,
    pub od: f32,
    pub mode: u8,
    pub max_combo: i32,
    pub hit_length: i32,
    pub bpm: i32,
    pub ranked: i8,
    pub latest_update: i64,
    pub ranked_status_freezed: bool,
    pub playcount: i64,
    pub passcount: i64,
    pub rankedby: Option<i64>,
    pub rating: f32,
    pub bancho_ranked_status: Option<i8>,
    pub count_circles: Option<i32>,
    pub count_spinners: Option<i32>,
    pub count_sliders: Option<i32>,
    pub bancho_creator_id: Option<i32>,
    pub bancho_creator_name: Option<String>,
}

#[derive(Serialize)]
struct BeatmapLookupQuery {
    beatmap_id: i32,
}

pub async fn fetch_by_id(beatmap_id: i32) -> ServiceResult<AkatsukiBeatmap> {
    let url = make_url("/api/akatsuki/v1/beatmaps/lookup");
    let response = CLIENT
        .get(url)
        .query(&BeatmapLookupQuery { beatmap_id })
        .send()
        .await?;
    match response.status() {
        StatusCode::NOT_FOUND => Err(AppError::BeatmapsNotFound),
        _ => Ok(response.json().await?),
    }
}
