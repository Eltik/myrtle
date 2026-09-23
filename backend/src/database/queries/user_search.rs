//! The player search: one page of public profiles, ranked by a roster metric
//! and cut by roster facts. Every metric is an aggregate over the roster
//! tables computed live per request, cached like any other search page.
//! Measured on 2,585 public players (620,118 rows in `user_operators`,
//! 1,294,458 in `user_operator_skills`), a cold page 1 costs 44.272 ms for
//! masteries and 190.532 ms for potentials against a 39.383 ms floor for the
//! plain score order, and warm pages land under 60 ms. That is why nothing
//! is precomputed into a stats table. Revisit if the population grows
//! tenfold: these are per-request full scans of the grouped table.

use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::database::models::user::SearchEntry;

/// How the page is ranked. Each variant names one aggregate; `Score` is the
/// view's own `total_score`, the order the search always had.
#[derive(Debug, Clone, Copy)]
pub enum Rank<'a> {
    Score,
    /// Owned operators.
    Operators,
    /// Game account registration time (`user_status.register_ts`).
    Joined,
    /// Enemies discovered: keys of the dexNav map.
    Enemies,
    /// Potential ranks summed over the roster: every duplicate counts one.
    Potentials,
    /// Skills at M3.
    Masteries,
    /// Unlocked modules at level 3.
    Modules,
    /// Skins owned.
    Skins,
    /// Owned operators among `ids`: a class or an archetype, resolved from
    /// game data by the caller.
    OwnedOf(&'a [String]),
}

impl Rank<'_> {
    /// Whether the rank is an aggregate with a `metric` CTE, or reads a
    /// profile column directly.
    const fn has_cte(&self) -> bool {
        !matches!(self, Self::Score | Self::Joined)
    }

    /// Pushes the `metric` CTE body, `(user_id, value)` for every user with
    /// any of the thing; users with none are absent and read as 0 in the
    /// page. Pushes nothing for the ranks without a CTE.
    fn push_cte(&self, qb: &mut QueryBuilder<'_, Postgres>) {
        match self {
            Self::Score | Self::Joined => {}
            Self::Operators => {
                qb.push(
                    "SELECT user_id, count(*)::bigint AS value FROM user_operators GROUP BY user_id",
                );
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

/// The user owns every operator in `ids` that their own server has released:
/// `required` is `(server code, count)` per loaded server, lower-cased to
/// match `servers.code` case-insensitively. A user on a server with no loaded
/// game data cannot be judged and drops out of the page.
#[derive(Debug, Clone)]
pub struct OwnsAll<'a> {
    pub ids: &'a [String],
    pub required: &'a [(String, i64)],
}

#[derive(Debug, Clone)]
pub struct UserSearch<'a> {
    /// Nickname substring, already trimmed and non-empty.
    pub q: Option<&'a str>,
    pub rank: Rank<'a>,
    pub descending: bool,
    /// Operator ids the user must all own; deduplicated by the caller so
    /// `count(*) = len` is an exact test. `(user_id, operator_id)` is a
    /// primary key, so the count cannot exceed the list and `>=` would read
    /// the same: the equality is the clearer statement, not a tighter one.
    pub has: &'a [String],
    /// An operator that must sit in one of the user's support slots.
    pub support: Option<&'a str>,
    pub owns_all: Option<OwnsAll<'a>>,
}

impl UserSearch<'_> {
    /// The `WITH` list: the rank's `metric`, and `owned` when the owns-all
    /// filter is on. Both are one grouped pass over a roster table, which is
    /// what keeps the filter off a per-row subquery as the population grows:
    /// on 2,585 public players the grouped form runs the heaviest filtered
    /// page in 27.213 ms against 59.040 ms for the correlated one.
    fn push_ctes(&self, qb: &mut QueryBuilder<'_, Postgres>, with_metric: bool) {
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
            qb.push("owned AS (SELECT user_id, count(*)::bigint AS n FROM user_operators WHERE operator_id = ANY(");
            qb.push_bind(all.ids.to_vec());
            qb.push(") GROUP BY user_id)");
        }
        qb.push(" ");
    }

    /// `FROM ... WHERE ...` shared by the page and its count. The page pushes
    /// the CTEs first, so the alias `m` exists only when `with_metric` is set.
    fn push_from_where(&self, qb: &mut QueryBuilder<'_, Postgres>, with_metric: bool) {
        qb.push(" FROM v_user_profile p");
        if with_metric {
            qb.push(" LEFT JOIN metric m ON m.user_id = p.id");
        }
        if let Some(all) = &self.owns_all {
            // An inner join, so a player who owns none of the scope drops out
            // here rather than passing a `0 >= 0` test. A scope that resolves
            // to no operators therefore matches nobody, which is the honest
            // answer for an archetype this game data does not have.
            let (servers, counts): (Vec<String>, Vec<i64>) = all.required.iter().cloned().unzip();
            qb.push(" JOIN owned o ON o.user_id = p.id JOIN unnest(");
            qb.push_bind(servers);
            qb.push("::text[], ");
            qb.push_bind(counts);
            qb.push("::bigint[]) AS req(server, n) ON req.server = lower(p.server)");
        }
        qb.push(" WHERE p.public_profile = true");
        if let Some(q) = self.q {
            qb.push(" AND p.nickname ILIKE ");
            qb.push_bind(format!("%{q}%"));
        }
        if !self.has.is_empty() {
            qb.push(
                " AND (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id AND uo.operator_id = ANY(",
            );
            qb.push_bind(self.has.to_vec());
            qb.push(")) = ");
            qb.push_bind(self.has.len() as i64);
        }
        if let Some(support) = self.support {
            qb.push(
                " AND EXISTS (SELECT 1 FROM user_support_units su WHERE su.user_id = p.id AND su.operator_id = ",
            );
            qb.push_bind(support.to_owned());
            qb.push(")");
        }
        if self.owns_all.is_some() {
            qb.push(" AND o.n >= req.n");
        }
    }

    /// One page. `metric` carries the rank's value per row (`NULL` under
    /// `Score`, where the profile's `total_score` already is the value).
    /// Ties fall back to score, then uid, so paging is stable.
    pub async fn fetch_page(
        &self,
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SearchEntry>, sqlx::Error> {
        let with_metric = self.rank.has_cte();
        let mut qb = QueryBuilder::new("");
        self.push_ctes(&mut qb, with_metric);
        qb.push("SELECT p.*, ");
        match self.rank {
            Rank::Score => qb.push("NULL::bigint AS metric"),
            Rank::Joined => qb.push("p.register_ts AS metric"),
            _ => qb.push("COALESCE(m.value, 0)::bigint AS metric"),
        };
        self.push_from_where(&mut qb, with_metric);
        let dir = if self.descending { "DESC" } else { "ASC" };
        match self.rank {
            Rank::Score => {
                qb.push(format!(" ORDER BY p.total_score {dir} NULLS LAST, p.uid"));
            }
            _ => {
                qb.push(format!(
                    " ORDER BY metric {dir} NULLS LAST, p.total_score DESC NULLS LAST, p.uid"
                ));
            }
        }
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
        qb.build_query_as::<SearchEntry>().fetch_all(pool).await
    }

    /// How many users the filters admit: the page's `total`.
    pub async fn count(&self, pool: &PgPool) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new("");
        self.push_ctes(&mut qb, false);
        qb.push("SELECT count(*)");
        self.push_from_where(&mut qb, false);
        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }
}
