use crate::common::context::{Context, PoolContext};
use crate::entities::news::NewsKind;

pub async fn create<C: Context>(
    ctx: &C,
    kind: NewsKind,
    title: &str,
    body: Option<&str>,
    url: Option<&str>,
    author_id: Option<i64>,
    beatmapset_id: Option<i32>,
    beatmap_id: Option<i32>,
) -> sqlx::Result<()> {
    const QUERY: &str = const_str::concat!(
        "INSERT INTO news ",
        "(kind, title, body, url, author_id, beatmapset_id, beatmap_id) ",
        "VALUES (?, ?, ?, ?, ?, ?, ?)",
    );
    sqlx::query(QUERY)
        .bind(kind.as_str())
        .bind(title)
        .bind(body)
        .bind(url)
        .bind(author_id)
        .bind(beatmapset_id)
        .bind(beatmap_id)
        .execute(ctx.db())
        .await?;
    Ok(())
}
