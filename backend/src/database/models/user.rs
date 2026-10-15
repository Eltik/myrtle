use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

use crate::database::models::profile_layout::{ProfileHeaderField, ProfileLayout, ProfileTabId};

/// `v_user_profile` view
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub role: String,
    pub server: String, // FROM servers.code
    // Scores (LEFT JOIN)
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    // Settings (LEFT JOIN)
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    // Status (LEFT JOIN)
    pub exp: Option<i32>,
    pub orundum: Option<i32>,
    pub lmd: Option<i32>,
    pub sanity: Option<i16>,
    pub max_sanity: Option<i16>,
    pub gacha_tickets: Option<i32>,
    pub ten_pull_tickets: Option<i32>,
    #[ts(type = "number | null")]
    pub monthly_sub_end: Option<i64>,
    #[ts(type = "number | null")]
    pub register_ts: Option<i64>,
    #[ts(type = "number | null")]
    pub last_online_ts: Option<i64>,
    pub resume: Option<String>,
    pub friend_num_limit: Option<i16>,
    pub cumulative_signin: Option<i32>,
    // Counts
    #[ts(type = "number | null")]
    pub operator_count: Option<i64>,
    #[ts(type = "number | null")]
    pub item_count: Option<i64>,
    #[ts(type = "number | null")]
    pub skin_count: Option<i64>,
    #[ts(type = "number | null")]
    pub non_default_skin_count: Option<i64>,
    pub updated_at: DateTime<Utc>,
    /// Originite Prime on the account (paid and free pools summed) as of the last sync.
    pub originite: Option<i32>,
    /// The owner's tab order and visibility, showcase, header art and header
    /// numbers, normalized on read. `None` when they never customized, which
    /// the client renders exactly as before layouts existed. A visitor
    /// receives [`UserProfile::hide_private`]'s projection, which lists only
    /// the visible tabs and nulls the header numbers the owner hid.
    #[sqlx(json(nullable))]
    pub profile_layout: Option<ProfileLayout>,
}

impl UserProfile {
    /// The profile as someone other than its owner may see it: private tabs
    /// are dropped from the layout, and `total_score` and `grade` go when the
    /// Score tab is private, since they are that tab's headline and
    /// `/get-user-score` refuses it to a visitor. Each header number the
    /// owner hid is `None`, and the layout's strip loses its card. The rest of
    /// the row feeds the header, which every visible tab sits under, so it
    /// stays. A NULL layout changes nothing.
    pub fn hide_private(&mut self) {
        let Some(layout) = &mut self.profile_layout else {
            return;
        };
        if !layout.is_visible(ProfileTabId::Score) {
            self.total_score = None;
            self.grade = None;
        }
        for field in ProfileHeaderField::ALL {
            if layout.header_shows(field) {
                continue;
            }
            match field {
                ProfileHeaderField::Level => {
                    self.level = None;
                    self.exp = None;
                }
                ProfileHeaderField::Registered => self.register_ts = None,
                ProfileHeaderField::SignIn => self.cumulative_signin = None,
                ProfileHeaderField::Operators => self.operator_count = None,
                ProfileHeaderField::Skins => {
                    self.skin_count = None;
                    self.non_default_skin_count = None;
                }
                ProfileHeaderField::Items => self.item_count = None,
                ProfileHeaderField::Orundum => self.orundum = None,
                ProfileHeaderField::Originite => self.originite = None,
                ProfileHeaderField::Lmd => self.lmd = None,
                ProfileHeaderField::HeadhuntingPermits => self.gacha_tickets = None,
                ProfileHeaderField::TenPullPermits => self.ten_pull_tickets = None,
            }
        }
        *layout = layout.visible_only();
    }
}

/// One player search row: the profile plus the value of the sort that
/// ranked it. `metric` is the count for a count sort and `register_ts` for
/// the join-date sort; it is `None` under the score sort, where the
/// profile's `total_score` already is the value.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchEntry {
    #[serde(flatten)]
    #[sqlx(flatten)]
    #[ts(flatten)]
    pub profile: UserProfile,
    #[ts(type = "number | null")]
    pub metric: Option<i64>,
}

/// One row of the admin panel's people list (`GET /admin/users`): the
/// account, its public-profile flag and score, and its roster counts.
/// Private profiles are included; only staff can read this.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AdminUserEntry {
    pub id: Uuid,
    pub uid: String,
    /// `servers.code`, e.g. `EN`.
    pub server: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    /// `false` when the account never saved settings: the profile gate reads
    /// a missing row as private.
    pub public_profile: bool,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    #[ts(type = "number")]
    pub operator_count: i64,
    #[ts(type = "number")]
    pub item_count: i64,
    #[ts(type = "number")]
    pub skin_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// users table
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub role: String,
    pub avatar_id: Option<String>,
    pub resume_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `user_settings` table
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub public_profile: bool,
    pub store_gacha: bool,
    pub share_stats: bool,
    pub updated_at: DateTime<Utc>,
}

