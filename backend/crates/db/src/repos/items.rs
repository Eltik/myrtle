use myrtle_types::{Server, UserId};

use crate::sql::contains_pattern;
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct ItemRow {
    pub item_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemLeaderboardRow {
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub grade: Option<String>,
    pub quantity: i64,
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct HoldingTotals {
    pub holders: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone)]
pub struct ItemHoldingRow {
    pub item_id: String,
    pub holders: i64,
    pub top: i64,
    pub total_quantity: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemStandingRow {
    pub item_id: String,
    pub quantity: i64,
    pub rank_global: i64,
    pub holders_global: i64,
    pub rank_server: i64,
    pub holders_server: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Lmd,
    Originite,
    Orundum,
    HggShard,
    LggShard,
    RecruitPermits,
    GachaTickets,
    TenPullTickets,
    PracticeTickets,
    SocialPoint,
    ClassicGachaTickets,
    ClassicTenPullTickets,
}

impl Currency {
    pub const ALL: [Self; 12] = [
        Self::Lmd,
        Self::Originite,
        Self::Orundum,
        Self::HggShard,
        Self::LggShard,
        Self::RecruitPermits,
        Self::GachaTickets,
        Self::TenPullTickets,
        Self::PracticeTickets,
        Self::SocialPoint,
        Self::ClassicGachaTickets,
        Self::ClassicTenPullTickets,
    ];

    #[must_use]
    pub const fn item_id(self) -> &'static str {
        match self {
            Self::Lmd => "4001",
            Self::Originite => "4002",
            Self::Orundum => "4003",
            Self::HggShard => "4004",
            Self::LggShard => "4005",
            Self::RecruitPermits => "7001",
            Self::GachaTickets => "7003",
            Self::TenPullTickets => "7004",
            Self::PracticeTickets => "6001",
            Self::SocialPoint => "SOCIAL_PT",
            Self::ClassicGachaTickets => "classic_gacha",
            Self::ClassicTenPullTickets => "classic_gacha_10",
        }
    }

    #[must_use]
    pub fn from_item_id(item_id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.item_id() == item_id)
    }

    const fn column(self) -> &'static str {
        match self {
            Self::Lmd => "lmd",
            Self::Originite => "originite",
            Self::Orundum => "orundum",
            Self::HggShard => "hgg_shard",
            Self::LggShard => "lgg_shard",
            Self::RecruitPermits => "recruit_permits",
            Self::GachaTickets => "gacha_tickets",
            Self::TenPullTickets => "ten_pull_tickets",
            Self::PracticeTickets => "practice_tickets",
            Self::SocialPoint => "social_point",
            Self::ClassicGachaTickets => "classic_gacha_tickets",
            Self::ClassicTenPullTickets => "classic_ten_pull_tickets",
        }
    }

    #[must_use]
    pub const fn header_field(self) -> Option<&'static str> {
        match self {
            Self::Lmd => Some("lmd"),
            Self::Originite => Some("originite"),
            Self::Orundum => Some("orundum"),
            Self::GachaTickets => Some("headhunting_permits"),
            Self::TenPullTickets => Some("ten_pull_permits"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holding<'a> {
    Currency(Currency),
    Item(&'a str),
}

impl<'a> Holding<'a> {
    #[must_use]
    pub fn of(item_id: &'a str) -> Self {
        Currency::from_item_id(item_id).map_or(Self::Item(item_id), Self::Currency)
    }

    fn cte(self, item_param: usize) -> String {
        match self {
            Self::Currency(currency) => {
                let col = currency.column();
                let hidden = currency.header_field().map_or_else(String::new, |field| {
                    format!(
                        " AND NOT EXISTS (SELECT 1 FROM user_settings hs WHERE hs.user_id = st.user_id AND hs.profile_layout -> 'header' -> 'hidden' @> '[\"{field}\"]'::jsonb)"
                    )
                });
                format!(
                    "SELECT st.user_id, st.{col}::bigint AS quantity FROM user_status st WHERE st.{col} > 0{hidden}"
                )
            }
            Self::Item(_) => format!(
                "SELECT ui.user_id, ui.quantity::bigint AS quantity FROM user_items ui WHERE ui.item_id = ${item_param} AND ui.quantity > 0"
            ),
        }
    }
}

const VISIBLE: &str = r#"EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile AND NOT COALESCE(us.profile_layout -> 'tabs' @> '[{"id":"inventory","visible":false}]'::jsonb, false))"#;

pub async fn inventory(conn: &mut Conn, user: UserId) -> Result<Vec<ItemRow>, DbError> {
    Ok(sqlx::query_as!(
        ItemRow,
        r#"SELECT item_id AS "item_id!", quantity AS "quantity!" FROM user_items WHERE user_id = $1
           UNION ALL
           SELECT c.item_id, c.q::int AS quantity
           FROM user_status st
           CROSS JOIN LATERAL (VALUES
               ('4001', st.lmd::bigint), ('4002', st.originite::bigint),
               ('4003', st.orundum::bigint), ('4004', st.hgg_shard::bigint),
               ('4005', st.lgg_shard::bigint), ('7001', st.recruit_permits::bigint),
               ('7003', st.gacha_tickets::bigint), ('7004', st.ten_pull_tickets::bigint),
               ('6001', st.practice_tickets::bigint), ('SOCIAL_PT', st.social_point::bigint),
               ('classic_gacha', st.classic_gacha_tickets::bigint),
               ('classic_gacha_10', st.classic_ten_pull_tickets::bigint)
           ) AS c(item_id, q)
           WHERE st.user_id = $1 AND c.q > 0
           ORDER BY 1"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn leaderboard(
    conn: &mut Conn,
    holding: Holding<'_>,
    server: Option<Server>,
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ItemLeaderboardRow>, DbError> {
    let sql = format!(
        r#"WITH holding AS ({cte}),
           ranked AS (
               SELECT rank() OVER (ORDER BY h.quantity DESC) AS rank,
                      u.uid, u.nickname, u.nick_number,
                      CASE WHEN NOT EXISTS (SELECT 1 FROM user_settings hs WHERE hs.user_id = u.id AND hs.profile_layout -> 'header' -> 'hidden' @> '["level"]'::jsonb) THEN u.level END AS level,
                      u.avatar_id, s.code AS server, sc.grade, h.quantity
               FROM holding h
               JOIN users u ON u.id = h.user_id
               JOIN servers s ON s.id = u.server_id
               LEFT JOIN user_scores sc ON sc.user_id = u.id
               WHERE {VISIBLE}
           )
           SELECT rank, uid, nickname, nick_number, level, avatar_id, server, grade, quantity
           FROM ranked
           WHERE ($1::text IS NULL OR server = $1)
             AND ($2::text IS NULL OR nickname ILIKE $2 ESCAPE '\' OR uid ILIKE $2 ESCAPE '\')
           ORDER BY quantity DESC, uid
           LIMIT $3 OFFSET $4"#,
        cte = holding.cte(5),
    );
    let mut query = sqlx::query_as::<_, ItemLeaderboardRow>(sqlx::AssertSqlSafe(sql))
        .bind(server.map(Server::as_str))
        .bind(q.map(contains_pattern))
        .bind(limit)
        .bind(offset);
    if let Holding::Item(item_id) = holding {
        query = query.bind(item_id);
    }
    Ok(query.fetch_all(conn).await?)
}

pub async fn holding_totals(
    conn: &mut Conn,
    holding: Holding<'_>,
    server: Option<Server>,
    q: Option<&str>,
) -> Result<HoldingTotals, DbError> {
    let sql = format!(
        r"WITH holding AS ({cte})
          SELECT count(*) AS holders, coalesce(sum(h.quantity), 0)::bigint AS quantity
          FROM holding h
          JOIN users u ON u.id = h.user_id
          JOIN servers s ON s.id = u.server_id
          WHERE {VISIBLE}
            AND ($1::text IS NULL OR s.code = $1)
            AND ($2::text IS NULL OR u.nickname ILIKE $2 ESCAPE '\' OR u.uid ILIKE $2 ESCAPE '\')",
        cte = holding.cte(3),
    );
    let mut query = sqlx::query_as::<_, HoldingTotals>(sqlx::AssertSqlSafe(sql))
        .bind(server.map(Server::as_str))
        .bind(q.map(contains_pattern));
    if let Holding::Item(item_id) = holding {
        query = query.bind(item_id);
    }
    Ok(query.fetch_one(conn).await?)
}

pub async fn standing(
    conn: &mut Conn,
    holding: Holding<'_>,
    uid: &str,
    server: Server,
) -> Result<Option<ItemStandingRow>, DbError> {
    let item_id = match holding {
        Holding::Currency(currency) => currency.item_id(),
        Holding::Item(item_id) => item_id,
    };
    let sql = format!(
        r"WITH holding AS ({cte}),
          ranked AS (
              SELECT u.uid, s.code AS server, h.quantity,
                     rank() OVER (ORDER BY h.quantity DESC) AS rank_global,
                     count(*) OVER () AS holders_global,
                     rank() OVER (PARTITION BY u.server_id ORDER BY h.quantity DESC) AS rank_server,
                     count(*) OVER (PARTITION BY u.server_id) AS holders_server
              FROM holding h
              JOIN users u ON u.id = h.user_id
              JOIN servers s ON s.id = u.server_id
              WHERE {VISIBLE}
          )
          SELECT $3::text AS item_id, quantity, rank_global, holders_global, rank_server, holders_server
          FROM ranked
          WHERE uid = $1 AND server = $2",
        cte = holding.cte(3),
    );
    Ok(
        sqlx::query_as::<_, ItemStandingRow>(sqlx::AssertSqlSafe(sql))
            .bind(uid)
            .bind(server.as_str())
            .bind(item_id)
            .fetch_optional(conn)
            .await?,
    )
}

pub async fn catalog(
    conn: &mut Conn,
    server: Option<Server>,
) -> Result<Vec<ItemHoldingRow>, DbError> {
    let mut rows = sqlx::query_as!(
        ItemHoldingRow,
        r#"WITH visible AS (
               SELECT u.id, us.profile_layout -> 'header' -> 'hidden' AS hidden
               FROM users u
               JOIN servers s ON s.id = u.server_id
               JOIN user_settings us ON us.user_id = u.id
               WHERE us.public_profile
                 AND NOT COALESCE(us.profile_layout -> 'tabs' @> '[{"id":"inventory","visible":false}]'::jsonb, false)
                 AND ($1::text IS NULL OR s.code = $1)
           )
           SELECT ui.item_id AS "item_id!", count(*)::bigint AS "holders!",
                  max(ui.quantity)::bigint AS "top!", sum(ui.quantity)::bigint AS "total_quantity!"
           FROM user_items ui
           JOIN visible v ON v.id = ui.user_id
           WHERE ui.quantity > 0
           GROUP BY ui.item_id
           UNION ALL
           SELECT c.item_id, count(*) FILTER (WHERE c.q > 0)::bigint, coalesce(max(c.q), 0)::bigint,
                  coalesce(sum(c.q) FILTER (WHERE c.q > 0), 0)::bigint
           FROM user_status st
           JOIN visible v ON v.id = st.user_id
           CROSS JOIN LATERAL (VALUES
               ('4001', CASE WHEN NOT COALESCE(v.hidden @> '["lmd"]', false) THEN st.lmd::bigint END),
               ('4002', CASE WHEN NOT COALESCE(v.hidden @> '["originite"]', false) THEN st.originite::bigint END),
               ('4003', CASE WHEN NOT COALESCE(v.hidden @> '["orundum"]', false) THEN st.orundum::bigint END),
               ('4004', st.hgg_shard::bigint),
               ('4005', st.lgg_shard::bigint),
               ('7001', st.recruit_permits::bigint),
               ('7003', CASE WHEN NOT COALESCE(v.hidden @> '["headhunting_permits"]', false) THEN st.gacha_tickets::bigint END),
               ('7004', CASE WHEN NOT COALESCE(v.hidden @> '["ten_pull_permits"]', false) THEN st.ten_pull_tickets::bigint END),
               ('6001', st.practice_tickets::bigint),
               ('SOCIAL_PT', st.social_point::bigint),
               ('classic_gacha', st.classic_gacha_tickets::bigint),
               ('classic_gacha_10', st.classic_ten_pull_tickets::bigint)
           ) AS c(item_id, q)
           GROUP BY c.item_id"#,
        server.map(Server::as_str),
    )
    .fetch_all(conn)
    .await?;
    rows.retain(|r| r.holders > 0);
    rows.sort_by(|a, b| {
        b.holders
            .cmp(&a.holders)
            .then_with(|| a.item_id.cmp(&b.item_id))
    });
    Ok(rows)
}

pub async fn count_visible_players(
    conn: &mut Conn,
    server: Option<Server>,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!"
           FROM users u JOIN servers s ON s.id = u.server_id
           WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile AND NOT COALESCE(us.profile_layout -> 'tabs' @> '[{"id":"inventory","visible":false}]'::jsonb, false))
             AND ($1::text IS NULL OR s.code = $1)"#,
        server.map(Server::as_str),
    )
    .fetch_one(conn)
    .await?)
}
