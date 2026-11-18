use sqlx::{Postgres, QueryBuilder};

use crate::repos::users::ProfileRow;
use crate::sql::contains_pattern;
use crate::{Conn, DbError};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchRow {
    #[sqlx(flatten)]
    pub profile: ProfileRow,
    pub metric: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum Rank<'a> {
    Score,
    Operators,
    Joined,
    Enemies,
    Potentials,
    /// Skills at M3
    Masteries,
    /// Unlocked modules at level 3
    Modules,
    Skins,
    /// Owned operators among these ids
    OwnedOf(&'a [String]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy)]
pub struct OwnsAll<'a> {
    pub ids: &'a [String],
    pub required: &'a [(String, i64)],
}

#[derive(Debug, Clone, Copy)]
pub struct Search<'a> {
    /// Nickname substring
    pub q: Option<&'a str>,
    pub rank: Rank<'a>,
    pub direction: Direction,
    /// Operator ids the user must all own
    pub has: &'a [String],
    /// An operator in one of the user's support slots
    pub support: Option<&'a str>,
    pub owns_all: Option<OwnsAll<'a>>,
}

#[derive(Debug, Clone, Copy)]
enum Tab {
    Score,
    Roster,
    Enemies,
}

#[derive(Debug, Clone, Copy)]
enum HeaderField {
    Registered,
    Operators,
    Skins,
}

const PROFILE_COLUMNS: &str = "p.id, p.uid, p.nickname, p.nick_number, p.level, p.avatar_id, \
     p.secretary, p.secretary_skin_id, p.resume_id, p.role, p.server, p.total_score, p.grade, \
     p.public_profile, p.store_gacha, p.share_stats, p.exp, p.orundum, p.lmd, p.sanity, \
     p.max_sanity, p.gacha_tickets, p.ten_pull_tickets, p.monthly_sub_end, p.register_ts, \
     p.last_online_ts, p.resume, p.friend_num_limit, p.cumulative_signin, p.operator_count, \
     p.item_count, p.skin_count, p.non_default_skin_count, p.updated_at, p.originite, \
     p.profile_layout";

impl Tab {
    const fn visible_sql(self) -> &'static str {
        match self {
            Self::Score => {
                r#"NOT COALESCE(p.profile_layout -> 'tabs' @> '[{"id":"score","visible":false}]'::jsonb, false)"#
            }
            Self::Roster => {
                r#"NOT COALESCE(p.profile_layout -> 'tabs' @> '[{"id":"roster","visible":false}]'::jsonb, false)"#
            }
            Self::Enemies => {
                r#"NOT COALESCE(p.profile_layout -> 'tabs' @> '[{"id":"enemies","visible":false}]'::jsonb, false)"#
            }
        }
    }
}

impl HeaderField {
    const fn visible_sql(self) -> &'static str {
        match self {
            Self::Registered => {
                r#"NOT COALESCE(p.profile_layout -> 'header' -> 'hidden' @> '["registered"]'::jsonb, false)"#
            }
            Self::Operators => {
                r#"NOT COALESCE(p.profile_layout -> 'header' -> 'hidden' @> '["operators"]'::jsonb, false)"#
            }
            Self::Skins => {
                r#"NOT COALESCE(p.profile_layout -> 'header' -> 'hidden' @> '["skins"]'::jsonb, false)"#
            }
        }
    }
}

impl Direction {
    const fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

impl Rank<'_> {
    const fn header_field(self) -> Option<HeaderField> {
        match self {
            Self::Joined => Some(HeaderField::Registered),
            Self::Operators => Some(HeaderField::Operators),
            Self::Skins => Some(HeaderField::Skins),
            _ => None,
        }
    }

    const fn has_cte(self) -> bool {
        !matches!(self, Self::Score | Self::Joined)
    }

    const fn reads_roster(self) -> bool {
        matches!(
            self,
            Self::Operators
                | Self::Potentials
                | Self::Masteries
                | Self::Modules
                | Self::Skins
                | Self::OwnedOf(_)
        )
    }

    fn push_cte(self, qb: &mut QueryBuilder<Postgres>) {
        match self {
            Self::Score | Self::Joined => {}
            Self::Operators => {
                qb.push("SELECT user_id, count(*)::bigint AS value FROM user_operators GROUP BY user_id");
            }
            Self::Enemies => {
                qb.push(
                    "SELECT e.user_id, (SELECT count(*) FROM jsonb_object_keys(e.enemies))::bigint AS value \
                     FROM user_enemy_progress e",
                );
            }
            Self::Potentials => {
                qb.push(
                    "SELECT user_id, sum(potential)::bigint AS value FROM user_operators GROUP BY user_id",
                );
            }
            Self::Masteries => {
                qb.push(
                    "SELECT user_id, count(*)::bigint AS value FROM user_operator_skills \
                     WHERE specialize_level = 3 GROUP BY user_id",
                );
            }
            Self::Modules => {
                qb.push(
                    "SELECT user_id, count(*)::bigint AS value FROM user_operator_modules \
                     WHERE module_level = 3 AND NOT locked GROUP BY user_id",
                );
            }
            Self::Skins => {
                qb.push(
                    "SELECT user_id, count(*)::bigint AS value FROM user_skins GROUP BY user_id",
                );
            }
            Self::OwnedOf(ids) => {
                qb.push(
                    "SELECT user_id, count(*)::bigint AS value FROM user_operators WHERE operator_id = ANY(",
                );
                qb.push_bind(ids.to_vec());
                qb.push(") GROUP BY user_id");
            }
        }
    }
}

