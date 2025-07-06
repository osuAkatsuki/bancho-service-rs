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
    pub mode: u8,
    pub mods: i32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub miss_count: u16,
}

#[derive(Deserialize)]
pub struct PerformanceResult {
    pub pp: f32,
    pub stars: f32,
}

pub async fn calculate_pp(
    perf_requests: &[PerformanceRequest],
) -> ServiceResult<Vec<PerformanceResult>> {
    let url = make_url("/api/v1/calculate");
    let response = CLIENT.post(url).json(perf_requests).send().await?;
    let results = response.json().await?;
    Ok(results)
}
