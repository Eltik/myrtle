//! What a visitor may see of someone else's profile, the owner's privacy settings, and what leaves the system.

pub mod layout;

use myrtle_db::DbError;
use myrtle_db::repos::users::{self, LayoutUpdate, ProfileRow, SettingsUpdate};
use myrtle_db::tools::{ImportGuard, Scope};
use myrtle_types::{ProfileTab, UserId};
use serde::Deserialize;

use self::layout::{ProfileLayout, ProfileLayoutPatch, ProfileTabId};
use crate::AppContext;
use crate::AppError;
use crate::policy::{Action, Resource, Subject};

/// Denied is 403 for every visitor, signed in or not
fn allow(
    ctx: &AppContext,
    subject: &Subject,
    action: Action,
    resource: &Resource,
) -> Result<(), AppError> {
    if ctx.policy.can(subject, action, resource).is_allowed() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn is_own(subject: &Subject, profile: &ProfileRow) -> bool {
    subject.user_id() == Some(UserId(profile.id))
}

fn stored_layout(profile: &ProfileRow) -> Option<ProfileLayout> {
    profile
        .profile_layout
        .as_ref()
        .map(ProfileLayout::normalize)
}

const fn policy_tab(tab: ProfileTabId) -> ProfileTab {
    match tab {
        ProfileTabId::Showcase => ProfileTab::Showcase,
        ProfileTabId::Stats => ProfileTab::Stats,
        ProfileTabId::Score => ProfileTab::Score,
        ProfileTabId::Roster => ProfileTab::Roster,
        ProfileTabId::Plans => ProfileTab::Plans,
        ProfileTabId::Inventory => ProfileTab::Inventory,
        ProfileTabId::Enemies => ProfileTab::Enemies,
        ProfileTabId::Optimizer => ProfileTab::Optimizer,
        ProfileTabId::Lookback => ProfileTab::Lookback,
    }
}

/// The profile behind `uid` and whether the subject owns it; gate before any cache read
pub async fn resolve_public_profile(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<(ProfileRow, bool), AppError> {
    let mut conn = ctx.db.conn().await?;
    let profile = users::profile_by_uid(&mut conn, uid, None)
        .await?
        .ok_or(AppError::NotFound(None))?;
    drop(conn);
    ensure_profile_readable(ctx, subject, profile)
}

pub fn ensure_profile_readable(
    ctx: &AppContext,
    subject: &Subject,
    profile: ProfileRow,
) -> Result<(ProfileRow, bool), AppError> {
    let resource = Resource::Profile {
        owner: UserId(profile.id),
        public: profile.public_profile == Some(true),
    };
    allow(ctx, subject, Action::ReadProfile, &resource)?;
    let own = is_own(subject, &profile);
    Ok((profile, own))
}

/// A profile that never set a layout hides no tab
pub fn ensure_tab_visible(
    ctx: &AppContext,
    subject: &Subject,
    profile: &ProfileRow,
    layout: Option<&ProfileLayout>,
    tab: ProfileTabId,
) -> Result<(), AppError> {
    let resource = Resource::ProfileTab {
        owner: UserId(profile.id),
        tab: policy_tab(tab),
        visible: layout.is_none_or(|l| l.is_visible(tab)),
    };
    allow(ctx, subject, Action::ReadProfileTab, &resource)
}

/// [`resolve_public_profile`] with every one of `tabs` visible
pub async fn resolve_profile_for_tabs(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
    tabs: &[ProfileTabId],
) -> Result<(ProfileRow, bool, Option<ProfileLayout>), AppError> {
    let (profile, own) = resolve_public_profile(ctx, subject, uid).await?;
    let layout = stored_layout(&profile);
    for &tab in tabs {
        ensure_tab_visible(ctx, subject, &profile, layout.as_ref(), tab)?;
    }
    Ok((profile, own, layout))
}

/// The `uid` param's user (readable, every one of `tabs` visible), else the subject's own
pub async fn resolve_user_id(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    tabs: &[ProfileTabId],
) -> Result<UserId, AppError> {
    match uid {
        Some(uid) => Ok(UserId(
            resolve_profile_for_tabs(ctx, subject, uid, tabs)
                .await?
                .0
                .id,
        )),
        None => subject.user_id().ok_or(AppError::Unauthorized),
    }
}

/// [`resolve_user_id`] for the endpoints keyed on the game-account uid; the own case reads the row, 401 once it is gone
pub async fn resolve_uid(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    tab: ProfileTabId,
) -> Result<String, AppError> {
    if let Some(uid) = uid {
        return Ok(resolve_profile_for_tabs(ctx, subject, uid, &[tab])
            .await?
            .0
            .uid);
    }
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    users::account(&mut *ctx.db.conn().await?, user)
        .await?
        .map(|account| account.uid)
        .ok_or(AppError::Unauthorized)
}

/// [`resolve_user_id`] with the layout limiting what a visitor reads; `None` for the owner
pub async fn resolve_user_id_as_viewer(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    tab: ProfileTabId,
) -> Result<(UserId, Option<ProfileLayout>), AppError> {
    match uid {
        Some(uid) => {
            let (profile, own, layout) =
                resolve_profile_for_tabs(ctx, subject, uid, &[tab]).await?;
            Ok((UserId(profile.id), layout.filter(|_| !own)))
        }
        None => Ok((subject.user_id().ok_or(AppError::Unauthorized)?, None)),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub public_profile: Option<bool>,
    pub store_gacha: Option<bool>,
    pub share_stats: Option<bool>,
    pub public_gacha: Option<bool>,
    /// Absent keeps the layout, `null` resets it, an object sets the keys it carries
    #[serde(default, deserialize_with = "present")]
    pub profile_layout: Option<Option<ProfileLayoutPatch>>,
}

#[allow(clippy::option_option)]
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

fn user_key(uid: &str) -> String {
    format!("user:{uid}")
}

/// The caller's own settings; the showcase and background referents must be validated first
pub async fn update_settings(
    ctx: &AppContext,
    subject: &Subject,
    request: &UpdateSettingsRequest,
) -> Result<(), AppError> {
    let (Some(user), Some(uid)) = (subject.user_id(), subject.uid()) else {
        return Err(AppError::Unauthorized);
    };
    let patch = request.profile_layout.as_ref().map(|p| {
        p.as_ref().map(|p| {
            (
                p.merge_json(),
                p.removed_keys(),
                p.tabs.is_some() || p.header.is_some(),
            )
        })
    });
    let layout = match &patch {
        None => LayoutUpdate::Keep,
        Some(None) => LayoutUpdate::Clear,
        Some(Some((merge, removed, _))) => LayoutUpdate::Merge {
            patch: merge,
            removed,
        },
    };
    let update = SettingsUpdate {
        public_profile: request.public_profile,
        store_gacha: request.store_gacha,
        share_stats: request.share_stats,
        public_gacha: request.public_gacha,
        layout,
    };
    let mut conn = ctx.db.conn().await?;
    match users::update_settings(&mut conn, user, &update).await {
        Ok(()) | Err(DbError::NotFound) => {} // no settings row yet: nothing to change
        Err(e) => return Err(e.into()),
    }
    drop(conn);
    ctx.cache.invalidate(&user_key(uid)).await;
    let ranking_changed = match &patch {
        None => false,
        Some(None) => true,
        Some(Some((_, _, ranked))) => *ranked,
    };
    if ranking_changed {
        for prefix in ["search:", "leaderboard:", "music:stats:listeners:"] {
            ctx.cache.invalidate_prefix(prefix).await; // search and the leaderboards hide by tab or header number
        }
    }
    Ok(())
}

/// An identifier a support report can still be matched against; four characters or fewer are blanked
#[must_use]
pub fn mask_identifier(value: &str) -> String {
    let count = value.chars().count();
    match count {
        0 => "<empty>".to_owned(),
        1..=4 => "*".repeat(count),
        _ => {
            let first = value.chars().next().unwrap_or('*');
            let last = value.chars().last().unwrap_or('*');
            format!("{first}{}{last}", "*".repeat(count - 2))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserTable {
    pub name: &'static str,
    pub scope: Scope,
    /// Surrogate keys the target database reassigns
    pub regenerate: &'static [&'static str],
    pub guard: Option<ImportGuard>,
}

impl UserTable {
    /// The column rewritten to the target user on import
    #[must_use]
    pub const fn owner_column(&self) -> Option<&'static str> {
        match self.scope {
            Scope::Owner => Some("user_id"),
            Scope::PlanGroup => None,
        }
    }
}

const fn owned(name: &'static str) -> UserTable {
    UserTable {
        name,
        scope: Scope::Owner,
        regenerate: &[],
        guard: None,
    }
}

/// Parents first; import deletes in reverse. Authored content, grants, telemetry and aggregates stay out
pub const USER_TABLES: &[UserTable] = &[
    owned("user_status"),
    owned("user_settings"),
    owned("user_operators"),
    owned("user_operator_skills"),
    owned("user_operator_modules"),
    owned("user_items"),
    owned("user_skins"),
    owned("user_stage_progress"),
    owned("user_roguelike_progress"),
    owned("user_sandbox_progress"),
    owned("user_enemy_progress"),
    owned("user_medals"),
    owned("user_building"),
    owned("user_checkin"),
    owned("user_scores"),
    owned("user_support_units"),
    owned("user_lookback_snapshots"),
    UserTable {
        regenerate: &["id"],
        ..owned("gacha_records")
    },
    owned("operator_plans"),
    owned("plan_groups"),
    owned("plan_presets"),
    UserTable {
        name: "plan_group_members",
        scope: Scope::PlanGroup,
        regenerate: &[],
        guard: None,
    },
    UserTable {
        guard: Some(ImportGuard::MissingTierList),
        ..owned("tier_list_favorites")
    },
];

/// Sealed with `user_id` as AAD: portable only to the same key and the same `users.id`
pub const CREDENTIALS_TABLE: UserTable = owned("user_game_credentials");

pub const EXPORT_KIND: &str = "myrtle-user-export";

pub const FORMAT_VERSION: u32 = 1;
