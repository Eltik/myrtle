//! The player search, from the query string to the page.
//!
//! Three layers, because each one can be wrong on its own:
//!
//! 1. `params_are_parsed_off_the_wire` drives `Query<SearchParams>` over real
//!    query strings, the same extractor the route uses, and pins what each
//!    one becomes or which 400 it earns.
//! 2. `real_data_invariants` checks the shapes a single rank or filter has to
//!    hold: the score order is unchanged, a metric never moves against its
//!    direction, a filtered row really satisfies its filter.
//! 3. `every_combination_matches_an_independent_oracle` is the exhaustive
//!    one. The page is assembled by `QueryBuilder` from a metric CTE, an
//!    owned CTE, three filters and a direction, and those compose in 288
//!    ways that no shape check covers: a wrong join or a mis-set alias can
//!    hold for each part alone and still lose rows together. So it rebuilds
//!    every user's metrics and roster facts in Rust from raw rows, then
//!    compares the whole page and its total against that oracle for all 288
//!    combinations of 9 ranks, 2 directions, 8 filter subsets and a nickname
//!    query, paging included.
//!
//! Needs the local Postgres with real roster data, so the two that read it
//! are ignored by default:
//! `cargo test --test user_search_test -- --ignored --nocapture`

use std::collections::{HashMap, HashSet};

use axum::extract::Query;
use backend::app::routes::search::SearchParams;
use backend::app::services::search::{Scope, SearchRequest, SearchSort, is_valid_operator_id};
use backend::core::gamedata::types::operator::OperatorProfession;
use backend::database::queries::user_search::{OwnsAll, Rank, UserSearch};
use sqlx::PgPool;
use sqlx::types::Uuid;

#[test]
fn sort_tokens_parse() {
    assert_eq!(SearchSort::parse("score"), Some(SearchSort::Score));
    assert_eq!(SearchSort::parse("masteries"), Some(SearchSort::Masteries));
    assert_eq!(
        SearchSort::parse("class:WARRIOR"),
        Some(SearchSort::Owned(Scope::Class(OperatorProfession::Guard)))
    );
    assert_eq!(
        SearchSort::parse("sub:centurion"),
        Some(SearchSort::Owned(Scope::Archetype("centurion".into())))
    );
    assert_eq!(SearchSort::parse("class:TOKEN"), None);
    assert_eq!(SearchSort::parse("class:warrior"), None);
    assert_eq!(SearchSort::parse("sub:Centurion"), None);
    assert_eq!(SearchSort::parse("sub:"), None);
    assert_eq!(SearchSort::parse("total_score"), None);
    assert_eq!(SearchSort::parse("sub:x; DROP TABLE users"), None);
}

#[test]
fn join_date_defaults_to_oldest_first() {
    assert!(!SearchSort::Joined.default_descending());
    assert!(SearchSort::Score.default_descending());
    assert!(SearchSort::Operators.default_descending());
}

#[test]
fn operator_id_validation() {
    assert!(is_valid_operator_id("char_1035_wisdel"));
    assert!(is_valid_operator_id("char_002_amiya"));
    assert!(!is_valid_operator_id(""));
    assert!(!is_valid_operator_id("char_002_amiya,char_003"));
    assert!(!is_valid_operator_id("a b"));
    assert!(!is_valid_operator_id(&"x".repeat(51)));
}

/// The route's own parsing, driven through the extractor the route uses.
fn parse(query: &str) -> Result<SearchRequest, String> {
    let uri: axum::http::Uri = format!("http://test/search?{query}")
        .parse()
        .map_err(|_| "unparseable uri".to_owned())?;
    let params = Query::<SearchParams>::try_from_uri(&uri)
        .map_err(|e| e.to_string())?
        .0;
    params.into_request().map_err(|e| format!("{e:?}"))
}

fn request(sort: SearchSort, descending: Option<bool>) -> SearchRequest {
    SearchRequest {
        q: None,
        sort,
        descending,
        has: Vec::new(),
        support: None,
        all: None,
        limit: 20,
        offset: 0,
    }
}

