//! One authorisation function: `Policy::can(subject, action, resource)`.

use std::sync::Arc;

use myrtle_types::{
    ApiKeyId, ProfileTab, Role, Server, TierListId, TierListType, UserId, WorkerId,
};

use crate::AppError;

#[derive(Debug, Clone)]
pub enum Subject {
    Anonymous,
    User {
        id: UserId,
        /// The game account uid
        uid: Arc<str>,
        /// `None` for a claim no server code matches exactly
        server: Option<Server>,
        role: Role,
        grants: Arc<[Grant]>,
    },
    /// Allowed exactly the actions its scopes name
    ApiKey {
        id: ApiKeyId,
        scopes: Arc<[String]>,
        owner: Option<UserId>,
    },
    Worker {
        id: WorkerId,
    },
    /// A job acting on its own
    System,
}

impl Subject {
    #[must_use]
    pub const fn user_id(&self) -> Option<UserId> {
        match self {
            Self::User { id, .. } => Some(*id),
            Self::ApiKey { owner, .. } => *owner,
            _ => None,
        }
    }

    /// The signed-in user's own game account uid
    #[must_use]
    pub fn uid(&self) -> Option<&str> {
        match self {
            Self::User { uid, .. } => Some(uid),
            _ => None,
        }
    }

    #[must_use]
    pub const fn server(&self) -> Option<Server> {
        match self {
            Self::User { server, .. } => *server,
            _ => None,
        }
    }

    /// The signed-in user's game server; a claim no server matches is 400, not 401
    pub fn game_server(&self) -> Result<Server, AppError> {
        match self {
            Self::User {
                server: Some(server),
                ..
            } => Ok(*server),
            Self::User { server: None, .. } => Err(AppError::bad_request("unknown server")),
            _ => Err(AppError::Unauthorized),
        }
    }

    #[must_use]
    pub const fn role(&self) -> Option<Role> {
        match self {
            Self::User { role, .. } => Some(*role),
            _ => None,
        }
    }
}

/// One `tier_list_permissions` or `translation_permissions` row
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub action: Action,
    /// Tier list id or locale code; `None` covers every one
    pub resource_id: Option<String>,
}

impl Grant {
    fn covers(&self, action: Action, resource_id: &str) -> bool {
        self.resource_id
            .as_deref()
            .is_none_or(|id| id == resource_id)
            && matches!(
                (self.action.ladder(), action.ladder()),
                (Some((held, rank)), Some((asked, required))) if held == asked && rank >= required
            )
    }
}

/// The level stored in a permission row's `permission` column
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Permission {
    View,
    Edit,
    Publish,
    Admin,
}

