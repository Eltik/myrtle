use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Permission levels for tier list operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    View,    // Can view the tier list
    Edit,    // Can modify placements
    Publish, // Can create new versions
    Admin,   // Can manage permissions + delete
}

impl Permission {
    /// Returns all permissions that this permission level grants (hierarchical)
    pub fn includes(&self) -> Vec<Permission> {
        match self {
            Permission::View => vec![Permission::View],
            Permission::Edit => vec![Permission::View, Permission::Edit],
            Permission::Publish => vec![Permission::View, Permission::Edit, Permission::Publish],
            Permission::Admin => vec![
                Permission::View,
                Permission::Edit,
                Permission::Publish,
                Permission::Admin,
            ],
        }
    }

    /// Check if this permission grants the required permission
    pub fn grants(&self, required: Permission) -> bool {
        self.includes().contains(&required)
    }

    /// All available permission levels
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::View,
            Permission::Edit,
            Permission::Publish,
            Permission::Admin,
        ]
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::View => write!(f, "view"),
            Permission::Edit => write!(f, "edit"),
            Permission::Publish => write!(f, "publish"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "view" => Ok(Permission::View),
            "edit" => Ok(Permission::Edit),
            "publish" => Ok(Permission::Publish),
            "admin" => Ok(Permission::Admin),
            _ => Err(format!("Unknown permission: {s}")),
        }
    }
}

/// Global user roles (stored in users.role)
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
    /// Check if this role has global tier list admin access
    pub fn is_tier_list_admin(&self) -> bool {
        matches!(self, GlobalRole::TierListAdmin | GlobalRole::SuperAdmin)
    }

    /// Check if this role has super admin access
    pub fn is_super_admin(&self) -> bool {
        matches!(self, GlobalRole::SuperAdmin)
    }

    /// Check if this role can be granted tier list permissions
    pub fn can_have_tier_permissions(&self) -> bool {
        !matches!(self, GlobalRole::User)
    }

    /// Get the minimum permission this role grants on all tier lists
    pub fn global_tier_permission(&self) -> Option<Permission> {
        match self {
            GlobalRole::User => None,
            GlobalRole::TierListEditor => None, // Needs explicit per-list permission
            GlobalRole::TierListAdmin => Some(Permission::Admin),
            GlobalRole::SuperAdmin => Some(Permission::Admin),
        }
    }
}

impl fmt::Display for GlobalRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlobalRole::User => write!(f, "user"),
            GlobalRole::TierListEditor => write!(f, "tier_list_editor"),
            GlobalRole::TierListAdmin => write!(f, "tier_list_admin"),
            GlobalRole::SuperAdmin => write!(f, "super_admin"),
        }
    }
}

impl FromStr for GlobalRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(GlobalRole::User),
            "tier_list_editor" => Ok(GlobalRole::TierListEditor),
            "tier_list_admin" => Ok(GlobalRole::TierListAdmin),
            "super_admin" => Ok(GlobalRole::SuperAdmin),
            _ => Err(format!("Unknown role: {s}")),
        }
    }
}

/// Authorization context for tier list operations
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub role: GlobalRole,
}

impl AuthContext {
    pub fn new(user_id: uuid::Uuid, role: GlobalRole) -> Self {
        Self { user_id, role }
    }

    /// Check if user can perform action on any tier list (global check)
    pub fn can_manage_all_tier_lists(&self) -> bool {
        self.role.is_tier_list_admin()
    }

    /// Check if user can create new tier lists
    pub fn can_create_tier_list(&self) -> bool {
        matches!(
            self.role,
            GlobalRole::TierListAdmin | GlobalRole::SuperAdmin
        )
    }
}