#[test]
fn params_are_parsed_off_the_wire() {
    // The bare call the page makes when nothing is chosen.
    assert_eq!(parse("").expect("bare"), request(SearchSort::Score, None));

    // Every sort token, as the toolbar writes them (a colon survives
    // percent-encoding either way).
    assert_eq!(
        parse("sort=masteries").expect("masteries"),
        request(SearchSort::Masteries, None)
    );
    assert_eq!(
        parse("sort=class%3AWARRIOR").expect("encoded class"),
        request(
            SearchSort::Owned(Scope::Class(OperatorProfession::Guard)),
            None
        )
    );
    assert_eq!(
        parse("sort=sub:centurion").expect("archetype"),
        request(
            SearchSort::Owned(Scope::Archetype("centurion".into())),
            None
        )
    );

    // Direction, including the default left absent.
    assert_eq!(
        parse("sort=joined&dir=desc").expect("dir"),
        request(SearchSort::Joined, Some(true))
    );
    assert_eq!(
        parse("sort=joined&dir=asc").expect("dir"),
        request(SearchSort::Joined, Some(false))
    );

    // Filters. `has` is split, trimmed, deduplicated and sorted, so the
    // count test in SQL is exact and two orderings share a cache key.
    let parsed = parse("has=char_002_amiya,%20char_1035_wisdel%20,char_002_amiya&support=char_102_texas&all=class%3ATANK")
        .expect("filters");
    assert_eq!(
        parsed.has,
        vec!["char_002_amiya".to_owned(), "char_1035_wisdel".to_owned()]
    );
    assert_eq!(parsed.support.as_deref(), Some("char_102_texas"));
    assert_eq!(parsed.all, Some(Scope::Class(OperatorProfession::Defender)));

    // An empty segment in `has` is dropped, not read as a malformed id: a
    // trailing comma is what a list the visitor is still editing looks like.
    assert_eq!(
        parse("has=char_002_amiya,").expect("trailing comma").has,
        vec!["char_002_amiya".to_owned()]
    );

    // Blank values are absences, not filters that match nothing.
    let blank = parse("q=%20&sort=&dir=&has=&support=&all=").expect("blank");
    assert_eq!(blank, request(SearchSort::Score, None));

    // Pagination still comes off the shared extractor, cap included.
    let paged = parse("limit=500&offset=48").expect("paged");
    assert_eq!((paged.limit, paged.offset), (100, 48));

    // `has` is capped: twenty distinct ids parse, twenty-one are a 400, and
    // duplicates do not count toward the cap.
    let ids: Vec<String> = (0..21).map(|i| format!("char_{i:03}_x")).collect();
    let twenty = format!("has={}", ids[..20].join(","));
    assert_eq!(parse(&twenty).expect("twenty").has.len(), 20);
    let twenty_twice = format!("has={},{}", ids[..20].join(","), ids[..20].join(","));
    assert_eq!(parse(&twenty_twice).expect("duplicates").has.len(), 20);
    assert!(
        parse(&format!("has={}", ids.join(","))).is_err(),
        "21 operators must be rejected"
    );

    // And what earns a 400.
    for bad in [
        "sort=total_score",
        "sort=class:TOKEN",
        "sort=sub:Centurion",
        "dir=up",
        "has=char_002_amiya;DROP%20TABLE%20users",
        "support=a%20b",
        "support=%22",
        "all=warrior",
        "all=sub:",
    ] {
        assert!(parse(bad).is_err(), "`{bad}` should be rejected");
    }
}

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("local Postgres with roster data")
}

/// The aggregate the page claims for one user, computed independently.
async fn independent_metric(pool: &PgPool, rank: &Rank<'_>, uid: &str) -> Option<i64> {
    let sql = match rank {
        Rank::Score => return None,
        Rank::Operators => {
            "SELECT count(*)::bigint FROM user_operators uo JOIN users u ON u.id = uo.user_id WHERE u.uid = $1"
        }
        Rank::Joined => {
            "SELECT st.register_ts FROM user_status st JOIN users u ON u.id = st.user_id WHERE u.uid = $1"
        }
        Rank::Enemies => {
            "SELECT COALESCE((SELECT count(*) FROM user_enemy_progress e, jsonb_object_keys(e.enemies) WHERE e.user_id = u.id), 0)::bigint FROM users u WHERE u.uid = $1"
        }
        Rank::Potentials => {
            "SELECT COALESCE(sum(uo.potential), 0)::bigint FROM user_operators uo JOIN users u ON u.id = uo.user_id WHERE u.uid = $1"
        }
        Rank::Masteries => {
            "SELECT count(*)::bigint FROM user_operator_skills s JOIN users u ON u.id = s.user_id WHERE u.uid = $1 AND s.specialize_level = 3"
        }
        Rank::Modules => {
            "SELECT count(*)::bigint FROM user_operator_modules m JOIN users u ON u.id = m.user_id WHERE u.uid = $1 AND m.module_level = 3 AND NOT m.locked"
        }
        Rank::Skins => {
            "SELECT count(*)::bigint FROM user_skins s JOIN users u ON u.id = s.user_id WHERE u.uid = $1"
        }
        Rank::OwnedOf(ids) => {
            return sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM user_operators uo JOIN users u ON u.id = uo.user_id WHERE u.uid = $1 AND uo.operator_id = ANY($2)",
            )
            .bind(uid)
            .bind(ids.to_vec())
            .fetch_one(pool)
            .await
            .ok();
        }
    };
    sqlx::query_scalar::<_, Option<i64>>(sql)
        .bind(uid)
        .fetch_one(pool)
        .await
        .expect("independent metric")
}

