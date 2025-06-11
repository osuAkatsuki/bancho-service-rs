use crate::adapters::discord;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult, unexpected};
use crate::models::bancho::ClientHashes;
use crate::models::hardware_logs::{
    AggregateHardwareInfo, AggregateHardwareMatch, AggregateMatchingHardwareResult,
    UserAggregateHardware,
};
use crate::repositories::hardware_logs;

pub async fn create<C: Context>(
    ctx: &C,
    user_id: i64,
    activation: bool,
    client_hashes: &ClientHashes,
) -> ServiceResult<()> {
    hardware_logs::create(
        ctx,
        user_id,
        activation,
        &client_hashes.adapters_md5,
        &client_hashes.uninstall_md5,
        &client_hashes.disk_signature_md5,
    )
    .await?;
    Ok(())
}

pub async fn fetch_self_aggregate_match<C: Context>(
    ctx: &C,
    user_id: i64,
    user_pending_verification: bool,
    client_hashes: &ClientHashes,
) -> ServiceResult<UserAggregateHardware> {
    let mac = &client_hashes.adapters_md5;
    let unique_id = &client_hashes.uninstall_md5;
    let disk_id = &client_hashes.disk_signature_md5;

    match hardware_logs::fetch_own_matching_hardware(ctx, user_id, mac, unique_id, disk_id).await {
        Ok(entries) => match entries.is_empty() {
            true => Ok(UserAggregateHardware {
                info: AggregateHardwareInfo::new([mac.clone(), unique_id.clone(), disk_id.clone()]),
                total_occurrences: 1,
                has_activated_hardware: user_pending_verification,
            }),
            false => Ok(UserAggregateHardware::from(entries)),
        },
        Err(e) => unexpected(e),
    }
}

pub async fn fetch_aggregate_hardware_matches<C: Context>(
    ctx: &C,
    user_id: i64,
    client_hashes: &ClientHashes,
) -> ServiceResult<AggregateMatchingHardwareResult> {
    let mac = &client_hashes.adapters_md5;
    let unique_id = &client_hashes.uninstall_md5;
    let disk_id = &client_hashes.disk_signature_md5;

    let hw_match_entries =
        hardware_logs::fetch_foreign_matching_hardware(ctx, user_id, mac, unique_id, disk_id)
            .await?;
    Ok(AggregateHardwareMatch::aggregate_by_user(hw_match_entries))
}

pub async fn check_for_multiaccounts<C: Context>(
    ctx: &C,
    user_id: i64,
    username: &str,
    user_verification_pending: bool,
    client_hashes: &ClientHashes,
) -> ServiceResult<()> {
    let hw_matches = fetch_aggregate_hardware_matches(ctx, user_id, client_hashes).await?;
    if hw_matches.total_hardware_matches == 0 {
        return Ok(());
    }

    match user_verification_pending {
        true => do_verification_checks(ctx, user_id, username, hw_matches).await,
        false => do_regular_checks(ctx, user_id, username, client_hashes, hw_matches).await,
    }
}

async fn do_verification_checks<C: Context>(
    _ctx: &C,
    user_id: i64,
    username: &str,
    hw_matches: AggregateMatchingHardwareResult,
) -> ServiceResult<()> {
    // users::ban(ctx, user_id).await?;
    let notification = format!("[{username}]({user_id})");
    let _ = discord::red("User banned (Multiaccount)", &notification, None).await;
    for (match_user_id, hw_match) in hw_matches.user_matches {
        let match_username = &hw_match.username;
        if hw_match.has_activated_hardware {
            match hw_match.user_privileges.is_publicly_visible() {
                true => {
                    // If the user is not restricted yet,
                    // restrict them and send the appropriate log message to discord
                    // users::restrict(ctx, match_user_id).await?;
                    let notification = format!(
                        "Restricted [{}]({}) for creating a multiaccount [{}]({})",
                        match_username, match_user_id, username, user_id,
                    );
                    let _ = discord::red("User restricted", &notification, None).await;
                }
                false => {
                    let notification = format!(
                        "[{match_username}]({match_user_id}) created multiaccount [{username}]({user_id})",
                    );
                    let _ = discord::red("User created multiaccount", &notification, None).await;
                }
            }
        } else {
            let usage_percent =
                hw_match.total_occurrences as f32 / hw_matches.total_hardware_matches as f32;
            let notification = format!(
                "[{}]({}) may have created a multiaccount [{}]({})\nHardware Usage Percentage: {:.2}%",
                match_username, match_user_id, username, user_id, usage_percent,
            );
            let _ = discord::red("Possible multiaccount association", &notification, None).await;
        }
    }

    Err(AppError::SessionsLoginForbidden)
}