impl Search<'_> {
    fn tabs_read(&self) -> Vec<Tab> {
        let mut tabs = Vec::new();
        if self.rank.reads_roster() || !self.has.is_empty() || self.owns_all.is_some() {
            tabs.push(Tab::Roster);
        }
        if matches!(self.rank, Rank::Enemies) {
            tabs.push(Tab::Enemies);
        }
        tabs
    }

    fn push_ctes(&self, qb: &mut QueryBuilder<Postgres>, with_metric: bool) {
        if !with_metric && self.owns_all.is_none() {
            return;
        }
        qb.push("WITH ");
        if with_metric {
            qb.push("metric AS (");
            self.rank.push_cte(qb);
            qb.push(")");
        }
        if let Some(all) = &self.owns_all {
            if with_metric {
                qb.push(", ");
            }
            qb.push(
                "owned AS (SELECT user_id, count(*)::bigint AS n FROM user_operators \
                 WHERE operator_id = ANY(",
            );
            qb.push_bind(all.ids.to_vec());
            qb.push(") GROUP BY user_id)");
        }
        qb.push(" ");
    }

    fn push_from_where(&self, qb: &mut QueryBuilder<Postgres>, with_metric: bool) {
        qb.push(" FROM v_user_profile p");
        if with_metric {
            qb.push(" LEFT JOIN metric m ON m.user_id = p.id");
        }
        if let Some(all) = &self.owns_all {
            let (servers, counts): (Vec<String>, Vec<i64>) = all.required.iter().cloned().unzip(); // inner join: an empty scope matches nobody, not `0 >= 0`
            qb.push(" JOIN owned o ON o.user_id = p.id JOIN unnest(");
            qb.push_bind(servers);
            qb.push("::text[], ");
            qb.push_bind(counts);
            qb.push("::bigint[]) AS req(server, n) ON req.server = lower(p.server)");
        }
        qb.push(" WHERE p.public_profile = true");
        for tab in self.tabs_read() {
            qb.push(" AND ");
            qb.push(tab.visible_sql());
        }
        if let Some(field) = self.rank.header_field() {
            qb.push(" AND ");
            qb.push(field.visible_sql());
        }
        if let Some(q) = self.q {
            qb.push(" AND p.nickname ILIKE ");
            qb.push_bind(contains_pattern(q));
            qb.push(" ESCAPE '\\'");
        }
        if !self.has.is_empty() {
            qb.push(" AND p.id IN (SELECT user_id FROM user_operators WHERE operator_id = ANY(");
            let mut has = self.has.to_vec();
            has.sort_unstable();
            has.dedup();
            let owned = has.len() as i64;
            qb.push_bind(has);
            qb.push(") GROUP BY user_id HAVING count(*) = ");
            qb.push_bind(owned);
            qb.push(")");
        }
        if let Some(support) = self.support {
            qb.push(
                " AND EXISTS (SELECT 1 FROM user_support_units su \
                 WHERE su.user_id = p.id AND su.operator_id = ",
            );
            qb.push_bind(support.to_owned());
            qb.push(")");
        }
        if self.owns_all.is_some() {
            qb.push(" AND o.n >= req.n");
        }
    }
}

pub async fn page(
    conn: &mut Conn,
    search: &Search<'_>,
    limit: i64,
    offset: i64,
) -> Result<Vec<SearchRow>, DbError> {
    let with_metric = search.rank.has_cte();
    let mut qb = QueryBuilder::<Postgres>::new("");
    search.push_ctes(&mut qb, with_metric);
    qb.push("SELECT ");
    qb.push(PROFILE_COLUMNS);
    qb.push(match search.rank {
        Rank::Score => ", NULL::bigint AS metric",
        Rank::Joined => ", p.register_ts AS metric",
        _ => ", COALESCE(m.value, 0)::bigint AS metric",
    });
    search.push_from_where(&mut qb, with_metric);
    let score = format!(
        "CASE WHEN {} THEN p.total_score END",
        Tab::Score.visible_sql()
    );
    let dir = search.direction.sql();
    match search.rank {
        Rank::Score => qb.push(format!(" ORDER BY {score} {dir} NULLS LAST, p.uid")),
        _ => qb.push(format!(
            " ORDER BY metric {dir} NULLS LAST, {score} DESC NULLS LAST, p.uid"
        )),
    };
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    Ok(qb.build_query_as::<SearchRow>().fetch_all(conn).await?)
}

pub async fn count(conn: &mut Conn, search: &Search<'_>) -> Result<i64, DbError> {
    let mut qb = QueryBuilder::<Postgres>::new("");
    search.push_ctes(&mut qb, false);
    qb.push("SELECT count(*)");
    search.push_from_where(&mut qb, false);
    Ok(qb.build_query_scalar::<i64>().fetch_one(conn).await?)
}