impl Permission {
    pub const ALL: [Self; 4] = [Self::View, Self::Edit, Self::Publish, Self::Admin];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Edit => "edit",
            Self::Publish => "publish",
            Self::Admin => "admin",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|p| p.as_str().eq_ignore_ascii_case(s))
    }

    #[must_use]
    pub fn grants(self, required: Self) -> bool {
        self >= required
    }

    #[must_use]
    pub const fn tier_list_action(self) -> Action {
        match self {
            Self::View => Action::ViewTierList,
            Self::Edit => Action::EditTierList,
            Self::Publish => Action::PublishTierList,
            Self::Admin => Action::AdministerTierList,
        }
    }

    /// A view grant writes nothing but still opens the admin panel
    #[must_use]
    pub const fn translation_action(self) -> Action {
        match self {
            Self::View => Action::AccessAdminPanel,
            Self::Edit | Self::Publish => Action::WriteTranslations,
            Self::Admin => Action::ManageTranslators,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    ReadProfile,
    ReadProfileTab,
    ViewTierList,
    EditTierList,
    PublishTierList,
    /// Delete and manage
    AdministerTierList,
    CreateTierList,
    ManageTierListGrants,
    ViewTierListAdminStats,
    AccessAdminPanel,
    WriteTranslations,
    ManageTranslators,
    ManageLocales,
    SetUserRole,
    ListUsers,
    ViewSiteStats,
    EditOperatorNotes,
    ReadOthersNoteAudit,
    EditReleaseData,
    EditGrid,
    BypassRateLimit,
    LeaseJobs,
    ManageApiKeys,
    ReadAudit,
}

impl Action {
    pub const ALL: [Self; 24] = [
        Self::ReadProfile,
        Self::ReadProfileTab,
        Self::ViewTierList,
        Self::EditTierList,
        Self::PublishTierList,
        Self::AdministerTierList,
        Self::CreateTierList,
        Self::ManageTierListGrants,
        Self::ViewTierListAdminStats,
        Self::AccessAdminPanel,
        Self::WriteTranslations,
        Self::ManageTranslators,
        Self::ManageLocales,
        Self::SetUserRole,
        Self::ListUsers,
        Self::ViewSiteStats,
        Self::EditOperatorNotes,
        Self::ReadOthersNoteAudit,
        Self::EditReleaseData,
        Self::EditGrid,
        Self::BypassRateLimit,
        Self::LeaseJobs,
        Self::ManageApiKeys,
        Self::ReadAudit,
    ];

    /// The API key scope that allows this action
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadProfile => "read_profile",
            Self::ReadProfileTab => "read_profile_tab",
            Self::ViewTierList => "view_tier_list",
            Self::EditTierList => "edit_tier_list",
            Self::PublishTierList => "publish_tier_list",
            Self::AdministerTierList => "administer_tier_list",
            Self::CreateTierList => "create_tier_list",
            Self::ManageTierListGrants => "manage_tier_list_grants",
            Self::ViewTierListAdminStats => "view_tier_list_admin_stats",
            Self::AccessAdminPanel => "access_admin_panel",
            Self::WriteTranslations => "write_translations",
            Self::ManageTranslators => "manage_translators",
            Self::ManageLocales => "manage_locales",
            Self::SetUserRole => "set_user_role",
            Self::ListUsers => "list_users",
            Self::ViewSiteStats => "view_site_stats",
            Self::EditOperatorNotes => "edit_operator_notes",
            Self::ReadOthersNoteAudit => "read_others_note_audit",
            Self::EditReleaseData => "edit_release_data",
            Self::EditGrid => "edit_grid",
            Self::BypassRateLimit => "bypass_rate_limit",
            Self::LeaseJobs => "lease_jobs",
            Self::ManageApiKeys => "manage_api_keys",
            Self::ReadAudit => "read_audit",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.as_str() == s)
    }

    /// (family, rank) for the actions a higher grant implies
    const fn ladder(self) -> Option<(u8, u8)> {
        match self {
            Self::ViewTierList => Some((0, 0)),
            Self::EditTierList => Some((0, 1)),
            Self::PublishTierList => Some((0, 2)),
            Self::AdministerTierList => Some((0, 3)),
            Self::WriteTranslations => Some((1, 0)),
            Self::ManageTranslators => Some((1, 1)),
            _ => None,
        }
    }
}

