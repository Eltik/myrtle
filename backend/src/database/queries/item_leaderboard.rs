//! Inventory leaderboard: every visible holder of one item, ranked by how much
//! of it they hold.
//!
//! The inventory lives in two shapes. Currencies (LMD, Originite Prime,
//! Orundum, certificates, permits) are columns on `user_status`, one row per
//! user; every other item is a `(user_id, item_id, quantity)` row in
//! `user_items`. Both are addressed here by the game's own item id, so a
//! caller never has to know which table a thing lives in: `CURRENCY_COLUMNS`
//! maps the handful of currency ids to their column and everything else is a
//! `user_items` lookup.
//!
//! Visibility is the same gate as `v_leaderboard`: `user_settings.public_profile`.

use std::fmt::Write;

use sqlx::PgPool;

use crate::database::models::item_leaderboard::{
    ItemHoldingSummary, ItemLeaderboardEntry, ItemStanding,
};

/// Game item id -> `user_status` column. `orundum_shard` is deliberately
/// absent: `extract_status` writes it as a constant 0, so it ranks nothing.
/// Expedited Plans (7002) are also a `status` field in the game but are
/// written as a `user_items` row by `extract_items`, so they need no entry.
/// `originite` is NULL for users who last refreshed before v019 landed
/// (2026-09-15); NULL fails `> 0`, so they are not holders yet.
const CURRENCY_COLUMNS: &[(&str, &str)] = &[
    ("4001", "lmd"),
    ("4002", "originite"),
    ("4003", "orundum"),
    ("4004", "hgg_shard"),
    ("4005", "lgg_shard"),
    ("7001", "recruit_permits"),
    ("7003", "gacha_tickets"),
    ("7004", "ten_pull_tickets"),
    ("6001", "practice_tickets"),
    ("SOCIAL_PT", "social_point"),
    ("classic_gacha", "classic_gacha_tickets"),
    ("classic_gacha_10", "classic_ten_pull_tickets"),
];

