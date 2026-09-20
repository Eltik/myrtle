//! Inventory leaderboard queries against the local roster.
//!
//! The queries are built from format strings with hand-counted bind indexes,
//! and a wrong index is a runtime error Postgres reports, not one the
//! compiler does. So this runs every query in both shapes (a `user_status`
//! currency and a `user_items` row) with and without the server filter, and
//! checks the invariants that tie them together: the page's `total` is the
//! count, ranks never decrease down the page, the top row's own standing is
//! rank 1, private profiles never appear, and the catalog's holder count for
//! an item equals what the page reports for it.
//!
//! Needs the local Postgres with real roster data, so it is ignored by default:
//! `cargo test --test item_leaderboard_test -- --ignored --nocapture`

use backend::database::queries::item_leaderboard::{
    count_item_holders, currency_column, get_item_catalog, get_item_leaderboard, get_item_standing,
    is_valid_item_id,
};
use sqlx::PgPool;

#[test]
fn item_id_validation() {
    assert!(is_valid_item_id("4002"));
    assert!(is_valid_item_id("SOCIAL_PT"));
    assert!(is_valid_item_id("mod_unlock_token"));
    assert!(!is_valid_item_id(""));
    assert!(!is_valid_item_id("4002; DROP TABLE users"));
    assert!(!is_valid_item_id("a b"));
    assert!(!is_valid_item_id(&"x".repeat(51)));
}

#[test]
fn currency_map_is_the_documented_ten() {
    for id in [
        "4001",
        "4002",
        "4003",
        "4004",
        "4005",
        "7001",
        "7003",
        "7004",
        "6001",
        "SOCIAL_PT",
    ] {
        assert!(currency_column(id).is_some(), "{id} should be a currency");
    }
    assert!(
        currency_column("4006").is_none(),
        "purchase certificates are a user_items row"
    );
    assert!(currency_column("30011").is_none());
}

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("connect to local postgres")
}

async fn check_item(pool: &PgPool, item: &str, server: Option<&str>) {
    let label = format!("{item} server={}", server.unwrap_or("all"));
    let total = count_item_holders(pool, item, server, None)
        .await
        .expect("count");
    let page = get_item_leaderboard(pool, item, server, None, 50, 0)
        .await
        .expect("page");
    assert_eq!(
        page.len() as i64,
        total.min(50),
        "{label}: page size vs total"
    );
    let mut prev_rank = 0;
    let mut prev_qty = i64::MAX;
    for row in &page {
        assert!(row.rank >= prev_rank, "{label}: ranks must not decrease");
        assert!(
            row.quantity <= prev_qty,
            "{label}: quantities must not increase"
        );
        assert!(row.quantity > 0, "{label}: zero holdings are not holders");
        if let Some(code) = server {
            assert_eq!(row.server, code, "{label}: server filter leaked");
        }
        prev_rank = row.rank;
        prev_qty = row.quantity;
    }
    // A second page continues the same ranking with no overlap.
    if total > 50 {
        let next = get_item_leaderboard(pool, item, server, None, 50, 50)
            .await
            .expect("page 2");
        assert!(!next.is_empty());
        assert!(next[0].rank >= page[page.len() - 1].rank);
        assert!(next[0].quantity <= page[page.len() - 1].quantity);
        assert!(
            !page
                .iter()
                .any(|p| p.uid == next[0].uid && p.server == next[0].server)
        );
    }
    // The top row's own standing agrees with the page.
    if let Some(top) = page.first() {
        let standing = get_item_standing(pool, item, &top.uid, &top.server)
            .await
            .expect("standing")
            .expect("top holder has a standing");
        assert_eq!(
            standing.quantity, top.quantity,
            "{label}: standing quantity"
        );
        assert_eq!(standing.item_id, item);
        if server.is_none() {
            assert_eq!(
                standing.rank_global, 1,
                "{label}: top of the global page is global rank 1"
            );
            assert_eq!(
                standing.holders_global, total,
                "{label}: standing population is the count"
            );
        } else {
            assert_eq!(
                standing.rank_server, 1,
                "{label}: top of a server page is server rank 1"
            );
            assert_eq!(standing.holders_server, total);
        }
        assert!(standing.rank_server <= standing.rank_global);
        assert!(standing.holders_server <= standing.holders_global);
    }
    eprintln!(
        "{label}: holders {total}, top {:?}",
        page.first().map(|r| r.quantity)
    );
}

#[tokio::test]
#[ignore = "reads the local Postgres roster"]
async fn real_data_invariants() {
    let pool = pool().await;
    for item in [
        "4001",
        "4003",
        "4002",
        "SOCIAL_PT",
        "3401",
        "4006",
        "30011",
        "mod_unlock_token",
    ] {
        check_item(&pool, item, None).await;
        check_item(&pool, item, Some("EN")).await;
    }
    // An item nobody holds is an empty page and a zero count, not an error.
    assert_eq!(
        count_item_holders(&pool, "no_such_item", None, None)
            .await
            .unwrap(),
        0
    );
    assert!(
        get_item_leaderboard(&pool, "no_such_item", None, None, 20, 0)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        get_item_standing(&pool, "no_such_item", "00000000", "EN")
            .await
            .unwrap()
            .is_none()
    );

    // Private profiles never appear, in either shape.
    let private: Vec<(String, String)> = sqlx::query_as(
        "SELECT u.uid, s.code FROM users u JOIN servers s ON s.id = u.server_id
         JOIN user_settings us ON us.user_id = u.id WHERE NOT us.public_profile",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    eprintln!("private profiles: {}", private.len());
    for item in ["4001", "3401"] {
        let total = count_item_holders(&pool, item, None, None).await.unwrap();
        let all = get_item_leaderboard(&pool, item, None, None, total.max(1), 0)
            .await
            .unwrap();
        for (uid, server) in &private {
            assert!(
                !all.iter().any(|r| &r.uid == uid && &r.server == server),
                "{item}: private profile {uid}@{server} is ranked"
            );
            assert!(
                get_item_standing(&pool, item, uid, server)
                    .await
                    .unwrap()
                    .is_none()
            );
        }
    }

    // The catalog agrees with the per-item count, currencies included.
    let catalog = get_item_catalog(&pool, None).await.unwrap();
    eprintln!("catalog rows: {}", catalog.len());
    assert!(
        catalog.windows(2).all(|w| w[0].holders >= w[1].holders),
        "catalog sorted by holders desc"
    );
    for row in catalog
        .iter()
        .filter(|r| ["4001", "4002", "4003", "3401", "4006"].contains(&r.item_id.as_str()))
    {
        let total = count_item_holders(&pool, &row.item_id, None, None)
            .await
            .unwrap();
        assert_eq!(
            row.holders, total,
            "{}: catalog holders vs count",
            row.item_id
        );
        let top = get_item_leaderboard(&pool, &row.item_id, None, None, 1, 0)
            .await
            .unwrap();
        assert_eq!(
            row.top, top[0].quantity,
            "{}: catalog top vs page",
            row.item_id
        );
        eprintln!(
            "catalog {}: holders {} top {}",
            row.item_id, row.holders, row.top
        );
    }
    assert!(catalog.iter().all(|r| r.holders > 0));
    let en = get_item_catalog(&pool, Some("EN")).await.unwrap();
    assert!(en.len() <= catalog.len());
}
