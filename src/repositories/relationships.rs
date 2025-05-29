use crate::common::context::Context;
use crate::entities::relationships::Relationship;

const TABLE_NAME: &str = "users_relationships";
const READ_FIELDS: &str = "id, user1, user2";

pub async fn fetch_one<C: Context>(
    ctx: &C,
    follower_id: i64,
    friend_id: i64,
) -> sqlx::Result<Relationship> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE user1 = ? AND user2 = ?"
    );
    let user_ids = sqlx::query_as(QUERY)
        .bind(follower_id)
        .bind(friend_id)
        .fetch_one(ctx.db())
        .await?;
    Ok(user_ids)
}

pub async fn fetch_friends<C: Context>(ctx: &C, user_id: i64) -> sqlx::Result<Vec<Relationship>> {
    const QUERY: &str = const_str::concat!(
        "SELECT ",
        READ_FIELDS,
        " FROM ",
        TABLE_NAME,
        " WHERE user1 = ?"
    );
    let user_ids = sqlx::query_as(QUERY)
        .bind(user_id)
        .fetch_all(ctx.db())
        .await?;
    Ok(user_ids)
}

pub async fn add_friend<C: Context>(ctx: &C, user_id: i64, to_add: i64) -> sqlx::Result<()> {
    const QUERY: &str = "INSERT INTO users_relationships (user1, user2) VALUES (?, ?)";
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(to_add)
        .execute(ctx.db())
        .await?;
    Ok(())
}

pub async fn remove_friend<C: Context>(ctx: &C, user_id: i64, to_remove: i64) -> sqlx::Result<()> {
    const QUERY: &str = "DELETE FROM users_relationships WHERE user1 = ? AND user2 = ?";
    sqlx::query(QUERY)
        .bind(user_id)
        .bind(to_remove)
        .execute(ctx.db())
        .await?;
    Ok(())
}
