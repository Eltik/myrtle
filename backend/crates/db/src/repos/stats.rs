use chrono::{DateTime, Utc};
use myrtle_types::Server;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct OperatorOwnershipRow {
    pub operator_id: String,
    pub owners: i64,
    pub e2_owners: i64,
}

#[derive(Debug, Clone)]
pub struct OperatorChoiceRow {
    pub choice: String,
    pub users: i64,
}

#[derive(Debug, Clone)]
pub struct OperatorLevelRow {
    /// Skill index as string
    pub key: String,
    /// Mastery, or module level with 0 for locked
    pub level: i16,
    pub users: i64,
}

#[derive(Debug, Clone)]
pub struct OwnershipCountsRow {
    pub ownership_rows: i64,
    pub e2_owners: i64,
    pub skill_choice_rows: i64,
    pub module_choice_rows: i64,
}

#[derive(Debug, Clone)]
pub struct MedalOwnershipRow {
    pub medal_id: String,
    pub owners: i64,
}

pub async fn operator_ownership(
    conn: &mut Conn,
    server: Server,
) -> Result<(i64, Vec<OperatorOwnershipRow>), DbError> {
    let rows = sqlx::query!(
        r#"SELECT operator_id, owners::bigint AS "owners!", e2_owners::bigint AS "e2_owners!",
                  population::bigint AS "population!"
           FROM operator_ownership_stats WHERE server_id = $1"#,
        server.index(),
    )
    .fetch_all(conn)
    .await?;
    let population = rows.first().map_or(0, |r| r.population);
    let rows = rows
        .into_iter()
        .map(|r| OperatorOwnershipRow {
            operator_id: r.operator_id,
            owners: r.owners,
            e2_owners: r.e2_owners,
        })
        .collect();
    Ok((population, rows))
}

