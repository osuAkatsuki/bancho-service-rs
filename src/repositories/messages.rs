use crate::common::context::Context;
use crate::entities::channels::ChannelName;
use crate::entities::messages::{Message, MessageStatus};

/*pub async fn fetch_history<C: Context>(
    ctx: &C,
    sender_id: Option<i64>,
    recipient_id: Option<i64>,
    recipient_channel: Option<&str>,
    unread: Option<bool>,
    limit: usize,
    page: usize,
) -> anyhow::Result<Vec<Message>> {
    if recipient_id.is_some() && recipient_channel.is_some() {

    }

    let query = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " ORDER BY created_at DESC LIMIT ?,?"
    );
    let limit_offset = limit * page;
    let messages = sqlx::query_as::<_, Message>(query)
        .fetch_all(ctx.db())
        .bind(limit_offset)
        .bind(limit)
        .await?;
    Ok(messages)
}*/

pub async fn fetch_unread_messages<C: Context>(
    ctx: &C,
    recipient_id: i64,
) -> sqlx::Result<Vec<Message>> {
    const QUERY: &str = const_str::concat!(
        "SELECT m.id, m.sender_id, m.recipient_id, m.recipient_channel,",
        "m.content, m.unread, m.created_at, m.status, users.username as sender_name ",
        "FROM messages m INNER JOIN users ON sender_id = users.id ",
        "WHERE recipient_id = ? AND status = ? AND unread IS TRUE"
    );
    sqlx::query_as(QUERY)
        .bind(recipient_id)
        .bind(MessageStatus::Active.as_str())
        .fetch_all(ctx.db())
        .await
}

pub async fn mark_all_read<C: Context>(ctx: &C, recipient_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "UPDATE messages SET unread = FALSE ",
        "WHERE recipient_id = ? AND unread IS TRUE"
    );
    sqlx::query(QUERY)
        .bind(recipient_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn send<C: Context>(
    ctx: &C,
    sender_id: i64,
    recipient_channel: Option<ChannelName<'_>>,
    recipient_id: Option<i64>,
    message_content: &str,
    mark_as_unread: bool,
) -> sqlx::Result<()> {
    let recipient_channel = recipient_channel.map(|channel_name| channel_name.to_string());
    const QUERY: &str = const_str::concat!(
        "INSERT INTO messages (sender_id, recipient_id, recipient_channel, content, unread) ",
        "VALUES (?, ?, ?, ?, ?)"
    );
    sqlx::query(QUERY)
        .bind(sender_id)
        .bind(recipient_id)
        .bind(recipient_channel)
        .bind(message_content)
        .bind(mark_as_unread)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn message_count<C: Context>(
    ctx: &C,
    sender_id: i64,
    delta_seconds: u64,
) -> sqlx::Result<i64> {
    const QUERY: &str = const_str::concat!(
        "SELECT COUNT(*) FROM messages ",
        "WHERE sender_id = ? AND created_at > (CURRENT_TIMESTAMP - ?)"
    );
    let msg_count = sqlx::query_scalar(QUERY)
        .bind(sender_id)
        .bind(delta_seconds)
        .fetch_one(ctx.db())
        .await?;
    Ok(msg_count)
}

pub async fn delete_recent<C: Context>(
    ctx: &C,
    sender_id: i64,
    delta_seconds: u64,
) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "UPDATE messages SET status = ? ",
        "WHERE sender_id = ? AND created_at > (CURRENT_TIMESTAMP - ?)"
    );
    sqlx::query(QUERY)
        .bind(MessageStatus::Deleted.as_str())
        .bind(sender_id)
        .bind(delta_seconds)
        .execute(ctx.db())
        .await?;
    Ok(())
}
