use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::tier_lists::{Tier, TierList, TierPlacement};

#[derive(Serialize)]
pub struct OperatorInTier {
    pub id: String,
    pub operator_id: String,
    pub sub_order: i32,
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_rarity: Option<String>,
}

#[derive(Serialize)]
pub struct TierWithOperators {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
    pub operators: Vec<OperatorInTier>,
}

#[derive(Serialize)]
pub struct TierListDetail {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub tiers: Vec<TierWithOperators>,
}

/// GET /tier-lists/{slug}
/// Get a tier list with all tiers and operators (public)
pub async fn get_tier_list(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<TierListDetail>, ApiError> {
    // Get tier list
    let tier_list = TierList::find_by_slug(&state.db, &slug)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Database error".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Tier list not found".into()))?;

    // Get tiers
    let tiers = Tier::find_by_tier_list(&state.db, tier_list.id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Failed to fetch tiers".into())
        })?;

    // Build response with operators for each tier
    let mut tiers_with_operators = Vec::new();

    for tier in tiers {
        let placements = TierPlacement::find_by_tier(&state.db, tier.id)
            .await
            .map_err(|e| {
                eprintln!("Database error: {e:?}");
                ApiError::Internal("Failed to fetch placements".into())
            })?;

        let operators: Vec<OperatorInTier> = placements
            .into_iter()
            .map(|p| {
                // Enrich with operator data from game_data
                let operator_data = state.game_data.operators.get(&p.operator_id);

                OperatorInTier {
                    id: p.id.to_string(),
                    operator_id: p.operator_id,
                    sub_order: p.sub_order,
                    notes: p.notes,
                    operator_name: operator_data.map(|o| o.name.clone()),
                    operator_rarity: operator_data.and_then(|o| {
                        serde_json::to_value(&o.rarity)
                            .ok()
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                    }),
                }
            })
            .collect();

        tiers_with_operators.push(TierWithOperators {
            id: tier.id.to_string(),
            name: tier.name,
            display_order: tier.display_order,
            color: tier.color,
            description: tier.description,
            operators,
        });
    }

    Ok(Json(TierListDetail {
        id: tier_list.id.to_string(),
        name: tier_list.name,
        slug: tier_list.slug,
        description: tier_list.description,
        is_active: tier_list.is_active,
        created_at: tier_list.created_at.to_rfc3339(),
        updated_at: tier_list.updated_at.to_rfc3339(),
        tiers: tiers_with_operators,
    }))
}
