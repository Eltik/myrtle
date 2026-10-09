//! The admin panel's people list: every account, private profiles included,
//! searched by nickname or game UID and cut by role and server.
//!
//! Unlike the public player search this reads `users` directly rather than
//! `v_user_profile`: the view computes four roster counts per row for every
//! row it scans, and the list only needs them for the page it returns. So
//! the page is cut first and the counts run on its rows only. Measured on
//! the local copy (2,629 users, 619,706 roster rows), the unfiltered first
//! page of 50 runs in 5 to 7 ms warm (114 ms on a cold buffer cache), a
//! 200-row page under a nickname-or-UID query and a server in 13 to 21 ms,
//! and either count under 0.3 ms. The `users` scan is sequential and costs
//! about 2 ms; the per-row counts are index-only scans and dominate.

use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::core::hypergryph::constants::Server;
use crate::database::models::user::AdminUserEntry;

/// The People tab's role filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminUserRole {
    All,
    /// The tier-list staff: super-admins, tier list admins and editors.
    Staff,
    Translators,
    /// Plain accounts with no panel role.
    Players,
}

impl AdminUserRole {
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "all" => Some(Self::All),
            "staff" => Some(Self::Staff),
            "translators" => Some(Self::Translators),
            "players" => Some(Self::Players),
            _ => None,
        }
    }

    /// The `users.role` values the filter admits, `None` for every role.
    const fn roles(self) -> Option<&'static [&'static str]> {
        match self {
            Self::All => None,
            Self::Staff => Some(&["super_admin", "tier_list_admin", "tier_list_editor"]),
            Self::Translators => Some(&["translator"]),
            Self::Players => Some(&["user"]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminUserSearch<'a> {
    /// Nickname substring or UID prefix, already trimmed and non-empty.
    pub q: Option<&'a str>,
    pub role: AdminUserRole,
    pub server: Option<Server>,
}

impl AdminUserSearch<'_> {
    /// `FROM ... WHERE ...` shared by the page and its count.
    fn push_from_where(&self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push(" FROM users u WHERE true");
        if let Some(q) = self.q {
            // The same nickname match the public search runs, plus the UID:
            // a prefix, so a full UID is an exact hit.
            let escaped = escape_like(q);
            qb.push(" AND (u.nickname ILIKE ");
            qb.push_bind(format!("%{escaped}%"));
            qb.push(" ESCAPE '\\' OR u.uid LIKE ");
            qb.push_bind(format!("{escaped}%"));
            qb.push(" ESCAPE '\\')");
        }
        if let Some(roles) = self.role.roles() {
            qb.push(" AND u.role = ANY(");
            qb.push_bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>());
            qb.push(")");
        }
        if let Some(server) = self.server {
            qb.push(" AND u.server_id = ");
            qb.push_bind(server.index() as i16);
        }
    }

    /// One page: an exact UID hit first, then the most privileged roles, then
    /// by nickname, with the row id as the final tie-break so paging is
    /// stable.
    pub async fn fetch_page(
        &self,
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AdminUserEntry>, sqlx::Error> {
        let mut qb = QueryBuilder::new(
            "SELECT p.id, p.uid, s.code AS server, p.nickname, p.nick_number, p.level, p.role, \
             p.avatar_id, p.secretary, p.secretary_skin_id, \
             COALESCE(us.public_profile, false) AS public_profile, \
             sc.total_score, sc.grade, \
             (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id) AS operator_count, \
             (SELECT count(*) FROM user_items ui WHERE ui.user_id = p.id) AS item_count, \
             (SELECT count(*) FROM user_skins sk WHERE sk.user_id = p.id) AS skin_count, \
             p.created_at, p.updated_at \
             FROM (SELECT u.*, ",
        );
        push_order_keys(&mut qb, self.q);
        self.push_from_where(&mut qb);
        qb.push(" ORDER BY exact_hit DESC, role_rank, lower(u.nickname) NULLS LAST, u.id LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
        qb.push(
            ") p \
             JOIN servers s ON s.id = p.server_id \
             LEFT JOIN user_settings us ON us.user_id = p.id \
             LEFT JOIN user_scores sc ON sc.user_id = p.id \
             ORDER BY p.exact_hit DESC, p.role_rank, lower(p.nickname) NULLS LAST, p.id",
        );
        qb.build_query_as::<AdminUserEntry>().fetch_all(pool).await
    }

    /// How many accounts the filters admit: the page's `total`.
    pub async fn count(&self, pool: &PgPool) -> Result<i64, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT count(*)");
        self.push_from_where(&mut qb);
        qb.build_query_scalar::<i64>().fetch_one(pool).await
    }
}

/// `exact_hit` and `role_rank`, the page's first two sort keys.
fn push_order_keys(qb: &mut QueryBuilder<'_, Postgres>, q: Option<&str>) {
    match q {
        Some(q) => {
            qb.push("(u.uid = ");
            qb.push_bind(q.to_owned());
            qb.push(") AS exact_hit, ");
        }
        None => {
            qb.push("false AS exact_hit, ");
        }
    }
    qb.push(
        "CASE u.role WHEN 'super_admin' THEN 0 WHEN 'tier_list_admin' THEN 1 \
         WHEN 'tier_list_editor' THEN 2 WHEN 'translator' THEN 3 ELSE 4 END AS role_rank",
    );
}

/// Neither the nickname substring nor the UID prefix may read a typed `%` or
/// `_` as a wildcard. Paired with an explicit `ESCAPE '\'` on both sides.
fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_tokens_parse() {
        assert_eq!(AdminUserRole::parse("all"), Some(AdminUserRole::All));
        assert_eq!(AdminUserRole::parse("staff"), Some(AdminUserRole::Staff));
        assert_eq!(
            AdminUserRole::parse("translators"),
            Some(AdminUserRole::Translators)
        );
        assert_eq!(
            AdminUserRole::parse("players"),
            Some(AdminUserRole::Players)
        );
        assert_eq!(AdminUserRole::parse("Staff"), None);
        assert_eq!(AdminUserRole::parse("user"), None);
    }

    #[test]
    fn staff_excludes_translators_and_players() {
        let staff = AdminUserRole::Staff.roles().expect("staff names roles");
        assert!(!staff.contains(&"translator"));
        assert!(!staff.contains(&"user"));
        assert_eq!(AdminUserRole::All.roles(), None);
    }

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(escape_like("123"), "123");
        assert_eq!(escape_like("1%_\\"), "1\\%\\_\\\\");
    }
}
