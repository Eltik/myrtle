use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OperatorOwnershipRow {
    pub operator_id: String,
    pub owners: i64,
    /// Owners who promoted this operator to E2. Always <= `owners`.
    ///
    /// Structurally zero for the 36 operators that cannot reach E2 at all (one
    /// to three stars carry fewer than three phases), so a zero here is not
    /// evidence that players declined to build them, and callers must not
    /// present it as a rate for those.
    pub e2_owners: i64,
}

/// One row of a per-operator choice distribution: how many users picked this
/// skill index, or this module id, as their default.
///
/// The whole distribution is kept rather than the winner alone. 12 of 364
/// operators have a modal share under 50%, and there the runner-up carries as
/// much information as the leader.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OperatorChoiceRow {
    /// 0-based skill index rendered as text, or the `uniEquipId` of an
    /// ADVANCED module. One shape so both distributions share a read path.
    pub choice: String,
    pub users: i64,
}

/// Per-server ownership counts plus the server's eligible population (the
/// denominator), read from the precomputed aggregate. Operators with no owners
/// are absent; callers treat a missing id as zero.
pub async fn get_operator_ownership(
    pool: &PgPool,
    server_id: i16,
) -> Result<(i64, Vec<OperatorOwnershipRow>), sqlx::Error> {
    // population is identical across every row for a server; pull it once.
    let population: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(population), 0)::BIGINT \
         FROM operator_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, OperatorOwnershipRow>(
        "SELECT operator_id, owners::BIGINT AS owners, e2_owners::BIGINT AS e2_owners \
         FROM operator_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok((population, rows))
}

/// The default-skill and default-module distributions for one operator, each
/// ordered most-picked first.
///
/// The ordering is `users DESC, choice ASC`, and the tie-break is load-bearing
/// rather than cosmetic: 4 operators tie exactly at the top of their
/// distribution, and an unordered read would name a different winner per
/// process the way `HashMap` iteration did for module order.
pub async fn get_operator_choices(
    pool: &PgPool,
    server_id: i16,
    operator_id: &str,
) -> Result<(Vec<OperatorChoiceRow>, Vec<OperatorChoiceRow>), sqlx::Error> {
    let skills = sqlx::query_as::<_, OperatorChoiceRow>(
        "SELECT skill_index::TEXT AS choice, users::BIGINT AS users \
         FROM operator_skill_choice_stats \
         WHERE server_id = $1 AND operator_id = $2 \
         ORDER BY users DESC, skill_index ASC",
    )
    .bind(server_id)
    .bind(operator_id)
    .fetch_all(pool)
    .await?;

    let modules = sqlx::query_as::<_, OperatorChoiceRow>(
        "SELECT uni_equip_id AS choice, users::BIGINT AS users \
         FROM operator_module_choice_stats \
         WHERE server_id = $1 AND operator_id = $2 \
         ORDER BY users DESC, uni_equip_id ASC",
    )
    .bind(server_id)
    .bind(operator_id)
    .fetch_all(pool)
    .await?;

    Ok((skills, modules))
}

/// One bucket of an investment histogram: how many users sit at this level.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OperatorLevelRow {
    /// Skill index or module id, matching the choice tables.
    pub key: String,
    /// Mastery 0..3, or module level 0..3 where 0 means "not unlocked".
    pub level: i16,
    pub users: i64,
}

/// Mastery-per-skill and level-per-module histograms for one operator.
///
/// Both are ordered by key then level ascending so a client can render them as
/// a strip without sorting, and both count E2 owners only: below E2 neither a
/// mastery nor a module level can exist, and the rows that do exist there carry
/// placeholder values rather than choices.
pub async fn get_operator_levels(
    pool: &PgPool,
    server_id: i16,
    operator_id: &str,
) -> Result<(Vec<OperatorLevelRow>, Vec<OperatorLevelRow>), sqlx::Error> {
    let masteries = sqlx::query_as::<_, OperatorLevelRow>(
        "SELECT skill_index::TEXT AS key, mastery AS level, users::BIGINT AS users \
         FROM operator_mastery_stats \
         WHERE server_id = $1 AND operator_id = $2 \
         ORDER BY skill_index ASC, mastery ASC",
    )
    .bind(server_id)
    .bind(operator_id)
    .fetch_all(pool)
    .await?;

    let module_levels = sqlx::query_as::<_, OperatorLevelRow>(
        "SELECT uni_equip_id AS key, module_level AS level, users::BIGINT AS users \
         FROM operator_module_level_stats \
         WHERE server_id = $1 AND operator_id = $2 \
         ORDER BY uni_equip_id ASC, module_level ASC",
    )
    .bind(server_id)
    .bind(operator_id)
    .fetch_all(pool)
    .await?;

    Ok((masteries, module_levels))
}

/// Timestamp of the most recent aggregate refresh, used to pace the background
/// job. `None` means it has never been computed.
pub async fn latest_ownership_refresh_at(
    pool: &PgPool,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar("SELECT MAX(computed_at) FROM operator_ownership_stats")
        .fetch_one(pool)
        .await
}

