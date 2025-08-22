use crate::common::error::ServiceResult;
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

fn make_url(endpoint: &str) -> String {
    let settings = AppSettings::get();
    format!("{}{endpoint}", settings.performance_service_base_url)
}

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Serialize)]
pub struct PerformanceRequest {
    pub beatmap_id: i32,
    pub beatmap_md5: String,
    pub mode: i32,
    pub mods: u32,
    pub max_combo: i32,
    pub accuracy: f32, // Option<f32>,
    /*pub count_300: Option<i32>,
    pub count_100: Option<i32>,
    pub count_50: Option<i32>,*/
    pub miss_count: i32,
}

#[derive(Deserialize)]
pub struct PerformanceResult {
    pub max_combo: i32,
    pub stars: f32,
    pub pp: f32,
    pub ar: f32,
    pub od: f32,
    #[serde(skip)]
    pub score_combo: i32,
    #[serde(skip)]
    pub accuracy: f32,
}

pub async fn calculate_pp(
    perf_requests: &[PerformanceRequest],
) -> ServiceResult<Vec<PerformanceResult>> {
    let url = make_url("/api/v1/calculate");
    let response = CLIENT.post(url).json(perf_requests).send().await?;
    let results: Vec<PerformanceResult> = response.json().await?;

    Ok(results
        .into_iter()
        .zip(perf_requests.iter())
        .map(|(mut res, req)| {
            // TODO: performance-service should return these attributes
            res.accuracy = req.accuracy;
            res.score_combo = req.max_combo;
            res
        })
        .collect())
}
