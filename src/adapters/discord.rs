use crate::common::error::ServiceResult;
use crate::settings::AppSettings;
use discord_webhook2::message::Message;
use discord_webhook2::webhook::DiscordWebhook;
use iso8061_timestamp::Timestamp;

const INFO_COLOR: u32 = 0x6611FF;
const WARN_COLOR: u32 = 0x00a2ff;

pub async fn info(title: &str, description: &str, url: Option<&str>) -> ServiceResult<()> {
    send(title, description, url, INFO_COLOR).await
}

pub async fn warn(title: &str, description: &str, url: Option<&str>) -> ServiceResult<()> {
    send(title, description, url, WARN_COLOR).await
}

pub async fn send(
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
