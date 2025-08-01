use crate::common::context::Context;
use crate::entities::channels::ChannelName;
use crate::entities::messages::Message;
use chrono::Utc;

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
        "m.content, m.read_at, m.created_at, m.deleted_at, users.username as sender_name ",
        "FROM messages m INNER JOIN users ON sender_id = users.id ",
        "WHERE recipient_id = ? AND deleted_at IS NULL AND read_at IS NULL"
    );
    sqlx::query_as(QUERY)
        .bind(recipient_id)
        .fetch_all(ctx.db())
        .await
}

pub async fn mark_all_read<C: Context>(ctx: &C, recipient_id: i64) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "UPDATE messages SET read_at = CURRENT_TIMESTAMP ",
        "WHERE recipient_id = ? AND read_at IS NULL"
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
    sender_name: &str,
    recipient_channel: Option<ChannelName<'_>>,
    recipient_id: Option<i64>,
    message_content: &str,
    mark_as_unread: bool,
) -> sqlx::Result<Message> {
    let recipient_channel = recipient_channel.map(|channel_name| channel_name.to_string());
    const QUERY: &str = const_str::concat!(
        "INSERT INTO messages (sender_id, recipient_id, recipient_channel, content, read_at) ",
        "VALUES (?, ?, ?, ?, ?)"
    );
    let created_at = Utc::now();
    let read_at = match mark_as_unread {
        true => None,
        false => Some(created_at),
    };
    let query_result = sqlx::query(QUERY)
        .bind(sender_id)
        .bind(recipient_id)
        .bind(&recipient_channel)
        .bind(message_content)
        .bind(read_at)
        .execute(ctx.db())
        .await?;
    let message_id = query_result.last_insert_id();
    Ok(Message {
        sender_id,
        recipient_id,
        recipient_channel,
        created_at,
        read_at,
        id: message_id,
        sender_name: sender_name.to_owned(),
        content: message_content.to_string(),
        deleted_at: None,
    })
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
        "UPDATE messages SET deleted_at = CURRENT_TIMESTAMP ",
        "WHERE sender_id = ? AND created_at > (CURRENT_TIMESTAMP - ?)"
    );
    sqlx::query(QUERY)
        .bind(sender_id)
        .bind(delta_seconds)
        .execute(ctx.db())
        .await?;
    Ok(())
}
