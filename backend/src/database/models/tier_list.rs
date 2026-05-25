use serde::Serialize;
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierList {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub list_type: String,
    pub created_by: Option<Uuid>,
    pub is_active: bool,
    pub is_listed: bool,
    pub flair_id: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierListFlair {
    pub id: i16,
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierListStats {
    pub tier_list_id: Uuid,
    pub view_count: i64,
    pub unique_view_count: i64,
    pub favorite_count: i32,
    pub share_count: i32,
    pub is_trending: bool,
    pub trending_score: f64,
    pub views_last_24h: i32,
    pub views_last_7d: i32,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub stats_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierListFavorite {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub favorited_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Tier {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierPlacement {
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: i16,
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierListVersion {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version: i32,
    pub snapshot: serde_json::Value,
    pub changelog: Option<String>,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TierListPermission {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}
