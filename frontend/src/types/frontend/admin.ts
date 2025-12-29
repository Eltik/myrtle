export type AdminRole = "super_admin" | "tier_list_admin";

// User statistics for admin dashboard
export interface UserStats {
    total: number;
    byRole: {
        user: number;
        tier_list_editor: number;
        tier_list_admin: number;
        super_admin: number;
    };
    byServer: Record<string, number>;
    recentUsers: RecentUser[];
}

export interface RecentUser {
    id: string;
    uid: string;
    server: string;
    nickname: string;
    level: number;
    role: string;
    createdAt: string;
}

// Tier list statistics
export interface TierListStats {
    total: number;
    active: number;
    totalVersions: number;
    totalPlacements: number;
    tierLists: TierListSummary[];
}

export interface TierListSummary {
    id: string;
    name: string;
    slug: string;
    isActive: boolean;
    tierCount: number;
    operatorCount: number;
    versionCount: number;
    createdAt: string;
    updatedAt: string;
}

// Recent activity/audit log
export interface RecentActivity {
    id: string;
    tierListId: string;
    tierListName: string;
    changeType: string;
    operatorId: string | null;
    operatorName: string | null;
    changedBy: string | null;
    changedByNickname: string | null;
    changedAt: string;
    reason: string | null;
}

// Complete admin stats response
export interface AdminStats {
    users: UserStats;
    tierLists: TierListStats;
    recentActivity: RecentActivity[];
}

// Tier list management types for admin panel
export interface TierListManagementData {
    tierList: TierListSummary;
    tiers: TierManagementData[];
}

export interface TierManagementData {
    id: string;
    name: string;
    displayOrder: number;
    color: string | null;
    description: string | null;
    placements: PlacementManagementData[];
}

export interface PlacementManagementData {
    id: string;
    operatorId: string;
    operatorName: string;
    operatorRarity: number;
    subOrder: number;
    notes: string | null;
}

export interface CreateTierListPayload {
    name: string;
    slug: string;
    description?: string;
    isActive?: boolean;
}

export interface UpdateTierListPayload {
    name?: string;
    description?: string;
    isActive?: boolean;
}

export interface CreateTierPayload {
    name: string;
    displayOrder: number;
    color?: string;
    description?: string;
}

export interface UpdateTierPayload {
    name?: string;
    displayOrder?: number;
    color?: string;
    description?: string;
}

export interface CreatePlacementPayload {
    tierId: string;
    operatorId: string;
    subOrder?: number;
    notes?: string;
}

export interface MovePlacementPayload {
    newTierId: string;
    newSubOrder?: number;
}
