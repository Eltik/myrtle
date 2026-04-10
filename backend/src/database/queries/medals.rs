use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_user_medals(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(String, Option<i64>, Option<i64>)>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT medal_id, first_ts, reach_ts
        FROM user_medals
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