async fn check_rank(pool: &PgPool, rank: Rank<'_>, descending: bool) {
    let search = UserSearch {
        q: None,
        rank,
        descending,
        has: &[],
        support: None,
        owns_all: None,
    };
    let page = search.fetch_page(pool, 24, 0).await.expect("page");
    let total = search.count(pool).await.expect("count");
    assert!(total >= page.len() as i64, "{rank:?}: total {total} < page");
    assert!(!page.is_empty(), "{rank:?}: empty page");

    match rank {
        Rank::Score => {
            for row in &page {
                assert!(row.metric.is_none(), "score rows carry no metric");
            }
        }
        _ => {
            let metrics: Vec<Option<i64>> = page.iter().map(|r| r.metric).collect();
            for pair in metrics.windows(2) {
                match (pair[0], pair[1]) {
                    (Some(a), Some(b)) => {
                        if descending {
                            assert!(a >= b, "{rank:?} desc moved up: {a} -> {b}");
                        } else {
                            assert!(a <= b, "{rank:?} asc moved down: {a} -> {b}");
                        }
                    }
                    (None, Some(_)) => panic!("{rank:?}: a NULL metric sorted before a value"),
                    _ => {}
                }
            }
            let top = &page[0];
            let expected = independent_metric(pool, &rank, &top.profile.uid).await;
            assert_eq!(
                top.metric, expected,
                "{rank:?} top row {} metric",
                top.profile.uid
            );
        }
    }
    let dir = if descending { "desc" } else { "asc" };
    println!(
        "{rank:?} {dir}: total {total}, top {} metric {:?}",
        page[0].profile.uid, page[0].metric
    );
}

