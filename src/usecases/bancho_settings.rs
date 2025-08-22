use crate::common::context::Context;
use crate::common::error::ServiceResult;
use crate::repositories::bancho_settings;

const MAINTENANCE_KEY: &str = "bancho_maintenance";

pub async fn in_maintenance_mode<C: Context>(ctx: &C) -> ServiceResult<bool> {
    let current = bancho_settings::fetch(ctx, MAINTENANCE_KEY).await?;
    let is_active = current.value_int != 0;
    Ok(is_active)
}

pub async fn toggle_maintenance<C: Context>(ctx: &C) -> ServiceResult<bool> {
    let current = bancho_settings::fetch(ctx, MAINTENANCE_KEY).await?;
    let is_active = current.value_int != 0;
    let new_value = match is_active {
        true => 0,
        false => 1,
    };
    bancho_settings::update_int(ctx, MAINTENANCE_KEY, new_value).await?;
    Ok(!is_active)
}