/// Default-skill and default-module distributions
pub async fn operator_choices(
    conn: &mut Conn,
    server: Server,
    operator_id: &str,
) -> Result<(Vec<OperatorChoiceRow>, Vec<OperatorChoiceRow>), DbError> {
    let skills = sqlx::query_as!(
        OperatorChoiceRow,
        r#"SELECT skill_index::text AS "choice!", users::bigint AS "users!"
           FROM operator_skill_choice_stats
           WHERE server_id = $1 AND operator_id = $2
           ORDER BY users DESC, skill_index ASC"#,
        server.index(),
        operator_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    let modules = sqlx::query_as!(
        OperatorChoiceRow,
        r#"SELECT uni_equip_id AS "choice!", users::bigint AS "users!"
           FROM operator_module_choice_stats
           WHERE server_id = $1 AND operator_id = $2
           ORDER BY users DESC, uni_equip_id ASC"#,
        server.index(),
        operator_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok((skills, modules))
}

/// Mastery-per-skill and level-per-module histograms of E2 owners
pub async fn operator_levels(
    conn: &mut Conn,
    server: Server,
    operator_id: &str,
) -> Result<(Vec<OperatorLevelRow>, Vec<OperatorLevelRow>), DbError> {
    let masteries = sqlx::query_as!(
        OperatorLevelRow,
        r#"SELECT skill_index::text AS "key!", mastery AS level, users::bigint AS "users!"
           FROM operator_mastery_stats
           WHERE server_id = $1 AND operator_id = $2
           ORDER BY skill_index ASC, mastery ASC"#,
        server.index(),
        operator_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    let module_levels = sqlx::query_as!(
        OperatorLevelRow,
        r#"SELECT uni_equip_id AS key, module_level AS level, users::bigint AS "users!"
           FROM operator_module_level_stats
           WHERE server_id = $1 AND operator_id = $2
           ORDER BY uni_equip_id ASC, module_level ASC"#,
        server.index(),
        operator_id,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok((masteries, module_levels))
}

pub async fn operator_ownership_refreshed_at(
    conn: &mut Conn,
) -> Result<Option<DateTime<Utc>>, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT MAX(computed_at) AS "computed_at?" FROM operator_ownership_stats"#
    )
    .fetch_one(conn)
    .await?)
}

pub async fn refresh_operator_stats(conn: &mut Conn) -> Result<(), DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;

    sqlx::query!("DELETE FROM operator_ownership_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO operator_ownership_stats (server_id, operator_id, owners, e2_owners, population)
         SELECT
             u.server_id,
             uo.operator_id,
             COUNT(*)::int AS owners,
             COUNT(*) FILTER (WHERE uo.elite = 2)::int AS e2_owners,
             pop.population
         FROM user_operators uo
         JOIN users u         ON u.id = uo.user_id
         JOIN user_settings s ON s.user_id = u.id
         JOIN (
             SELECT u2.server_id, COUNT(DISTINCT uo2.user_id)::int AS population
             FROM user_operators uo2
             JOIN users u2         ON u2.id = uo2.user_id
             JOIN user_settings s2 ON s2.user_id = u2.id
             WHERE s2.share_stats = true
             GROUP BY u2.server_id
         ) pop ON pop.server_id = u.server_id
         WHERE s.share_stats = true
         GROUP BY u.server_id, uo.operator_id, pop.population"
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM operator_skill_choice_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO operator_skill_choice_stats (server_id, operator_id, skill_index, users)
         SELECT u.server_id, uo.operator_id, uo.default_skill, COUNT(*)::int
         FROM user_operators uo
         JOIN users u         ON u.id = uo.user_id
         JOIN user_settings s ON s.user_id = u.id
         WHERE s.share_stats = true
           AND uo.elite = 2
           AND uo.default_skill >= 0
         GROUP BY u.server_id, uo.operator_id, uo.default_skill"
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM operator_module_choice_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO operator_module_choice_stats (server_id, operator_id, uni_equip_id, users)
         SELECT u.server_id, uo.operator_id, uo.current_equip, COUNT(*)::int
         FROM user_operators uo
         JOIN users u         ON u.id = uo.user_id
         JOIN user_settings s ON s.user_id = u.id
         WHERE s.share_stats = true
           AND uo.current_equip ~ '^uniequip_0(0[2-9]|[1-9][0-9])'
         GROUP BY u.server_id, uo.operator_id, uo.current_equip"
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM operator_mastery_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO operator_mastery_stats (server_id, operator_id, skill_index, mastery, users)
         SELECT u.server_id, uo.operator_id, sk.skill_index, sk.specialize_level, COUNT(*)::int
         FROM user_operator_skills sk
         JOIN user_operators uo ON uo.user_id = sk.user_id AND uo.operator_id = sk.operator_id
         JOIN users u         ON u.id = uo.user_id
         JOIN user_settings s ON s.user_id = u.id
         WHERE s.share_stats = true
           AND uo.elite = 2
         GROUP BY u.server_id, uo.operator_id, sk.skill_index, sk.specialize_level"
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM operator_module_level_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO operator_module_level_stats
             (server_id, operator_id, uni_equip_id, module_level, users)
         SELECT u.server_id, uo.operator_id, m.module_id,
                (CASE WHEN m.locked THEN 0 ELSE m.module_level END)::smallint,
                COUNT(*)::int
         FROM user_operator_modules m
         JOIN user_operators uo ON uo.user_id = m.user_id AND uo.operator_id = m.operator_id
         JOIN users u         ON u.id = uo.user_id
         JOIN user_settings s ON s.user_id = u.id
         WHERE s.share_stats = true
           AND uo.elite = 2
         GROUP BY u.server_id, uo.operator_id, m.module_id,
                  (CASE WHEN m.locked THEN 0 ELSE m.module_level END)"
    )
    .execute(&mut *tx)
    .await?; // a locked module row has module_level 1 as a placeholder

    tx.commit().await?;
    Ok(())
}

