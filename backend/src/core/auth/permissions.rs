use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    View,    // Can view the tier list
    Edit,    // Can modify placements
    Publish, // Can create new versions
    Admin,   // Can manage permissions + delete
}

impl Permission {
    const fn level(self) -> u8 {
        match self {
            Self::View => 0,
            Self::Edit => 1,
            Self::Publish => 2,
            Self::Admin => 3,
        }
    }
    pub const fn grants(self, required: Self) -> bool {
        self.level() >= required.level()
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::View,
            Self::Edit,
            Self::Publish,
            Self::Admin,
        ]
    }

    pub const fn includes(self) -> &'static [Self] {
        match self {
            Self::View => &[Self::View],
            Self::Edit => &[Self::View, Self::Edit],
            Self::Publish => &[Self::View, Self::Edit, Self::Publish],
            Self::Admin => &[
                Self::View,
                Self::Edit,
                Self::Publish,
                Self::Admin,
            ],
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::View => write!(f, "view"),
            Self::Edit => write!(f, "edit"),
            Self::Publish => write!(f, "publish"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "view" => Ok(Self::View),
            "edit" => Ok(Self::Edit),
            "publish" => Ok(Self::Publish),
            "admin" => Ok(Self::Admin),
            _ => Err(format!("Unknown permission: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GlobalRole {
    #[default]
    User, // Default - no special permissions
    TierListEditor, // Can edit tier lists they have permission for
    TierListAdmin,  // Can manage all tier lists
    SuperAdmin,     // Full access to everything
}

impl GlobalRole {
    pub const fn is_tier_list_admin(self) -> bool {
        matches!(self, Self::TierListAdmin | Self::SuperAdmin)
    }

    pub const fn is_super_admin(self) -> bool {
        matches!(self, Self::SuperAdmin)
    }

    pub const fn is_any_admin_role(self) -> bool {
        matches!(
            self,
            Self::TierListEditor | Self::TierListAdmin | Self::SuperAdmin
        )
    }

    pub const fn can_have_tier_permissions(self) -> bool {
        !matches!(self, Self::User)
    }

    pub const fn global_tier_permission(self) -> Option<Permission> {
        match self {
            Self::TierListAdmin | Self::SuperAdmin => Some(Permission::Admin),
            _ => None,
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::User,
            Self::TierListEditor,
            Self::TierListAdmin,
            Self::SuperAdmin,
        ]
    }
}

impl fmt::Display for GlobalRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "user",
            Self::TierListEditor => "tier_list_editor",
            Self::TierListAdmin => "tier_list_admin",
            Self::SuperAdmin => "super_admin",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuthContext {
    pub user_id: u128,
    pub role: GlobalRole,
}

impl AuthContext {
    pub const fn new(user_id: u128, role: GlobalRole) -> Self {
        Self { user_id, role }
    }

    pub const fn can_manage_all_tier_lists(self) -> bool {
        self.role.is_tier_list_admin()
    }

    pub const fn can_create_tier_list(self) -> bool {
        self.role.is_any_admin_role()
    }
}
