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