#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn real_data_invariants() {
    let pool = pool().await;

    // The score order with no filters is what `search_by_nickname` produced
    // before the builder existed: the same rows, in the same order, with the
    // uid tiebreak only ever splitting equal scores.
    let baseline: Vec<(String, Option<f64>)> = sqlx::query_as(
        "SELECT uid, total_score FROM v_user_profile WHERE public_profile = true \
         ORDER BY total_score DESC NULLS LAST, uid LIMIT 24",
    )
    .fetch_all(&pool)
    .await
    .expect("baseline");
    let baseline_total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM v_user_profile WHERE public_profile = true")
            .fetch_one(&pool)
            .await
            .expect("baseline count");
    let plain = UserSearch {
        q: None,
        rank: Rank::Score,
        descending: true,
        has: &[],
        support: None,
        owns_all: None,
    };
    let page = plain.fetch_page(&pool, 24, 0).await.expect("plain page");
    let got: Vec<(String, Option<f64>)> = page
        .iter()
        .map(|r| (r.profile.uid.clone(), r.profile.total_score))
        .collect();
    assert_eq!(got, baseline, "score order changed");
    assert_eq!(
        plain.count(&pool).await.expect("plain count"),
        baseline_total
    );
    println!("score floor: {baseline_total} public players, page 1 identical");

    let guards: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT operator_id FROM user_operators WHERE operator_id LIKE 'char_1%' ORDER BY 1 LIMIT 40",
    )
    .fetch_all(&pool)
    .await
    .expect("some operator ids");
    for rank in [
        Rank::Score,
        Rank::Operators,
        Rank::Joined,
        Rank::Enemies,
        Rank::Potentials,
        Rank::Masteries,
        Rank::Modules,
        Rank::Skins,
        Rank::OwnedOf(&guards),
    ] {
        check_rank(&pool, rank, true).await;
        check_rank(&pool, rank, false).await;
    }

    // Filters: every admitted row satisfies the fact, and the total is the
    // independent count of users who do.
    let has = vec!["char_002_amiya".to_owned(), "char_1001_amiya2".to_owned()];
    let has_search = UserSearch {
        q: None,
        rank: Rank::Masteries,
        descending: true,
        has: &has,
        support: None,
        owns_all: None,
    };
    let has_total = has_search.count(&pool).await.expect("has count");
    let has_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true \
         AND EXISTS (SELECT 1 FROM user_operators uo WHERE uo.user_id = p.id AND uo.operator_id = 'char_002_amiya') \
         AND EXISTS (SELECT 1 FROM user_operators uo WHERE uo.user_id = p.id AND uo.operator_id = 'char_1001_amiya2')",
    )
    .fetch_one(&pool)
    .await
    .expect("has expected");
    assert_eq!(has_total, has_expected, "has= total");
    let has_page = has_search.fetch_page(&pool, 24, 0).await.expect("has page");
    for row in &has_page {
        let owned: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM user_operators WHERE user_id = $1 AND operator_id = ANY($2)",
        )
        .bind(row.profile.id)
        .bind(&has)
        .fetch_one(&pool)
        .await
        .expect("owned");
        assert_eq!(owned, 2, "{} admitted without both", row.profile.uid);
    }
    println!("has=amiya,amiya2: {has_total} players (baseline {baseline_total})");

    let support_id: String = sqlx::query_scalar(
        "SELECT operator_id FROM user_support_units GROUP BY 1 ORDER BY count(*) DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("a support operator");
    let support_search = UserSearch {
        q: None,
        rank: Rank::Score,
        descending: true,
        has: &[],
        support: Some(&support_id),
        owns_all: None,
    };
    let support_total = support_search.count(&pool).await.expect("support count");
    let support_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true \
         AND EXISTS (SELECT 1 FROM user_support_units su WHERE su.user_id = p.id AND su.operator_id = $1)",
    )
    .bind(&support_id)
    .fetch_one(&pool)
    .await
    .expect("support expected");
    assert_eq!(support_total, support_expected, "support= total");
    println!("support={support_id}: {support_total} players");

    // Owns-all over a tiny scope: the starters everyone has, so the per-server
    // required count is met by nearly everyone, and one id nobody can own,
    // which nobody meets. Servers absent from `required` drop out.
    let servers: Vec<String> = sqlx::query_scalar("SELECT lower(code) FROM servers ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("servers");
    let starters = vec!["char_002_amiya".to_owned(), "char_500_noirc".to_owned()];
    let required: Vec<(String, i64)> = servers.iter().map(|s| (s.clone(), 2)).collect();
    let all_search = UserSearch {
        q: None,
        rank: Rank::Score,
        descending: true,
        has: &[],
        support: None,
        owns_all: Some(OwnsAll {
            ids: &starters,
            required: &required,
        }),
    };
    let all_total = all_search.count(&pool).await.expect("all count");
    let all_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true \
         AND (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id AND uo.operator_id = ANY($1)) = 2",
    )
    .bind(&starters)
    .fetch_one(&pool)
    .await
    .expect("all expected");
    assert_eq!(all_total, all_expected, "all= total");
    let page = all_search.fetch_page(&pool, 5, 0).await.expect("all page");
    assert_eq!(page.len(), all_total.min(5) as usize);
    println!("owns all of [amiya, noir corne]: {all_total} players");

    let impossible = vec!["char_000_nobody".to_owned()];
    let required_one: Vec<(String, i64)> = servers.iter().map(|s| (s.clone(), 1)).collect();
    let none = UserSearch {
        q: None,
        rank: Rank::Score,
        descending: true,
        has: &[],
        support: None,
        owns_all: Some(OwnsAll {
            ids: &impossible,
            required: &required_one,
        }),
    };
    assert_eq!(none.count(&pool).await.expect("none count"), 0);
    assert!(
        none.fetch_page(&pool, 5, 0)
            .await
            .expect("none page")
            .is_empty()
    );

    // A server missing from `required` cannot be judged: its users drop out.
    let en_only = vec![("en".to_owned(), 2)];
    let en_search = UserSearch {
        owns_all: Some(OwnsAll {
            ids: &starters,
            required: &en_only,
        }),
        ..all_search.clone()
    };
    let en_total = en_search.count(&pool).await.expect("en count");
    let en_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true AND lower(p.server) = 'en' \
         AND (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id AND uo.operator_id = ANY($1)) = 2",
    )
    .bind(&starters)
    .fetch_one(&pool)
    .await
    .expect("en expected");
    assert_eq!(en_total, en_expected, "unjudged servers must drop out");
    println!("owns all, EN judged only: {en_total} players");

    // A scope that resolves to no operators matches nobody, rather than
    // admitting everyone on a vacuous `0 >= 0`.
    let empty: Vec<String> = Vec::new();
    let required_zero: Vec<(String, i64)> = servers.iter().map(|s| (s.clone(), 0)).collect();
    let empty_scope = UserSearch {
        q: None,
        rank: Rank::Score,
        descending: true,
        has: &[],
        support: None,
        owns_all: Some(OwnsAll {
            ids: &empty,
            required: &required_zero,
        }),
    };
    assert_eq!(
        empty_scope.count(&pool).await.expect("empty scope count"),
        0
    );
    assert!(
        empty_scope
            .fetch_page(&pool, 5, 0)
            .await
            .expect("empty scope page")
            .is_empty()
    );
    println!("owns all of an empty scope: 0 players");

    // The nickname filter composes with a metric sort and every roster filter.
    let combo = UserSearch {
        q: Some("a"),
        rank: Rank::Modules,
        descending: true,
        has: &has,
        support: Some(&support_id),
        owns_all: Some(OwnsAll {
            ids: &starters,
            required: &required,
        }),
    };
    let combo_total = combo.count(&pool).await.expect("combo count");
    let combo_page = combo.fetch_page(&pool, 24, 0).await.expect("combo page");
    assert!(combo_total >= combo_page.len() as i64);
    println!("q=a + modules + has + support + all: {combo_total} players");
}

