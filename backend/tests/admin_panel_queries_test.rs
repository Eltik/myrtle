//! The admin panel's read queries against real rows: the people list, the
//! tier-list grant views, the note and translation audits' actors, the
//! translation grants' names and the translations `todo` filter. Each is
//! checked against an independent SQL or set oracle rather than a fixed
//! number, so it holds as the data changes.
//!
//! Needs the local Postgres with real data, so every test is ignored by
//! default:
//! `cargo test --test admin_panel_queries_test -- --ignored --nocapture`

use std::collections::HashSet;

use backend::core::hypergryph::constants::Server;
use backend::database::queries::admin_users::{AdminUserRole, AdminUserSearch};
use backend::database::queries::{i18n, operator_notes, tier_lists};
use sqlx::PgPool;
use sqlx::types::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("local Postgres with real data")
}

fn role_rank(role: &str) -> u8 {
    match role {
        "super_admin" => 0,
        "tier_list_admin" => 1,
        "tier_list_editor" => 2,
        "translator" => 3,
        _ => 4,
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn people_list_matches_the_users_table() {
    let pool = pool().await;
    let everyone: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();

    let all = AdminUserSearch {
        q: None,
        role: AdminUserRole::All,
        server: None,
    };
    assert_eq!(all.count(&pool).await.unwrap(), everyone);

    // Paging the whole table returns every account exactly once, in role
    // order, private profiles included.
    let mut seen = HashSet::new();
    let mut last_rank = 0;
    let mut any_private = false;
    let mut offset = 0;
    loop {
        let page = all.fetch_page(&pool, 200, offset).await.unwrap();
        if page.is_empty() {
            break;
        }
        for row in &page {
            assert!(seen.insert(row.id), "{} appears twice", row.id);
            let rank = role_rank(&row.role);
            assert!(rank >= last_rank, "role order broken at {}", row.uid);
            last_rank = rank;
            any_private |= !row.public_profile;
            assert!(row.operator_count >= 0 && row.item_count >= 0 && row.skin_count >= 0);
        }
        offset += 200;
    }
    assert_eq!(i64::try_from(seen.len()).unwrap(), everyone);
    let private: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM users u LEFT JOIN user_settings us ON us.user_id = u.id \
         WHERE NOT COALESCE(us.public_profile, false)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(any_private, private > 0);
    println!("{everyone} users, {private} private");

    for (role, sql_roles) in [
        (
            AdminUserRole::Staff,
            "'super_admin','tier_list_admin','tier_list_editor'",
        ),
        (AdminUserRole::Translators, "'translator'"),
        (AdminUserRole::Players, "'user'"),
    ] {
        let search = AdminUserSearch {
            q: None,
            role,
            server: None,
        };
        let expected: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM users WHERE role IN ({sql_roles})"
        ))
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(search.count(&pool).await.unwrap(), expected, "{role:?}");
        let page = search.fetch_page(&pool, 200, 0).await.unwrap();
        assert!(
            page.iter()
                .all(|r| sql_roles.contains(&format!("'{}'", r.role)))
        );
    }

    let en = AdminUserSearch {
        q: None,
        role: AdminUserRole::All,
        server: Some(Server::EN),
    };
    let page = en.fetch_page(&pool, 200, 0).await.unwrap();
    assert!(page.iter().all(|r| r.server.eq_ignore_ascii_case("en")));
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn people_search_finds_a_uid_first_and_matches_nicknames() {
    let pool = pool().await;
    let (uid, nickname): (String, Option<String>) =
        sqlx::query_as("SELECT uid, nickname FROM users ORDER BY uid DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    let by_uid = AdminUserSearch {
        q: Some(&uid),
        role: AdminUserRole::All,
        server: None,
    };
    let page = by_uid.fetch_page(&pool, 50, 0).await.unwrap();
    assert_eq!(page.first().map(|r| r.uid.as_str()), Some(uid.as_str()));

    let prefix = &uid[..uid.len().min(3)];
    let by_prefix = AdminUserSearch {
        q: Some(prefix),
        role: AdminUserRole::All,
        server: None,
    };
    let expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM users WHERE nickname ILIKE '%' || $1 || '%' OR uid LIKE $1 || '%'",
    )
    .bind(prefix)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(by_prefix.count(&pool).await.unwrap(), expected);

    if let Some(nickname) = nickname.filter(|n| !n.is_empty()) {
        let needle = nickname.to_uppercase();
        let by_name = AdminUserSearch {
            q: Some(&needle),
            role: AdminUserRole::All,
            server: None,
        };
        let page = by_name.fetch_page(&pool, 200, 0).await.unwrap();
        assert!(
            page.iter().any(|r| r.uid == uid),
            "case-insensitive nickname"
        );
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn grant_views_agree_with_the_permission_table() {
    let pool = pool().await;
    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM tier_list_permissions p \
         JOIN tier_lists t ON t.id = p.tier_list_id AND t.is_active",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let all = tier_lists::get_all_grants(&pool, None).await.unwrap();
    assert_eq!(i64::try_from(all.len()).unwrap(), rows);
    println!("{rows} grants");

    let users: HashSet<Uuid> = all.iter().map(|g| g.user_id).collect();
    for user in users {
        let mine = tier_lists::get_all_grants(&pool, Some(user)).await.unwrap();
        assert!(mine.iter().all(|g| g.user_id == user));
        assert_eq!(mine.len(), all.iter().filter(|g| g.user_id == user).count());

        let granted = tier_lists::get_granted_lists(&pool, user).await.unwrap();
        let lists: HashSet<Uuid> = mine.iter().map(|g| g.tier_list_id).collect();
        assert_eq!(granted.len(), lists.len(), "one row per list");
        for list in &granted {
            let best = mine
                .iter()
                .filter(|g| g.tier_list_id == list.tier_list_id)
                .map(|g| level(&g.permission))
                .max()
                .unwrap();
            assert_eq!(level(&list.permission), best, "{}", list.slug);
        }
    }
}

fn level(permission: &str) -> u8 {
    match permission {
        "admin" => 3,
        "publish" => 2,
        "edit" => 1,
        _ => 0,
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn note_audit_names_its_actor_and_scopes_by_it() {
    let pool = pool().await;
    let actors: Vec<Uuid> =
        sqlx::query_scalar("SELECT DISTINCT changed_by FROM operator_notes_audit_log")
            .fetch_all(&pool)
            .await
            .unwrap();
    let everyone = operator_notes::count_audit_log(&pool, None).await.unwrap();
    let mut summed = 0;
    for actor in &actors {
        let rows = operator_notes::get_audit_log_global(&pool, 500, None, Some(*actor))
            .await
            .unwrap();
        assert!(rows.iter().all(|r| r.actor_user_id == *actor));
        let n = operator_notes::count_audit_log(&pool, Some(*actor))
            .await
            .unwrap();
        assert_eq!(i64::try_from(rows.len()).unwrap(), n.min(500));
        summed += n;
    }
    assert_eq!(summed, everyone);

    let operator: Option<String> = sqlx::query_scalar(
        "SELECT n.operator_id FROM operator_notes_audit_log a \
         JOIN operator_notes n ON n.id = a.note_id LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    if let Some(operator) = operator {
        let log = operator_notes::get_audit_log(&pool, &operator)
            .await
            .unwrap();
        assert!(!log.is_empty());
        for entry in &log {
            assert_eq!(entry.actor.id, entry.changed_by);
        }
        assert!(log.iter().any(|e| e.actor.nickname.is_some()));
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn todo_is_untranslated_or_stale() {
    let pool = pool().await;
    let locales: Vec<String> = sqlx::query_scalar("SELECT code FROM locales ORDER BY code")
        .fetch_all(&pool)
        .await
        .unwrap();
    let keys = |locale: String, filter: &'static str| {
        let pool = pool.clone();
        async move {
            i18n::list_entries(&pool, &locale, "", "", filter, 100_000, 0)
                .await
                .unwrap()
                .into_iter()
                .map(|e| e.key)
                .collect::<HashSet<_>>()
        }
    };
    for locale in locales {
        let todo = keys(locale.clone(), "todo").await;
        let untranslated = keys(locale.clone(), "untranslated").await;
        let stale = keys(locale.clone(), "stale").await;
        let union: HashSet<_> = untranslated.union(&stale).cloned().collect();
        assert_eq!(todo, union, "{locale}");
        let count = i18n::count_entries(&pool, &locale, "", "", "todo")
            .await
            .unwrap();
        assert_eq!(usize::try_from(count).unwrap(), todo.len(), "{locale}");
        println!(
            "{locale}: todo {} = untranslated {} + stale {}",
            todo.len(),
            untranslated.len(),
            stale.len()
        );
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn translation_entry_audit_names_its_actor() {
    let pool = pool().await;
    let pairs: Vec<(String, String)> = sqlx::query_as(
        "SELECT message_key, locale FROM ui_message_audit_log \
         GROUP BY message_key, locale ORDER BY count(*) DESC LIMIT 20",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let mut named = 0;
    for (key, locale) in &pairs {
        let log = i18n::get_audit_log(&pool, key, locale).await.unwrap();
        assert!(!log.is_empty(), "{locale}/{key}");
        for entry in &log {
            assert_eq!(entry.actor.id, entry.changed_by);
            let oracle: Option<String> =
                sqlx::query_scalar("SELECT nickname FROM users WHERE id = $1")
                    .bind(entry.changed_by)
                    .fetch_optional(&pool)
                    .await
                    .unwrap()
                    .flatten();
            assert_eq!(entry.actor.nickname, oracle, "{locale}/{key}");
            named += usize::from(entry.actor.nickname.is_some());
        }
    }
    println!("{} keys, {named} named rows", pairs.len());
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn translation_grants_carry_both_names() {
    let pool = pool().await;
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM translation_permissions")
        .fetch_one(&pool)
        .await
        .unwrap();
    let all = i18n::list_permissions(&pool, None, None).await.unwrap();
    assert_eq!(i64::try_from(all.len()).unwrap(), rows);
    for grant in &all {
        let (uid, nickname): (Option<String>, Option<String>) =
            sqlx::query_as("SELECT uid, nickname FROM users WHERE id = $1")
                .bind(grant.user_id)
                .fetch_optional(&pool)
                .await
                .unwrap()
                .unwrap_or((None, None));
        assert_eq!(grant.user_uid, uid);
        assert_eq!(grant.user_nickname, nickname);
        let granter: Option<String> = match grant.granted_by {
            Some(id) => sqlx::query_scalar("SELECT nickname FROM users WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .unwrap()
                .flatten(),
            None => None,
        };
        assert_eq!(grant.granted_by_nickname, granter);
    }
    let locales: HashSet<&str> = all.iter().map(|g| g.locale.as_str()).collect();
    for locale in locales {
        let mine = i18n::list_permissions(&pool, Some(locale), None)
            .await
            .unwrap();
        assert!(mine.iter().all(|g| g.locale == locale));
        assert_eq!(
            mine.len(),
            all.iter().filter(|g| g.locale == locale).count()
        );
    }
    // The self-scoped view a non-admin gets: exactly that grantee's rows, and
    // the locale filter still narrows within them.
    let grantees: HashSet<Uuid> = all.iter().map(|g| g.user_id).collect();
    for user in grantees {
        let own = i18n::list_permissions(&pool, None, Some(user))
            .await
            .unwrap();
        assert_eq!(own.len(), all.iter().filter(|g| g.user_id == user).count());
        assert!(own.iter().all(|g| g.user_id == user));
        for grant in &own {
            let one = i18n::list_permissions(&pool, Some(&grant.locale), Some(user))
                .await
                .unwrap();
            assert!(
                one.iter()
                    .all(|g| g.user_id == user && g.locale == grant.locale)
            );
            assert_eq!(
                one.len(),
                own.iter().filter(|g| g.locale == grant.locale).count()
            );
        }
    }
    let nobody = i18n::list_permissions(&pool, None, Some(Uuid::nil()))
        .await
        .unwrap();
    assert!(nobody.is_empty());
    println!(
        "{rows} grants, {} named",
        all.iter().filter(|g| g.user_nickname.is_some()).count()
    );
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn effective_list_level_is_the_highest_held() {
    let pool = pool().await;
    // Every (list, user) pair with a grant, and the oracle: its best level.
    let pairs: Vec<(Uuid, Uuid)> =
        sqlx::query_as("SELECT DISTINCT tier_list_id, user_id FROM tier_list_permissions")
            .fetch_all(&pool)
            .await
            .unwrap();
    let mut multi = 0;
    for (list, user) in &pairs {
        let held: Vec<String> = sqlx::query_scalar(
            "SELECT permission FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2",
        )
        .bind(list)
        .bind(user)
        .fetch_all(&pool)
        .await
        .unwrap();
        multi += usize::from(held.len() > 1);
        let best = held.iter().map(|p| level(p)).max().unwrap();
        let got = tier_lists::get_user_permission(&pool, *list, *user)
            .await
            .unwrap()
            .expect("a held grant");
        assert_eq!(level(&got.permission), best, "{list}/{user}");
    }
    assert!(
        tier_lists::get_user_permission(&pool, Uuid::nil(), Uuid::nil())
            .await
            .unwrap()
            .is_none()
    );
    println!("{} pairs, {multi} with several levels", pairs.len());
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn people_search_reads_wildcards_literally() {
    let pool = pool().await;
    for needle in ["%", "_", "\\"] {
        let search = AdminUserSearch {
            q: Some(needle),
            role: AdminUserRole::All,
            server: None,
        };
        let expected: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM users WHERE strpos(lower(coalesce(nickname, '')), lower($1)) > 0 \
             OR starts_with(uid, $1)",
        )
        .bind(needle)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(search.count(&pool).await.unwrap(), expected, "{needle:?}");
    }
}