/// `user_checkin` table: daily sign-in state from the game's `checkIn` section,
/// joined with the timestamps needed to render it as a calendar.
///
/// The game's monthly sign-in is a *sequential list of reward slots*, not a
/// dated calendar: missing a day leaves you one slot behind, it never forfeits
/// the slot. Nothing in this payload maps a claim to the date it happened on,
/// so no consumer can say "the 4th was missed", only "N of the month's slots
/// are claimed, D days have elapsed".
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserCheckin {
    /// One flag per sign-in claimed this month, in claim order: `1` if the
    /// monthly-subscription Daily Supply came with that claim, `0` if not.
    ///
    /// The game's raw `checkInHistory`, not a per-day calendar: a missed day
    /// adds no entry, so `flags[d - 1]` is not "day `d` was claimed", and the
    /// `1`s count monthly card days, not sign-ins. (Checked against
    /// `user_status.monthly_sub_end`: the final flag agrees with subscription
    /// state in 2 193 of 2 219 non-empty rows.) Use [`Self::claimed_this_month`]
    /// for the sign-in count.
    #[sqlx(rename = "history")]
    pub monthly_card_flags: Vec<i16>,
    /// Sign-ins claimed in the current month's series: the length of
    /// `monthly_card_flags`, derived in SQL so no client re-derives it.
    pub claimed_this_month: i32,
    /// Lifetime cumulative sign-in days (the "total days of sign-ins" counter).
    /// `None` only for a visitor when the owner hid it from their header
    /// ([`UserCheckin::hide_private`]).
    pub cumulative_signin: Option<i32>,
    /// Active monthly sign-in series id (e.g. `signin<N>`).
    pub checkin_group_id: Option<String>,
    /// The game's raw `checkInRewardIndex`: a 0-based pointer into the month's
    /// reward slots. It equals `claimed_this_month` while a claim is pending
    /// and `claimed_this_month - 1` just after one, and a handful of rows carry
    /// a stale `0` across a series rollover. Prefer `claimed_this_month`.
    pub reward_index: i16,
    /// Whether a daily sign-in is claimable right now (as of the last sync).
    pub can_check_in: bool,
    /// Account creation time (Unix seconds), for "days since joining".
    #[ts(type = "number | null")]
    pub register_ts: Option<i64>,
    /// Player's last in-game online time (Unix seconds), distinct from the DB
    /// sync time below.
    #[ts(type = "number | null")]
    pub last_online_ts: Option<i64>,
    /// When this row was last synced to our DB. Every field above is a snapshot
    /// as of this moment, which may be days/weeks before "now", so the month
    /// the calendar belongs to is this timestamp's month, not today's.
    pub updated_at: DateTime<Utc>,
}

