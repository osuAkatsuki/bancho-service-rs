use crate::adapters::performance_service::{PerformanceRequest, calculate_pp};
use crate::common::error::ServiceResult;
use crate::models::performance::PerformanceRequestArgs;
use bancho_protocol::structures::Mods;
use std::fmt::Write;

const DEFAULT_ACCURACY_VALUES: [f32; 4] = [100.0, 99.0, 98.0, 95.0];

pub async fn fetch_pp_message(args: PerformanceRequestArgs) -> ServiceResult<String> {
    let pp_response = match args.extra {
        None => {
            let requests = default_pp_requests(&args);
            calculate_pp(&requests).await?
        }
        Some(extra) => {
            let perf_request = PerformanceRequest {
                beatmap_id: args.beatmap_id,
                beatmap_md5: args.beatmap_md5,
                mode: args.mode,
                mods: args.mods,
                max_combo: extra.max_combo.unwrap_or(args.beatmap_max_combo),
                accuracy: extra.accuracy.unwrap_or(100.0),
                miss_count: extra.miss_count.unwrap_or(0),
            };
            calculate_pp(&[perf_request]).await?
        }
    };
    let info = match pp_response.first() {
        Some(info) => info,
        None => return Ok("Failed calculating performance: empty response".to_owned()),
    };
    let stars = info.stars;
    let ar = info.ar;
    let od = info.od;

    // Generate the pp table
    let mut pp_display = String::with_capacity(pp_response.len() * 14);
    for res in pp_response {
        write!(&mut pp_display, "{}%: {:.2}pp | ", res.accuracy, res.pp)
            .expect("Can't write to string");
    }
    pp_display.truncate(pp_display.len() - 3);

    let pp_message = format!(
        "{} {} | {stars}★ | AR{ar:.2} OD{od:.2}\n✦ {pp_display}",
        args.beatmap_song_name,
        Mods::from_bits_truncate(args.mods),
    );
    Ok(pp_message)
}

fn default_pp_requests(args: &PerformanceRequestArgs) -> [PerformanceRequest; 4] {
    std::array::from_fn(|i| PerformanceRequest {
        beatmap_id: args.beatmap_id,
        beatmap_md5: args.beatmap_md5.clone(),
        mode: args.mode as _,
        mods: args.mods,
        max_combo: args.beatmap_max_combo,
        accuracy: DEFAULT_ACCURACY_VALUES[i], // Some(DEFAULT_ACCURACY_VALUES[i]),
        /*count_300: None,
        count_100: None,
        count_50: None,*/
        miss_count: 0,
    })
}