/// One public player as the oracle knows them, rebuilt in Rust from raw rows
/// so nothing here shares code with the query under test.
struct OracleUser {
    id: Uuid,
    uid: String,
    nickname: Option<String>,
    server: String,
    total_score: Option<f64>,
    register_ts: Option<i64>,
    owned: HashMap<String, i64>,
    supports: HashSet<String>,
    enemies: i64,
    masteries: i64,
    modules: i64,
    skins: i64,
}

impl OracleUser {
    /// The value the page should carry for a rank, by the same rule the SQL
    /// uses: an absent aggregate reads 0, and `joined` passes `register_ts`
    /// through, `None` and all.
    fn metric(&self, rank: &Rank<'_>) -> Option<i64> {
        match rank {
            Rank::Score => None,
            Rank::Operators => Some(self.owned.len() as i64),
            Rank::Joined => self.register_ts,
            Rank::Enemies => Some(self.enemies),
            Rank::Potentials => Some(self.owned.values().sum()),
            Rank::Masteries => Some(self.masteries),
            Rank::Modules => Some(self.modules),
            Rank::Skins => Some(self.skins),
            Rank::OwnedOf(ids) => {
                Some(ids.iter().filter(|id| self.owned.contains_key(*id)).count() as i64)
            }
        }
    }
}

/// One identity row: id, uid, nickname, server, score, registration.
type IdentityRow = (
    Uuid,
    String,
    Option<String>,
    String,
    Option<f64>,
    Option<i64>,
);