impl UserCheckin {
    /// The row as a visitor may see it under the owner's `layout`: the
    /// sign-in total and the registration date go when the header hides them,
    /// as `/get-user` nulls the same numbers. The month's slots stay: they are
    /// the Stats tab's own calendar, not a header number.
    pub fn hide_private(&mut self, layout: &ProfileLayout) {
        if !layout.header_shows(ProfileHeaderField::SignIn) {
            self.cumulative_signin = None;
        }
        if !layout.header_shows(ProfileHeaderField::Registered) {
            self.register_ts = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// A synced, public profile with every header number set.
    fn profile(layout: Option<Value>) -> UserProfile {
        UserProfile {
            id: Uuid::nil(),
            uid: "123456".into(),
            nickname: Some("Doctor".into()),
            nick_number: Some("1234".into()),
            level: Some(120),
            avatar_id: Some("avatar_def_01".into()),
            secretary: None,
            secretary_skin_id: None,
            resume_id: None,
            role: "user".into(),
            server: "EN".into(),
            total_score: Some(9_876.5),
            grade: Some("S".into()),
            public_profile: Some(true),
            store_gacha: Some(true),
            share_stats: Some(true),
            exp: Some(0),
            orundum: Some(54_321),
            lmd: Some(9_000_000),
            sanity: Some(100),
            max_sanity: Some(135),
            gacha_tickets: Some(12),
            ten_pull_tickets: Some(3),
            monthly_sub_end: Some(1_700_000_000),
            register_ts: Some(1_600_000_000),
            last_online_ts: Some(1_760_000_000),
            resume: Some("hi".into()),
            friend_num_limit: Some(50),
            cumulative_signin: Some(1_024),
            operator_count: Some(300),
            item_count: Some(250),
            skin_count: Some(400),
            non_default_skin_count: Some(120),
            updated_at: DateTime::<Utc>::from_timestamp(1_760_000_000, 0).unwrap(),
            originite: Some(777),
            profile_layout: layout.map(ProfileLayout::from),
        }
    }

    fn visitor(layout: Option<Value>) -> Value {
        let mut p = profile(layout);
        p.hide_private();
        serde_json::to_value(p).unwrap()
    }

    /// `hide_private_tabs` as it stood before header layouts, verbatim, to
    /// prove the kill switch against.
    fn before_headers(layout: Option<Value>) -> Value {
        let mut p = profile(layout);
        if let Some(layout) = &mut p.profile_layout {
            if !layout.is_visible(ProfileTabId::Score) {
                p.total_score = None;
                p.grade = None;
            }
            *layout = layout.visible_only();
        }
        serde_json::to_value(p).unwrap()
    }

    #[test]
    fn a_null_layout_reaches_a_visitor_byte_identical() {
        let owner = serde_json::to_string(&profile(None)).unwrap();
        let mut projected = profile(None);
        projected.hide_private();
        assert_eq!(serde_json::to_string(&projected).unwrap(), owner);
        assert_eq!(visitor(None), before_headers(None));
    }

    #[test]
    fn a_layout_without_a_header_projects_exactly_as_before() {
        for layout in [
            json!({ "tabs": [] }),
            json!({ "tabs": [{ "id": "score", "visible": false }, { "id": "roster", "visible": false }] }),
            json!({ "background": { "kind": "operator", "id": "char_003_kalts" } }),
            json!({ "header": null }),
            json!({ "header": "originite" }),
        ] {
            assert_eq!(
                serde_json::to_string(&visitor(Some(layout.clone()))).unwrap(),
                serde_json::to_string(&before_headers(Some(layout.clone()))).unwrap(),
                "{layout}"
            );
        }
    }

    #[test]
    fn a_hidden_field_is_null_for_a_visitor_and_its_card_leaves_the_strip() {
        let got = visitor(Some(json!({ "header": { "hidden": ["originite"] } })));
        let full = serde_json::to_value(profile(None)).unwrap();
        assert_eq!(got["originite"], Value::Null);
        assert_eq!(
            got["profile_layout"]["header"],
            json!({ "hidden": ["originite"], "strip": ["operators", "skins", "items", "orundum"] })
        );
        // Nothing else on the row moves.
        for (key, value) in full.as_object().unwrap() {
            if key != "originite" && key != "profile_layout" {
                assert_eq!(&got[key], value, "{key}");
            }
        }
    }

    #[test]
    fn every_hideable_field_nulls_its_columns() {
        let columns: [(&str, &[&str]); 11] = [
            ("level", &["level", "exp"]),
            ("registered", &["register_ts"]),
            ("sign_in", &["cumulative_signin"]),
            ("operators", &["operator_count"]),
            ("skins", &["skin_count", "non_default_skin_count"]),
            ("items", &["item_count"]),
            ("orundum", &["orundum"]),
            ("originite", &["originite"]),
            ("lmd", &["lmd"]),
            ("headhunting_permits", &["gacha_tickets"]),
            ("ten_pull_permits", &["ten_pull_tickets"]),
        ];
        assert_eq!(columns.len(), ProfileHeaderField::ALL.len());
        let full = serde_json::to_value(profile(None)).unwrap();
        for (id, nulled) in columns {
            let got = visitor(Some(json!({ "header": { "hidden": [id] } })));
            for (key, value) in full.as_object().unwrap() {
                if key == "profile_layout" {
                    continue;
                }
                if nulled.contains(&key.as_str()) {
                    assert_eq!(got[key], Value::Null, "{id}: {key}");
                } else {
                    assert_eq!(&got[key], value, "{id}: {key}");
                }
            }
        }
    }

    #[test]
    fn the_owner_row_is_untouched_by_a_header() {
        // `/get-user` projects only for a visitor; the owner's row is the
        // stored one, header and all.
        let owner = profile(Some(
            json!({ "header": { "hidden": ["originite", "lmd"], "strip": ["lmd"] } }),
        ));
        assert_eq!(owner.originite, Some(777));
        assert_eq!(owner.lmd, Some(9_000_000));
        let header = owner.profile_layout.and_then(|l| l.header).unwrap();
        assert_eq!(header.strip, Some(vec![ProfileHeaderField::Lmd]));
    }

    #[test]
    fn the_checkin_row_follows_the_header() {
        let checkin = || UserCheckin {
            monthly_card_flags: vec![1, 0],
            claimed_this_month: 2,
            cumulative_signin: Some(1_024),
            checkin_group_id: None,
            reward_index: 1,
            can_check_in: false,
            register_ts: Some(1_600_000_000),
            last_online_ts: Some(1_760_000_000),
            updated_at: DateTime::<Utc>::from_timestamp(1_760_000_000, 0).unwrap(),
        };
        let mut kept = checkin();
        kept.hide_private(&ProfileLayout::normalize(&json!({ "tabs": [] })));
        assert_eq!(
            serde_json::to_value(&kept).unwrap(),
            serde_json::to_value(checkin()).unwrap()
        );
        let mut hidden = checkin();
        hidden.hide_private(&ProfileLayout::normalize(
            &json!({ "header": { "hidden": ["sign_in", "registered"] } }),
        ));
        assert_eq!((hidden.cumulative_signin, hidden.register_ts), (None, None));
        assert_eq!(hidden.claimed_this_month, 2);
    }
}
