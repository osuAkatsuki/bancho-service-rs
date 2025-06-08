use crate::adapters::discord;
use crate::common::context::Context;
use crate::common::error::{AppError, ServiceResult};
use crate::models::bancho::ClientHashes;
use crate::models::privileges::Privileges;
use crate::repositories::{hardware_logs, users};
use rust_decimal::prelude::ToPrimitive;

pub async fn perform_checks<C: Context>(
    ctx: &C,
    user_id: i64,
    user_verification_pending: bool,
    client_hashes: &ClientHashes,
) -> ServiceResult<()> {
    let mac = &client_hashes.adapters_md5;
    let unique_id = &client_hashes.uninstall_md5;
    let disk_id = &client_hashes.disk_signature_md5;
    let user_hw_entry = hardware_logs::fetch_one(ctx, user_id, &mac, &unique_id, &disk_id).await?;
    let user_hw_entry_count = user_hw_entry.occurencies.to_i64().unwrap();

    let hwid_matches =
        hardware_logs::fetch_potential_multiaccounts(ctx, user_id, mac, unique_id, disk_id).await?;

    if user_verification_pending && !hwid_matches.is_empty() {
        users::ban(ctx, user_id).await?;
        let _ = discord::warn("", "", None).await?;
        for hw_match in hwid_matches {
            let privs = Privileges::from_bits_retain(hw_match.user_privileges);
            if hw_match.activated && privs.is_publicly_visible() {
                users::restrict(ctx, user_id).await?;
            }
        }

        Err(AppError::SessionsLoginForbidden)
    } else {
        // TODO: make this not stupid
        for hw_match in hwid_matches {
            let privs = Privileges::from_bits_retain(hw_match.user_privileges);
            if hw_match.activated {
                let match_id = hw_match.user_id;
                if privs.is_publicly_visible() {
                    let message = format!("{user_id} logged in from {match_id}'s hardware");
                    let _ = discord::warn("User login hardware match", &message, None).await;
                } else {
                    let message = format!(
                        "{user_id} logged in with {match_id}'s hardware, who is restricted"
                    );
                    let _ = discord::warn("User login hardware match", &message, None).await;
                    users::restrict(ctx, user_id).await?;
                }
            } else {
                let message = format!(
                    "Possible multiaccount: {} logged in with {}'s hardware {} times.\n{} has logged in {} times.",
                    user_id,
                    hw_match.user_id,
                    user_hw_entry_count,
                    hw_match.user_id,
                    hw_match.occurencies,
                );
                let _ = discord::warn("User login hardware match", &message, None).await;
            }
        }

        Ok(())
    }
}

pub async fn create_entry<C: Context>(
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