/// What is acted on, with the facts the decision needs
#[derive(Debug, Clone)]
pub enum Resource {
    Profile {
        owner: UserId,
        public: bool,
    },
    ProfileTab {
        owner: UserId,
        tab: ProfileTab,
        visible: bool,
    },
    TierList {
        id: TierListId,
        owner: Option<UserId>,
    },
    NewTierList {
        kind: TierListType,
    },
    Locale {
        code: String,
    },
    User {
        id: UserId,
    },
    Grid {
        owner: UserId,
    },
    OperatorNotes,
    ReleaseData,
    AdminPanel,
    /// Stats, the user list, the rate limiter
    Site,
    JobQueue,
    ApiKeys,
    Audit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny { reason: &'static str },
}

impl Decision {
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

const SIGN_IN_REQUIRED: &str = "sign in required";

#[derive(Debug, Clone, Copy)]
pub struct Policy {
    /// Re-read `users.role` per request instead of trusting the token's claim
    pub role_recheck: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self { role_recheck: true }
    }
}

impl Policy {
    #[must_use]
    pub fn can(&self, subject: &Subject, action: Action, resource: &Resource) -> Decision {
        let allowed = match subject {
            Subject::System => true,
            Subject::Worker { .. } => action == Action::LeaseJobs,
            Subject::ApiKey { scopes, .. } => scopes.iter().any(|s| s == action.as_str()),
            Subject::Anonymous => {
                let profile_read = matches!(action, Action::ReadProfile | Action::ReadProfileTab);
                if !profile_read {
                    return Decision::Deny {
                        reason: SIGN_IN_REQUIRED,
                    };
                }
                open_to_all(action, resource)
            }
            Subject::User {
                id, role, grants, ..
            } => open_to_all(action, resource) || user_can(*id, *role, grants, action, resource),
        };
        if allowed {
            Decision::Allow
        } else {
            Decision::Deny {
                reason: "not permitted",
            }
        }
    }

    /// [`Self::can`] for `?`: `Unauthorized` where signing in could help, else `Forbidden`
    pub fn require(
        &self,
        subject: &Subject,
        action: Action,
        resource: &Resource,
    ) -> Result<(), AppError> {
        match self.can(subject, action, resource) {
            Decision::Allow => Ok(()),
            Decision::Deny { reason } if reason == SIGN_IN_REQUIRED => Err(AppError::Unauthorized),
            Decision::Deny { .. } => Err(AppError::Forbidden),
        }
    }
}

const fn open_to_all(action: Action, resource: &Resource) -> bool {
    matches!(
        (action, resource),
        (Action::ReadProfile, Resource::Profile { public: true, .. })
            | (
                Action::ReadProfileTab,
                Resource::ProfileTab { visible: true, .. }
            )
    )
}

fn user_can(id: UserId, role: Role, grants: &[Grant], action: Action, resource: &Resource) -> bool {
    use Action as A;
    use Resource as R;

    let staff = matches!(
        role,
        Role::TierListEditor | Role::TierListAdmin | Role::SuperAdmin
    );
    let list_admin = matches!(role, Role::TierListAdmin | Role::SuperAdmin);
    let super_admin = role == Role::SuperAdmin;
    let granted = |resource_id: &str| grants.iter().any(|g| g.covers(action, resource_id));

    match (action, resource) {
        (A::ReadProfile, R::Profile { owner, .. })
        | (A::ReadProfileTab, R::ProfileTab { owner, .. }) => *owner == id,
        (
            A::ViewTierList | A::EditTierList | A::PublishTierList | A::AdministerTierList,
            R::TierList { id: list, owner },
        ) => list_admin || *owner == Some(id) || granted(&list.0.to_string()),
        (A::CreateTierList, R::NewTierList { kind }) => match kind {
            TierListType::Community => true,
            TierListType::Official => staff,
        },
        (A::EditGrid, R::Grid { owner }) => *owner == id || list_admin,
        (A::WriteTranslations | A::ManageTranslators, R::Locale { code }) => {
            super_admin || granted(code)
        }
        (A::AccessAdminPanel, _) => {
            role != Role::User
                || grants.iter().any(|g| {
                    g.action == A::AccessAdminPanel || g.action.ladder().is_some_and(|l| l.0 == 1)
                })
        }
        (A::EditOperatorNotes, _) => staff,
        (
            A::ManageTierListGrants
            | A::ViewTierListAdminStats
            | A::ListUsers
            | A::ViewSiteStats
            | A::ReadOthersNoteAudit
            | A::ReadAudit,
            _,
        ) => list_admin,
        (A::SetUserRole | A::ManageLocales | A::EditReleaseData | A::ManageApiKeys, _) => {
            super_admin
        }
        _ => false,
    }
}
