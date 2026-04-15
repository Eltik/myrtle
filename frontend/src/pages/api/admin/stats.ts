import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { AdminStats, RecentUser, TierListSummary } from "~/types/frontend/impl/admin";

// Shape returned by backend `GET /admin/stats` (camelCase, flattened from
// `AdminStatsResponse` in backend::app::services::stats).
interface BackendAdminStatsResponse {
    users: {
        total: number;
        byServer: Record<string, number>;
        signups7d: number;
        signups30d: number;
        publicProfiles: number;
    };
    gacha: unknown;
    gameData: unknown;
    tierLists: {
        total: number;
        active: number;
        totalVersions: number;
        totalPlacements: number;
    };
    computedAt: string;
    usersByRole: {
        user: number;
        tierListEditor: number;
        tierListAdmin: number;
        superAdmin: number;
    };
    recentUsers: Array<{
        uid: string;
        serverId: number;
        nickname: string | null;
        level: number | null;
        createdAt: string;
    }>;
}

interface BackendTierListEntry {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    list_type: string | null;
    tier_list_type?: string | null;
    is_active: boolean;
    created_by: string | null;
    created_at: string;
    updated_at: string;
}

const SERVER_CODES: Record<number, string> = { 0: "en", 1: "jp", 2: "kr", 3: "cn", 4: "bili", 5: "tw" };

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        res.setHeader("Allow", ["GET"]);
        return res.status(405).json({ success: false, error: `Method ${req.method} not allowed` });
    }

    try {
        const token = getToken(req);
        if (!token) {
            return res.status(401).json({ success: false, error: "Not authenticated" });
        }

        // Fetch admin stats + full tier list index in parallel. The admin
        // stats endpoint only returns aggregate counts; the dashboard needs
        // the actual list of tier lists to render management controls.
        const [statsResponse, tierListsResponse] = await Promise.all([
            backendFetch("/admin/stats", {
                method: "GET",
                headers: { Authorization: `Bearer ${token}` },
            }),
            backendFetch("/tier-lists?type=all", { method: "GET" }),
        ]);

        if (!statsResponse.ok) {
            return res.status(statsResponse.status).json({ success: false, error: "Failed to fetch admin stats" });
        }

        const backend: BackendAdminStatsResponse = await statsResponse.json();

        const tierListEntries: BackendTierListEntry[] = tierListsResponse.ok
            ? ((await tierListsResponse.json()) as BackendTierListEntry[])
            : [];

        const tierListSummaries: TierListSummary[] = tierListEntries.map((t) => ({
            id: t.id,
            name: t.name,
            slug: t.slug,
            isActive: t.is_active,
            tierListType: (t.list_type ?? t.tier_list_type ?? "official") === "community" ? "community" : "official",
            createdBy: t.created_by,
            // The admin stats endpoint doesn't surface per-list tier/operator
            // counts; leave as 0 until the backend provides them.
            tierCount: 0,
            operatorCount: 0,
            versionCount: 0,
            createdAt: t.created_at,
            updatedAt: t.updated_at,
        }));

        const recentUsers: RecentUser[] = backend.recentUsers.map((u) => ({
            // Backend omits `id`/`role` on RecentUser rows; the admin panel
            // only displays uid/nickname/level/createdAt so empty strings
            // are an acceptable stub.
            id: u.uid,
            uid: u.uid,
            server: SERVER_CODES[u.serverId] ?? String(u.serverId),
            nickname: u.nickname ?? "",
            level: u.level ?? 0,
            role: "",
            createdAt: u.createdAt,
        }));

        const data: AdminStats = {
            users: {
                total: backend.users.total,
                byRole: {
                    user: backend.usersByRole.user,
                    tier_list_editor: backend.usersByRole.tierListEditor,
                    tier_list_admin: backend.usersByRole.tierListAdmin,
                    super_admin: backend.usersByRole.superAdmin,
                },
                byServer: backend.users.byServer,
                recentUsers,
            },
            tierLists: {
                total: backend.tierLists.total,
                active: backend.tierLists.active,
                totalVersions: backend.tierLists.totalVersions,
                totalPlacements: backend.tierLists.totalPlacements,
                tierLists: tierListSummaries,
            },
            // Audit-log feed isn't exposed by the v3 backend yet; send an
            // empty array so the Recent Activity card stays hidden.
            recentActivity: [],
        };

        return res.status(200).json({ success: true, data });
    } catch (error) {
        console.error("Admin stats handler error:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