async fn do_regular_checks<C: Context>(
    ctx: &C,
    user_id: i64,
    username: &str,
    client_hashes: &ClientHashes,
    hw_matches: AggregateMatchingHardwareResult,
) -> ServiceResult<()> {
    let user_hw_entry = fetch_self_aggregate_match(ctx, user_id, false, client_hashes).await?;

    let total_hardware_usages =
        (hw_matches.total_hardware_matches + user_hw_entry.total_occurrences) as f32;
    let usages = user_hw_entry.total_occurrences as f32;
    let usage_percent = usages / total_hardware_usages;
    for (match_user_id, hw_match) in hw_matches.user_matches {
        let match_username = &hw_match.username;
        let match_usage_percent = hw_match.total_occurrences as f32 / total_hardware_usages;
        if hw_match.has_activated_hardware {
            if hw_match.user_privileges.is_publicly_visible() {
                if usage_percent > 0.1 {
                    let notification = format!(
                        "[{}]({}) ({:.2}%) has logged in with [{}]({}) ({:.2}%)'s hardware too often!",
                        username,
                        user_id,
                        usage_percent * 100.0,
                        match_username,
                        match_user_id,
                        match_usage_percent * 100.0,
                    );
                    let _ = discord::red("Possible multiaccount", &notification, None).await;
                } else {
                    let notification = format!(
                        "[{username}]({user_id}) logged in with [{match_username}]({match_user_id})'s hardware."
                    );
                    let _ = discord::blue(
                        "User logged in with another users' hardware",
                        &notification,
                        None,
                    )
                    .await;
                }
            } else {
                // users::ban(ctx, user_id).await?;
                let notification = format!(
                    "[{username}]({user_id}) logged in with [{match_username}]({match_user_id})'s hardware, who is restricted."
                );
                let _ =
                    discord::red("Banned User (Possible Multiaccount)", &notification, None).await;
                return Err(AppError::SessionsLoginForbidden);
            }
        } else {
            if !hw_match.user_privileges.is_publicly_visible() {
                if match_usage_percent > 0.2 {
                    // users::ban(ctx, user_id).await?;
                    let notification = format!(
                        "[{username}]({user_id}) logged in with hardware used more than 20% by [{match_username}]({match_user_id}), who is restricted."
                    );
                    let _ =
                        discord::red("Banned User (Possible Multiaccount)", &notification, None)
                            .await;
                    return Err(AppError::SessionsLoginForbidden);
                } else {
                    let notification = format!(
                        "[{username}]({user_id}) has hardware match with [{match_username}]({match_user_id}), who is restricted."
                    );
                    let _ = discord::blue("Possible Multiaccount", &notification, None).await;
                }
            } else if match_usage_percent > usage_percent {
                let notification = format!(
                    "[{}]({}) ({:.2}%) logged in with hardware used more by [{}]({}) ({:.2}%)",
                    username,
                    user_id,
                    usage_percent * 100.0,
                    match_username,
                    match_user_id,
                    match_usage_percent * 100.0,
                );
                let _ = discord::blue(
                    "User logged in with another users' hardware",
                    &notification,
                    None,
                )
                .await;
            }
        }
    }
    Ok(())
}
