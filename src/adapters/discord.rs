use crate::common::error::ServiceResult;
use crate::settings::AppSettings;
use discord_webhook2::message::Message;
use discord_webhook2::webhook::DiscordWebhook;
use iso8061_timestamp::Timestamp;

const PURPLE_EMBED_COLOR: u32 = 0x6611FF;
const RED_EMBED_COLOR: u32 = 0xFF5555;
const BLUE_EMBED_COLOR: u32 = 0x00a2ff;

pub async fn send_purple_embed(title: &str, description: &str, url: Option<&str>) -> ServiceResult<()> {
    send_embed(title, description, url, PURPLE_EMBED_COLOR).await
}

pub async fn send_blue_embed(title: &str, description: &str, url: Option<&str>) -> ServiceResult<()> {
    send_embed(title, description, url, BLUE_EMBED_COLOR).await
}

pub async fn send_red_embed(title: &str, description: &str, url: Option<&str>) -> ServiceResult<()> {
    send_embed(title, description, url, RED_EMBED_COLOR).await
}

pub async fn send_embed(
    title: &str,
    description: &str,
    url: Option<&str>,
    color: u32,
) -> ServiceResult<()> {
    let settings = AppSettings::get();
    if settings.discord_webhook_url.is_none() {
        tracing::warn!(title, description, url, "Discord Webhook url not set");
        return Ok(());
    }

    let webhook_url = settings.discord_webhook_url.as_ref().unwrap();
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