/// Everything the oracle needs, in five queries that touch the raw tables
/// rather than `v_user_profile`'s own aggregates.
async fn load_oracle(pool: &PgPool) -> Vec<OracleUser> {
    let rows: Vec<IdentityRow> =
        sqlx::query_as(
            "SELECT u.id, u.uid, u.nickname, lower(s.code), sc.total_score, st.register_ts \
             FROM users u \
             JOIN servers s ON s.id = u.server_id \
             LEFT JOIN user_scores sc ON sc.user_id = u.id \
             LEFT JOIN user_status st ON st.user_id = u.id \
             WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)",
        )
        .fetch_all(pool)
        .await
        .expect("public users");

    let mut users: HashMap<Uuid, OracleUser> = rows
        .into_iter()
        .map(|(id, uid, nickname, server, total_score, register_ts)| {
            (
                id,
                OracleUser {
                    id,
                    uid,
                    nickname,
                    server,
                    total_score,
                    register_ts,
                    owned: HashMap::new(),
                    supports: HashSet::new(),
                    enemies: 0,
                    masteries: 0,
                    modules: 0,
                    skins: 0,
                },
            )
        })
        .collect();

    let owned: Vec<(Uuid, String, i16)> =
        sqlx::query_as("SELECT user_id, operator_id, potential FROM user_operators")
            .fetch_all(pool)
            .await
            .expect("rosters");
    for (user_id, operator_id, potential) in owned {
        if let Some(user) = users.get_mut(&user_id) {
            user.owned.insert(operator_id, i64::from(potential));
        }
    }

    let supports: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT user_id, operator_id FROM user_support_units")
            .fetch_all(pool)
            .await
            .expect("support units");
    for (user_id, operator_id) in supports {
        if let Some(user) = users.get_mut(&user_id) {
            user.supports.insert(operator_id);
        }
    }

    for (sql, field) in [
        (
            "SELECT user_id, count(*)::bigint FROM user_operator_skills WHERE specialize_level = 3 GROUP BY user_id",
            "masteries",
        ),
        (
            "SELECT user_id, count(*)::bigint FROM user_operator_modules WHERE module_level = 3 AND NOT locked GROUP BY user_id",
            "modules",
        ),
        (
            "SELECT user_id, count(*)::bigint FROM user_skins GROUP BY user_id",
            "skins",
        ),
        (
            "SELECT user_id, (SELECT count(*)::bigint FROM jsonb_object_keys(enemies)) FROM user_enemy_progress",
            "enemies",
        ),
    ] {
        let counts: Vec<(Uuid, i64)> = sqlx::query_as(sql).fetch_all(pool).await.expect(field);
        for (user_id, value) in counts {
            let Some(user) = users.get_mut(&user_id) else {
                continue;
            };
            match field {
                "masteries" => user.masteries = value,
                "modules" => user.modules = value,
                "skins" => user.skins = value,
                _ => user.enemies = value,
            }
        }
    }

    users.into_values().collect()
}

/// The filters of one combination, as the oracle applies them.
struct OracleFilters<'a> {
    q: Option<&'a str>,
    has: &'a [String],
    support: Option<&'a str>,
    owns_all: Option<&'a OwnsAll<'a>>,
}

impl OracleFilters<'_> {
    fn admits(&self, user: &OracleUser) -> bool {
        if let Some(q) = self.q {
            // `nickname ILIKE '%q%'`: a NULL nickname is never a match.
            let Some(nickname) = &user.nickname else {
                return false;
            };
            if !nickname.to_lowercase().contains(&q.to_lowercase()) {
                return false;
            }
        }
        if !self.has.iter().all(|id| user.owned.contains_key(id)) {
            return false;
        }
        if let Some(support) = self.support
            && !user.supports.contains(support)
        {
            return false;
        }
        if let Some(all) = self.owns_all {
            let Some((_, required)) = all
                .required
                .iter()
                .find(|(server, _)| *server == user.server)
            else {
                // A server with no loaded game data cannot be judged.
                return false;
            };
            let owned = all
                .ids
                .iter()
                .filter(|id| user.owned.contains_key(*id))
                .count() as i64;
            // The inner join on `owned` also drops anyone owning none of it.
            if owned == 0 || owned < *required {
                return false;
            }
        }
        true
    }
}

/// `ORDER BY metric {dir} NULLS LAST, total_score DESC NULLS LAST, uid`, or
/// the score order when the rank is `Score`.
fn oracle_order(users: &mut [&OracleUser], rank: &Rank<'_>, descending: bool) {
    users.sort_by(|a, b| {
        let primary = if matches!(rank, Rank::Score) {
            nulls_last(a.total_score, b.total_score, descending)
        } else {
            nulls_last(a.metric(rank), b.metric(rank), descending)
        };
        primary
            .then_with(|| nulls_last(a.total_score, b.total_score, true))
            .then_with(|| a.uid.cmp(&b.uid))
    });
}

/// One `ORDER BY` term: the requested direction, with NULLs last either way.
fn nulls_last<T: PartialOrd>(a: Option<T>, b: Option<T>, descending: bool) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a), Some(b)) => {
            let ordering = a.partial_cmp(&b).expect("comparable");
            if descending {
                ordering.reverse()
            } else {
                ordering
            }
        }
    }
}