/// `('4001', st.lmd::bigint), ('4002', st.originite::bigint), ...`: one VALUES
/// row per currency, for a `CROSS JOIN LATERAL` over `user_status st`.
/// Generated from `CURRENCY_COLUMNS` so no query can drift from the map.
pub fn currency_values_sql() -> String {
    CURRENCY_COLUMNS
        .iter()
        .map(|(id, col)| format!("('{id}', st.{col}::bigint)"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn currency_column(item_id: &str) -> Option<&'static str> {
    CURRENCY_COLUMNS
        .iter()
        .find(|(id, _)| *id == item_id)
        .map(|(_, col)| *col)
}

/// Item ids are `varchar(50)` of `[A-Za-z0-9_]`; anything else cannot match a
/// row and is refused before it reaches a bind.
pub fn is_valid_item_id(item_id: &str) -> bool {
    !item_id.is_empty()
        && item_id.len() <= 50
        && item_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// The `holding` CTE: `(user_id, quantity)` for everyone holding the item,
/// and how the query's `$n` parameters start. Currencies bake their column
/// name in (whitelisted above, never user text) and bind nothing; items bind
/// the id as `$1`.
struct Holding {
    cte: String,
    binds_item: bool,
}

impl Holding {
    fn of(item_id: &str) -> Self {
        match currency_column(item_id) {
            Some(col) => Self {
                cte: format!(
                    "SELECT st.user_id, st.{col}::bigint AS quantity FROM user_status st WHERE st.{col} > 0"
                ),
                binds_item: false,
            },
            None => Self {
                cte: "SELECT ui.user_id, ui.quantity::bigint AS quantity FROM user_items ui WHERE ui.item_id = $1 AND ui.quantity > 0".to_owned(),
                binds_item: true,
            },
        }
    }

    /// Index of the first `$n` a caller may use after the CTE's own binds.
    const fn next_param(&self) -> usize {
        if self.binds_item { 2 } else { 1 }
    }

    /// The bind values the CTE itself consumes, in `$n` order.
    fn params<'a>(&self, item_id: &'a str) -> Vec<&'a str> {
        if self.binds_item {
            vec![item_id]
        } else {
            Vec::new()
        }
    }
}

/// `holding` joined to its players: aliases `h`, `u`, `s`. Always followed
/// by `WHERE {VISIBLE}`, possibly after further joins.
const HOLDERS_JOIN: &str = r"FROM holding h
            JOIN users u ON u.id = h.user_id
            JOIN servers s ON s.id = u.server_id";

/// The same gate as `v_leaderboard`: only public profiles are ranked.
/// Expects the `users` alias `u`.
const VISIBLE: &str =
    "EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)";

/// Optional server and nickname/uid filters, as `AND` clauses against the
/// given `(server, nickname, uid)` column names, binding from `$first`.
/// `params` are the values to bind, in `$n` order; `next_param` is the first
/// index left free for the caller.
struct Filters {
    sql: String,
    params: Vec<String>,
    next_param: usize,
}

impl Filters {
    fn new(first: usize, server: Option<&str>, q: Option<&str>, cols: (&str, &str, &str)) -> Self {
        let (server_col, nick_col, uid_col) = cols;
        let mut sql = String::new();
        let mut params = Vec::new();
        // `write!` into a String cannot fail, so the results are discarded.
        if let Some(code) = server {
            let _ = write!(sql, " AND {server_col} = ${}", first + params.len());
            params.push(code.to_owned());
        }
        if let Some(q) = q {
            let idx = first + params.len();
            let _ = write!(
                sql,
                " AND ({nick_col} ILIKE ${idx} OR {uid_col} ILIKE ${idx})"
            );
            params.push(format!("%{q}%"));
        }
        let next_param = first + params.len();
        Self {
            sql,
            params,
            next_param,
        }
    }
}

/// One page of holders of `item_id`, highest quantity first. Ties share a
/// rank and are ordered by uid so paging is stable. `q` filters by nickname
/// or uid the same way the score leaderboard does; the rank is still taken
/// over every visible holder, so a filtered row keeps its true rank.
pub async fn get_item_leaderboard(
    pool: &PgPool,
    item_id: &str,
    server: Option<&str>,
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ItemLeaderboardEntry>, sqlx::Error> {
    let holding = Holding::of(item_id);
    // The filters apply to the ranked rows, so a search does not renumber
    // the rows it finds.
    let filters = Filters::new(
        holding.next_param(),
        server,
        q,
        ("server", "nickname", "uid"),
    );
    let limit_idx = filters.next_param;
    let offset_idx = limit_idx + 1;
    let sql = format!(
        r"
        WITH holding AS ({cte}),
        ranked AS (
            SELECT
                rank() OVER (ORDER BY h.quantity DESC) AS rank,
                u.uid, u.nickname, u.nick_number, u.level, u.avatar_id,
                s.code AS server, sc.grade,
                h.quantity
            {HOLDERS_JOIN}
            LEFT JOIN user_scores sc ON sc.user_id = u.id
            WHERE {VISIBLE}
        )
        SELECT * FROM ranked
        WHERE TRUE{tail}
        ORDER BY quantity DESC, uid
        LIMIT ${limit_idx} OFFSET ${offset_idx}
        ",
        cte = holding.cte,
        tail = filters.sql,
    );
    let mut qry = sqlx::query_as::<_, ItemLeaderboardEntry>(&sql);
    for p in holding
        .params(item_id)
        .into_iter()
        .chain(filters.params.iter().map(String::as_str))
    {
        qry = qry.bind(p);
    }
    qry.bind(limit).bind(offset).fetch_all(pool).await
}

/// Visible holders of `item_id` matching the same filters as the page: the
/// row count behind `get_item_leaderboard`.
pub async fn count_item_holders(
    pool: &PgPool,
    item_id: &str,
    server: Option<&str>,
    q: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let holding = Holding::of(item_id);
    let filters = Filters::new(
        holding.next_param(),
        server,
        q,
        ("s.code", "u.nickname", "u.uid"),
    );
    let sql = format!(
        r"
        WITH holding AS ({cte})
        SELECT count(*)
        {HOLDERS_JOIN}
        WHERE {VISIBLE}{tail}
        ",
        cte = holding.cte,
        tail = filters.sql,
    );
    let mut qry = sqlx::query_scalar::<_, i64>(&sql);
    for p in holding
        .params(item_id)
        .into_iter()
        .chain(filters.params.iter().map(String::as_str))
    {
        qry = qry.bind(p);
    }
    qry.fetch_one(pool).await
}

/// Holder count and top holding for every item at least one visible player
/// holds, currencies included. One pass over `user_items` and one over
/// `user_status`; the currency VALUES list is generated from
/// `CURRENCY_COLUMNS` so the two can never disagree.
pub async fn get_item_catalog(
    pool: &PgPool,
    server: Option<&str>,
) -> Result<Vec<ItemHoldingSummary>, sqlx::Error> {
    let server_sql = server.map_or("", |_| " AND s.code = $1");
    let currency_values = currency_values_sql();
    let sql = format!(
        r"
        WITH visible AS (
            SELECT u.id
            FROM users u
            JOIN servers s ON s.id = u.server_id
            WHERE {VISIBLE}{server_sql}
        )
        SELECT ui.item_id, count(*)::bigint AS holders, max(ui.quantity)::bigint AS top
        FROM user_items ui
        JOIN visible v ON v.id = ui.user_id
        WHERE ui.quantity > 0
        GROUP BY ui.item_id
        UNION ALL
        SELECT c.item_id, count(*) FILTER (WHERE c.q > 0)::bigint AS holders, coalesce(max(c.q), 0)::bigint AS top
        FROM user_status st
        JOIN visible v ON v.id = st.user_id
        CROSS JOIN LATERAL (VALUES {currency_values}) AS c(item_id, q)
        GROUP BY c.item_id
        "
    );
    let mut qry = sqlx::query_as::<_, ItemHoldingSummary>(&sql);
    if let Some(code) = server {
        qry = qry.bind(code);
    }
    let mut rows = qry.fetch_all(pool).await?;
    rows.retain(|r| r.holders > 0);
    rows.sort_by(|a, b| {
        b.holders
            .cmp(&a.holders)
            .then_with(|| a.item_id.cmp(&b.item_id))
    });
    Ok(rows)
}

/// Where one player sits among visible holders of `item_id`, globally and on
/// their own server. `None` when they hold none of it, or are not visible.
pub async fn get_item_standing(
    pool: &PgPool,
    item_id: &str,
    uid: &str,
    server: &str,
) -> Result<Option<ItemStanding>, sqlx::Error> {
    let holding = Holding::of(item_id);
    let uid_idx = holding.next_param();
    let server_idx = uid_idx + 1;
    let item_idx = server_idx + 1;
    let sql = format!(
        r"
        WITH holding AS ({cte}),
        ranked AS (
            SELECT
                u.uid, s.code AS server, h.quantity,
                rank() OVER (ORDER BY h.quantity DESC) AS rank_global,
                count(*) OVER () AS holders_global,
                rank() OVER (PARTITION BY u.server_id ORDER BY h.quantity DESC) AS rank_server,
                count(*) OVER (PARTITION BY u.server_id) AS holders_server
            {HOLDERS_JOIN}
            WHERE {VISIBLE}
        )
        SELECT ${item_idx}::text AS item_id, quantity, rank_global, holders_global, rank_server, holders_server
        FROM ranked
        WHERE uid = ${uid_idx} AND server = ${server_idx}
        ",
        cte = holding.cte,
    );
    let mut qry = sqlx::query_as::<_, ItemStanding>(&sql);
    for p in holding.params(item_id) {
        qry = qry.bind(p);
    }
    qry.bind(uid)
        .bind(server)
        .bind(item_id)
        .fetch_optional(pool)
        .await
}
