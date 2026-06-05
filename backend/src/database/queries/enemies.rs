use sqlx::PgPool;
use uuid::Uuid;

/// Return the set of enemy IDs the user has encountered (the keys of the
/// stored `dexNav.enemy.enemies` map). Empty if the user has never synced or
/// the blob is absent.
pub async fn get_user_encountered_enemies(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT key
        FROM user_enemy_progress uep
        CROSS JOIN LATERAL jsonb_object_keys(uep.enemies) AS key
        WHERE uep.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}