#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn every_combination_matches_an_independent_oracle() {
    let pool = pool().await;
    let oracle = load_oracle(&pool).await;
    assert!(!oracle.is_empty(), "no public players to check against");

    // The scope both the scoped rank and the owns-all filter use: forty real
    // operator ids, with a threshold that actually splits the population so
    // the filter is doing work rather than admitting everyone.
    let scope: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT operator_id FROM user_operators WHERE operator_id LIKE 'char_1%' ORDER BY 1 LIMIT 40",
    )
    .fetch_all(&pool)
    .await
    .expect("scope ids");
    let servers: Vec<String> = sqlx::query_scalar("SELECT lower(code) FROM servers ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("servers");
    let required: Vec<(String, i64)> = servers.iter().map(|s| (s.clone(), 5)).collect();
    let owns_all = OwnsAll {
        ids: &scope,
        required: &required,
    };

    // Two operators most rosters have, and the most fielded support unit, so
    // the filtered sets are large enough to page through.
    let has: Vec<String> = vec!["char_002_amiya".to_owned(), "char_1001_amiya2".to_owned()];
    let support: String = sqlx::query_scalar(
        "SELECT operator_id FROM user_support_units GROUP BY 1 ORDER BY count(*) DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("a support operator");

    let ranks: [(&str, Rank<'_>); 9] = [
        ("score", Rank::Score),
        ("operators", Rank::Operators),
        ("joined", Rank::Joined),
        ("enemies", Rank::Enemies),
        ("potentials", Rank::Potentials),
        ("masteries", Rank::Masteries),
        ("modules", Rank::Modules),
        ("skins", Rank::Skins),
        ("scoped", Rank::OwnedOf(&scope)),
    ];

    let mut checked = 0_usize;
    let mut zero_row_cases = 0_usize;
    for (rank_name, rank) in &ranks {
        for descending in [true, false] {
            for subset in 0..8_u8 {
                let use_has = subset & 1 != 0;
                let use_support = subset & 2 != 0;
                let use_all = subset & 4 != 0;
                for q in [None, Some("a")] {
                    let search = UserSearch {
                        q,
                        rank: *rank,
                        descending,
                        has: if use_has { &has } else { &[] },
                        support: if use_support {
                            Some(support.as_str())
                        } else {
                            None
                        },
                        owns_all: if use_all {
                            Some(owns_all.clone())
                        } else {
                            None
                        },
                    };
                    let filters = OracleFilters {
                        q,
                        has: if use_has { &has } else { &[] },
                        support: if use_support {
                            Some(support.as_str())
                        } else {
                            None
                        },
                        owns_all: if use_all { Some(&owns_all) } else { None },
                    };
                    let label = format!(
                        "{rank_name}/{}/has={use_has}/support={use_support}/all={use_all}/q={}",
                        if descending { "desc" } else { "asc" },
                        q.unwrap_or("-")
                    );

                    let mut expected: Vec<&OracleUser> =
                        oracle.iter().filter(|u| filters.admits(u)).collect();
                    oracle_order(&mut expected, rank, descending);

                    let total = search.count(&pool).await.expect("count");
                    assert_eq!(total, expected.len() as i64, "{label}: total");
                    if expected.is_empty() {
                        zero_row_cases += 1;
                    }

                    // Page 1 and page 2, so the offset is checked against the
                    // same ordering rather than assumed to follow from it.
                    for offset in [0_usize, 24] {
                        let page = search
                            .fetch_page(&pool, 24, offset as i64)
                            .await
                            .expect("page");
                        let want: Vec<&&OracleUser> =
                            expected.iter().skip(offset).take(24).collect();
                        assert_eq!(page.len(), want.len(), "{label}: page {offset} length");
                        for (row, expect) in page.iter().zip(want.iter()) {
                            // Ties are real: several players share a metric and
                            // a score, so a row is checked on its value and its
                            // identity only where the order is determined.
                            assert_eq!(
                                row.metric,
                                expect.metric(rank),
                                "{label}: metric at offset {offset} for {}",
                                row.profile.uid
                            );
                            assert_eq!(
                                row.profile.id, expect.id,
                                "{label}: row identity at offset {offset}"
                            );
                        }
                    }
                    checked += 1;
                }
            }
        }
    }

    println!(
        "{checked} combinations checked against the oracle over {} public players ({zero_row_cases} of them empty)",
        oracle.len()
    );
    assert_eq!(checked, 288, "the sweep must cover every combination");
}
