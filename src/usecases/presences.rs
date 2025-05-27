use crate::api::RequestContext;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::Gamemode;
use crate::models::presences::Presence;
use crate::repositories::presences;
use bancho_protocol::structures::{Action, Country, Mods};

pub async fn create_default(
    ctx: &RequestContext,
    user_id: i64,
    ranked_score: u64,
    total_score: u64,
    accuracy: f64,
    playcount: u32,
    performance: u32,
    global_rank: usize,
    country: Country,
    latitude: f32,
    longitude: f32,
    utc_offset: i8,
) -> ServiceResult<Presence> {
    create(
        ctx,
        user_id,
        Action::Idle,
        "".to_string(),
        "".to_string(),
        0,
        Default::default(),
        Gamemode::Standard,
        ranked_score,
        total_score,
        accuracy,
        playcount,
        performance,
        global_rank,
        country,
        latitude,
        longitude,
        utc_offset,
    )
    .await
}

pub async fn create(
    ctx: &RequestContext,
    user_id: i64,
    action: Action,
    info_text: String,
    beatmap_md5: String,
    beatmap_id: i32,
    mods: Mods,
    mode: Gamemode,
    ranked_score: u64,
    total_score: u64,
    accuracy: f64,
    playcount: u32,
    performance: u32,
    global_rank: usize,
    country: Country,
    latitude: f32,
    longitude: f32,
    utc_offset: i8,
) -> ServiceResult<Presence> {
    match presences::create(
        ctx,
        user_id,
        action as _,
        info_text,
        beatmap_md5,
        beatmap_id,
        mods.bits(),
        mode as _,
        ranked_score,
        total_score,
        accuracy,
        playcount,
        performance,
        global_rank,
        country.code().to_string(),
        latitude,
        longitude,
        utc_offset,
    )
    .await
    {
        Ok(presence) => Presence::try_from(presence),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_one(ctx: &RequestContext, user_id: i64) -> ServiceResult<Presence> {
    match presences::fetch_one(ctx, user_id).await {
        Ok(Some(presence)) => Presence::try_from(presence),
        Ok(None) => Err(AppError::PresencesNotFound),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_multiple(
    ctx: &RequestContext,
    user_ids: &[i32],
) -> ServiceResult<Vec<(i32, Option<Presence>)>> {
    match presences::fetch_multiple(ctx, user_ids).await {
        Ok(presences) => user_ids
            .iter()
            .zip(presences)
            .map(|(user_id, presence)| match presence {
                None => Ok((*user_id, None)),
                Some(presence) => Ok((*user_id, Some(Presence::try_from(presence)?))),
            })
            .collect(),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_user_ids(ctx: &RequestContext) -> ServiceResult<Vec<i32>> {
    match presences::fetch_user_ids(ctx).await {
        Ok(user_ids) => Ok(user_ids),
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_all(ctx: &RequestContext) -> ServiceResult<Vec<Presence>> {
    match presences::fetch_all(ctx).await {
        Ok(presences) => presences.map(Presence::try_from).collect(),
        Err(e) => unexpected(e),
    }
}

pub async fn update(ctx: &RequestContext, presence: Presence) -> ServiceResult<Presence> {
    match presences::update(ctx, presence.clone().into()).await {
        Ok(_) => Ok(presence),
        Err(e) => unexpected(e),
    }
}

pub async fn delete(ctx: &RequestContext, user_id: i64) -> ServiceResult<()> {
    match presences::delete(ctx, user_id).await {
        Ok(_) => Ok(()),
        Err(e) => unexpected(e),
    }
}
