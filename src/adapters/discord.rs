use crate::common::error::ServiceResult;
use crate::common::{osu_assets, website};
use crate::models::beatmaps::{Beatmap, RankedStatus};
use crate::settings::AppSettings;
use bancho_protocol::structures::Mode;
use dedent::dedent;
use discord_webhook2::message::Message;
use discord_webhook2::webhook::DiscordWebhook;
use iso8061_timestamp::Timestamp;

const LOGS_PURPLE_EMBED_COLOR: u32 = 0x6611FF;
const LOGS_RED_EMBED_COLOR: u32 = 0xFF5555;
const LOGS_BLUE_EMBED_COLOR: u32 = 0x00a2ff;

const BEATMAPS_RANKED_EMBED_COLOR: u32 = 0x6611FF;
const BEATMAPS_LOVED_EMBED_COLOR: u32 = 0xFF66AA;
const BEATMAPS_PENDING_EMBED_COLOR: u32 = 0x696969;

pub async fn send_logs_purple_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_logs_embed(title, description, url, LOGS_PURPLE_EMBED_COLOR).await
}

pub async fn send_logs_blue_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_logs_embed(title, description, url, LOGS_BLUE_EMBED_COLOR).await
}

pub async fn send_logs_red_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
) -> ServiceResult<()> {
    send_logs_embed(title, description, url, LOGS_RED_EMBED_COLOR).await
}

pub async fn send_logs_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    if settings.discord_logs_webhook_url.is_none() {
        tracing::warn!(title, description, url, "Discord Logs Webhook url not set");
        return Ok(());
    }

    let webhook_url = settings.discord_logs_webhook_url.as_ref().unwrap();
    let webhook = DiscordWebhook::new(webhook_url)?;
    webhook
        .send(&Message::new(|message| {
            message.embed(|embed| {
                embed
                    .description(description)
                    .author(|author| {
                        let author = author.name(title);
                        match url {
                            Some(url) => author.url(url),
                            None => author,
                        }
                    })
                    .color(color)
                    .footer(|footer| footer.text("bancho-service ✈️"))
                    .timestamp(Timestamp::now_utc())
            })
        }))
        .await?;

    Ok(())
}

fn get_status_emoji_id(status: RankedStatus) -> &'static str {
    match status {
        RankedStatus::Loved => "1166976753869279272",
        RankedStatus::Ranked => "1166976760424964126",
        _ => "1166976756230651934",
    }
}

fn get_emoji_link(emoji_id: &str) -> String {
    format!("https://cdn.discordapp.com/emojis/{emoji_id}.png")
}

const AKATSUKI_ICON: &str = "<:akatsuki:1253876231645171814>";

pub async fn send_ranked_maps_embed(
    beatmap: &Beatmap,
    previous_status: RankedStatus,
    username: &str,
    user_id: i64,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    if settings.discord_ranked_maps_webhook_url.is_none() {
        tracing::warn!("Discord Ranked Maps Webhook url not set");
        return Ok(());
    }

    let status_color = match beatmap.ranked_status {
        RankedStatus::Ranked => BEATMAPS_RANKED_EMBED_COLOR,
        RankedStatus::Loved => BEATMAPS_LOVED_EMBED_COLOR,
        _ => BEATMAPS_PENDING_EMBED_COLOR,
    };
    let status_emoji = get_status_emoji_id(beatmap.ranked_status);
    let previous_status_emoji = get_status_emoji_id(previous_status);

    let mode_emoji = match beatmap.mode {
        Mode::Standard => "<:modeosu:1087863892308410398>",
        Mode::Taiko => "<:modetaiko:1087863916278853662>",
        Mode::Catch => "<:modefruits:1087863938982612994>",
        Mode::Mania => "<:modemania:1087863868782547014>",
    };

    let webhook_url = settings.discord_ranked_maps_webhook_url.as_ref().unwrap();
    let webhook = DiscordWebhook::new(webhook_url)?;

    let beatmap_mins = beatmap.hit_length / 60;
    let beatmap_secs = beatmap.hit_length % 60;

    let title = format!("{mode_emoji} {}", beatmap.song_name);
    let thumbnail = get_emoji_link(status_emoji);
    let map_cover = osu_assets::get_beatmap_cover_url(beatmap.beatmapset_id);
    let beatmap_link = website::get_beatmap_link(beatmap.beatmap_id);
    let description = format!(
        dedent!(
            r#"
            ### This map has received a status update. 📝
            **Length**: `{}:{}` **BPM**: `{}`
            **AR**: `{:.2}` **OD**: `{:.2}` **Combo**: `{}x
            "#
        ),
        beatmap_mins, beatmap_secs, beatmap.bpm, beatmap.ar, beatmap.od, beatmap.max_combo,
    );
    webhook
        .send(&Message::new(|message| {
            message.embed(|embed| {
                embed
                    .title(title)
                    .url(beatmap_link.clone())
                    .thumbnail(|t| t.url(thumbnail))
                    .image(|i| i.url(map_cover))
                    .author(|author| {
                        let ranked_by = format!("{} ({})", username, user_id);
                        let profile_url = website::get_profile_link(user_id);
                        author.name(ranked_by).url(profile_url)
                    })
                    .description(description)
                    .field(|field| {
                        let previous_status_text = format!(
                            "<:{previous_status:?}:{previous_status_emoji}>・{previous_status:?}"
                        );
                        field.name("Previous Status").value(previous_status_text)
                    })
                    .field(|field| {
                        let leaderboard_text =
                            format!("\n{AKATSUKI_ICON}・[`Akatsuki`]({beatmap_link})");
                        field.name("Leaderboard").value(leaderboard_text)
                    })
                    .color(status_color)
                    .footer(|footer| footer.text("bancho-service 🎵"))
                    .timestamp(Timestamp::now_utc())
            })
        }))
        .await?;

    Ok(())
}