/// Recompute the per-server aggregates from scratch and atomically replace all
/// five tables. Measured at 62.531 ms over 619,706 `user_operators` rows, so
/// it is cheap, but it still runs only from the background job and never on a
/// request, because the cost grows with the roster while a request budget does
/// not.
///
/// Only users who opted into stat sharing are counted, for the owner tallies,
/// the choice distributions and the population denominator alike. The
/// denominator is the eligible users on each server who have imported a roster,
/// so percentages reflect the sharing population rather than every registered
/// account.
pub async fn refresh_build_stats(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM operator_ownership_stats")
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r"
        INSERT INTO operator_ownership_stats (server_id, operator_id, owners, e2_owners, population)
        SELECT
            u.server_id,
            uo.operator_id,
            COUNT(*)::INT AS owners,
            COUNT(*) FILTER (WHERE uo.elite = 2)::INT AS e2_owners,
            pop.population
        FROM user_operators uo
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        JOIN (
            SELECT u2.server_id, COUNT(DISTINCT uo2.user_id)::INT AS population
            FROM user_operators uo2
            JOIN users u2         ON u2.id = uo2.user_id
            JOIN user_settings s2 ON s2.user_id = u2.id
            WHERE s2.share_stats = true
            GROUP BY u2.server_id
        ) pop ON pop.server_id = u.server_id
        WHERE s.share_stats = true
        GROUP BY u.server_id, uo.operator_id, pop.population
        ",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM operator_skill_choice_stats")
        .execute(&mut *tx)
        .await?;

    // Restricted to E2 owners because a skill the operator has not unlocked
    // cannot have been chosen, and to default_skill >= 0 because -1 is the
    // game's "this operator has no skills" sentinel, not a skill. Coercing that
    // sentinel to a skill would put 26,070 rows on S1 that never picked it.
    sqlx::query(
        r"
        INSERT INTO operator_skill_choice_stats (server_id, operator_id, skill_index, users)
        SELECT u.server_id, uo.operator_id, uo.default_skill, COUNT(*)::INT
        FROM user_operators uo
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        WHERE s.share_stats = true
          AND uo.elite = 2
          AND uo.default_skill >= 0
        GROUP BY u.server_id, uo.operator_id, uo.default_skill
        ",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM operator_module_choice_stats")
        .execute(&mut *tx)
        .await?;

    // The regex admits only ADVANCED modules. NULL current_equip (72.75% of
    // rows) and uniequip_001_* (a further 13.62%, the ORIGINAL module, which is
    // Type_ = INITIAL for all 380 of them) both mean "no module chosen", and
    // counting either would drown the real preference in defaults nobody set.
    sqlx::query(
        r"
        INSERT INTO operator_module_choice_stats (server_id, operator_id, uni_equip_id, users)
        SELECT u.server_id, uo.operator_id, uo.current_equip, COUNT(*)::INT
        FROM user_operators uo
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        WHERE s.share_stats = true
          AND uo.current_equip ~ '^uniequip_0(0[2-9]|[1-9][0-9])'
        GROUP BY u.server_id, uo.operator_id, uo.current_equip
        ",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM operator_mastery_stats")
        .execute(&mut *tx)
        .await?;

    // Where a skill is LEFT, not whether it was touched. Restricted to E2
    // owners because mastery needs E2 and rows exist for every owned operator
    // regardless: 664,176 at E0 and 183,465 at E1, every one of them
    // specialize_level = 0. Counting those would bury each real figure under
    // operators that cannot be mastered at all, which is the same mistake as
    // reading a locked option as a declined one.
    sqlx::query(
        r"
        INSERT INTO operator_mastery_stats (server_id, operator_id, skill_index, mastery, users)
        SELECT u.server_id, uo.operator_id, sk.skill_index, sk.specialize_level, COUNT(*)::INT
        FROM user_operator_skills sk
        JOIN user_operators uo ON uo.user_id = sk.user_id AND uo.operator_id = sk.operator_id
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        WHERE s.share_stats = true
          AND uo.elite = 2
        GROUP BY u.server_id, uo.operator_id, sk.skill_index, sk.specialize_level
        ",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM operator_module_level_stats")
        .execute(&mut *tx)
        .await?;

    // `locked` is the discriminator, NOT the row's presence: a locked row always
    // carries module_level = 1, which is a placeholder rather than a level, and
    // such rows exist at E0 and E1 where a module cannot be equipped at all.
    // Remapping locked to 0 keeps "has not unlocked it" as its own bucket, which
    // is the largest stopping point there is and would otherwise masquerade as
    // Lv1.
    sqlx::query(
        r"
        INSERT INTO operator_module_level_stats (server_id, operator_id, uni_equip_id, module_level, users)
        SELECT u.server_id, uo.operator_id, m.module_id,
               (CASE WHEN m.locked THEN 0 ELSE m.module_level END)::SMALLINT,
               COUNT(*)::INT
        FROM user_operator_modules m
        JOIN user_operators uo ON uo.user_id = m.user_id AND uo.operator_id = m.operator_id
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        WHERE s.share_stats = true
          AND uo.elite = 2
        GROUP BY u.server_id, uo.operator_id, m.module_id,
                 (CASE WHEN m.locked THEN 0 ELSE m.module_level END)
        ",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
