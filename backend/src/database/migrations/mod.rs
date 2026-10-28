use sqlx::PgPool;

/// Ordered migrations.
///
/// `v001_initial`..`v005_indexes` are the squashed baseline (`pg_dump` of the old
/// v001..v022, split by object type). They keep the old names on purpose: a database
/// that applied the originals already has them in `_migrations` and skips them, so
/// the bodies only run on a fresh database. New changes go after the baseline.
const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_views", include_str!("v002_views.sql")),
    ("v003_triggers", include_str!("v003_triggers.sql")),
    ("v004_procedures", include_str!("v004_procedures.sql")),
    ("v005_indexes", include_str!("v005_indexes.sql")),
    (
        "v006_cumulative_signin",
        include_str!("v006_cumulative_signin.sql"),
    ),
    (
        "v007_performance_indexes",
        include_str!("v007_performance_indexes.sql"),
    ),
    (
        "v008_medal_ownership",
        include_str!("v008_medal_ownership.sql"),
    ),
    (
        "v009_profile_sync_ts",
        include_str!("v009_profile_sync_ts.sql"),
    ),
    ("v010_base_facts", include_str!("v010_base_facts.sql")),
    (
        "v011_base_grade_components",
        include_str!("v011_base_grade_components.sql"),
    ),
    (
        "v012_game_credentials",
        include_str!("v012_game_credentials.sql"),
    ),
    (
        "v013_checkin_semantics",
        include_str!("v013_checkin_semantics.sql"),
    ),
    ("v014_build_stats", include_str!("v014_build_stats.sql")),
    (
        "v015_investment_levels",
        include_str!("v015_investment_levels.sql"),
    ),
    (
        "v016_release_ledger",
        include_str!("v016_release_ledger.sql"),
    ),
    (
        "v017_release_override_names",
        include_str!("v017_release_override_names.sql"),
    ),
    (
        "v018_release_override_rosters",
        include_str!("v018_release_override_rosters.sql"),
    ),
    (
        "v019_user_status_originite",
        include_str!("v019_user_status_originite.sql"),
    ),
    ("v020_release_plans", include_str!("v020_release_plans.sql")),
    ("v021_i18n", include_str!("v021_i18n.sql")),
    ("v022_source_locale", include_str!("v022_source_locale.sql")),
    (
        "v023_translation_source_snapshot",
        include_str!("v023_translation_source_snapshot.sql"),
    ),
    (
        "v024_user_items_leaderboard_index",
        include_str!("v024_user_items_leaderboard_index.sql"),
    ),
    (
        "v025_user_story_progress",
        include_str!("v025_user_story_progress.sql"),
    ),
    (
        "v026_user_game_story_read",
        include_str!("v026_user_game_story_read.sql"),
    ),
    (
        "v027_game_story_read_semantics",
        include_str!("v027_game_story_read_semantics.sql"),
    ),
    (
        "v028_game_story_read_verdict",
        include_str!("v028_game_story_read_verdict.sql"),
    ),
    ("v029_tier_entities", include_str!("v029_tier_entities.sql")),
    (
        "v030_planner_pins_presets",
        include_str!("v030_planner_pins_presets.sql"),
    ),
    ("v031_grids", include_str!("v031_grids.sql")),
    (
        "v032_grid_entity_kinds",
        include_str!("v032_grid_entity_kinds.sql"),
    ),
    (
        "v033_profile_layout",
        include_str!("v033_profile_layout.sql"),
    ),
    (
        "v034_leaderboard_private_score",
        include_str!("v034_leaderboard_private_score.sql"),
    ),
    ("v035_user_squads", include_str!("v035_user_squads.sql")),
    ("v036_music_library", include_str!("v036_music_library.sql")),
    ("v037_music_stats", include_str!("v037_music_stats.sql")),
    (
        "v038_leaderboard_hidden_level",
        include_str!("v038_leaderboard_hidden_level.sql"),
    ),
    (
        "v041_leaderboard_unscored_last",
        include_str!("v041_leaderboard_unscored_last.sql"),
    ),
    (
        "v042_public_gacha_history",
        include_str!("v042_public_gacha_history.sql"),
    ),
    (
        "v043_account_lookback",
        include_str!("v043_account_lookback.sql"),
    ),
    (
        "v044_content_audit_log",
        include_str!("v044_content_audit_log.sql"),
    ),
];

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS _migrations (
            name VARCHAR(100) PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    ",
    )
    .execute(pool)
    .await?;

    let applied: std::collections::HashSet<String> =
        sqlx::query_scalar("SELECT name FROM _migrations")
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect();

    for (name, sql) in MIGRATIONS {
        if !applied.contains(*name) {
            let mut tx = pool.begin().await?;
            sqlx::raw_sql(sql).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO _migrations (name) VALUES ($1)")
                .bind(name)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            tracing::info!(migration = name, "applied migration");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::MIGRATIONS;

    /// The body of the last migration that (re)defines `view`: the definition
    /// a fresh or fully migrated database runs.
    fn live_definition(view: &str) -> (&'static str, &'static str) {
        let needle = format!("VIEW {view} AS");
        MIGRATIONS
            .iter()
            .rev()
            .find(|(_, sql)| sql.contains(&needle))
            .copied()
            .unwrap_or_else(|| panic!("no migration defines {view}"))
    }

    /// Every `OVER (...)` window in `sql`, parentheses balanced.
    fn windows(sql: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = sql;
        while let Some(at) = rest.find("OVER (") {
            let body = &rest[at + "OVER (".len()..];
            let mut depth = 1usize;
            let end = body
                .char_indices()
                .find(|&(_, c)| {
                    match c {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    depth == 0
                })
                .map(|(i, _)| i)
                .expect("balanced window");
            out.push(&body[..end]);
            rest = &body[end..];
        }
        out
    }

    /// `v_leaderboard` LEFT JOINs `user_scores`, so an ungraded player ranks on
    /// a NULL score, and a bare `DESC` sorts NULL first: v038 gave every
    /// ungraded player rank 1. This reads whichever migration defines the view
    /// LAST, so a future redefinition that drops `NULLS LAST` fails here, not
    /// on the live board.
    #[test]
    fn the_live_leaderboard_view_ranks_null_scores_last() {
        let (name, sql) = live_definition("v_leaderboard");
        let windows = windows(sql);
        assert_eq!(windows.len(), 2, "{name}: rank_global and rank_server");
        for w in windows {
            assert!(
                w.contains("DESC NULLS LAST"),
                "{name}: window `{w}` sorts NULL first"
            );
        }
    }

    #[test]
    fn v038_alone_would_rank_null_scores_first() {
        // The check above, against the definition v041 replaced: it must catch it.
        let sql = include_str!("v038_leaderboard_hidden_level.sql");
        assert!(windows(sql).iter().all(|w| !w.contains("NULLS LAST")));
    }
}
