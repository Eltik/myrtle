// Tier List types matching the backend Rust models

export interface TierList {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    created_by: string | null;
    created_at: string;
    updated_at: string;
}

export interface Tier {
    id: string;
    tier_list_id: string;
    name: string;
    display_order: number;
    color: string | null;
    description: string | null;
}

export interface TierPlacement {
    id: string;
    tier_id: string;
    operator_id: string;
    sub_order: number;
    notes: string | null;
    created_at: string;
    updated_at: string;
}

// Response types for the tier list API
export interface TierWithPlacements extends Tier {
    placements: TierPlacement[];
}

export interface TierListResponse {
    tier_list: TierList;
    tiers: TierWithPlacements[];
}