pub async fn operator_stats_counts(conn: &mut Conn) -> Result<OwnershipCountsRow, DbError> {
    Ok(sqlx::query_as!(
        OwnershipCountsRow,
        r#"SELECT (SELECT COUNT(*) FROM operator_ownership_stats) AS "ownership_rows!",
                  (SELECT COALESCE(SUM(e2_owners), 0)::bigint FROM operator_ownership_stats)
                      AS "e2_owners!",
                  (SELECT COUNT(*) FROM operator_skill_choice_stats) AS "skill_choice_rows!",
                  (SELECT COUNT(*) FROM operator_module_choice_stats) AS "module_choice_rows!""#
    )
    .fetch_one(conn)
    .await?)
}

pub async fn medal_ownership(
    conn: &mut Conn,
    server: Server,
) -> Result<(i64, Vec<MedalOwnershipRow>), DbError> {
    let rows = sqlx::query!(
        r#"SELECT medal_id, owners::bigint AS "owners!", population::bigint AS "population!"
           FROM medal_ownership_stats WHERE server_id = $1"#,
        server.index(),
    )
    .fetch_all(conn)
    .await?;
    let population = rows.first().map_or(0, |r| r.population);
    let rows = rows
        .into_iter()
        .map(|r| MedalOwnershipRow {
            medal_id: r.medal_id,
            owners: r.owners,
        })
        .collect();
    Ok((population, rows))
}

pub async fn medal_ownership_refreshed_at(
    conn: &mut Conn,
) -> Result<Option<DateTime<Utc>>, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT MAX(computed_at) AS "computed_at?" FROM medal_ownership_stats"#
    )
    .fetch_one(conn)
    .await?)
}

pub async fn refresh_medal_ownership(conn: &mut Conn) -> Result<(), DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;

    sqlx::query!("DELETE FROM medal_ownership_stats")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO medal_ownership_stats (server_id, medal_id, owners, population)
         SELECT
             u.server_id,
             um.medal_id,
             COUNT(*)::int AS owners,
             pop.population
         FROM user_medals um
         JOIN users u         ON u.id = um.user_id
         JOIN user_settings s ON s.user_id = u.id
         JOIN (
             SELECT u2.server_id, COUNT(DISTINCT um2.user_id)::int AS population
             FROM user_medals um2
             JOIN users u2         ON u2.id = um2.user_id
             JOIN user_settings s2 ON s2.user_id = u2.id
             WHERE s2.share_stats = true
             GROUP BY u2.server_id
         ) pop ON pop.server_id = u.server_id
         WHERE s.share_stats = true
           AND (
               COALESCE(um.reach_ts, 0) > 0
               OR (
                   COALESCE(um.reach_ts, 0) = -1
                   AND COALESCE(um.first_ts, 0) > 0
                   AND jsonb_typeof(um.val) = 'array'
                   AND NOT EXISTS (
                       SELECT 1
                       FROM jsonb_array_elements(um.val) AS cond
                       WHERE NOT CASE
                           WHEN jsonb_typeof(cond -> 0) = 'number' AND jsonb_typeof(cond -> 1) = 'number'
                                AND cond ->> 0 ~ '^-?[0-9]+$' AND cond ->> 1 ~ '^-?[0-9]+$'
                           THEN LEAST((cond ->> 0)::numeric, (cond ->> 1)::numeric) >= -9223372036854775808
                                AND GREATEST((cond ->> 0)::numeric, (cond ->> 1)::numeric) <= 9223372036854775807
                                AND (cond ->> 0)::numeric >= (cond ->> 1)::numeric
                           ELSE false
                       END
                   )
               )
           )
         GROUP BY u.server_id, um.medal_id, pop.population"
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn medal_ownership_rows(conn: &mut Conn) -> Result<i64, DbError> {
    Ok(
        sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM medal_ownership_stats"#)
            .fetch_one(conn)
            .await?,
    )
}
