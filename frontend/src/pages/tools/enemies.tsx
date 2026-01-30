import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { EnemiesDatabase } from "~/components/tools/enemies-database";
import { env } from "~/env";
import type { EnemiesResponse, Enemy, EnemyInfoList, LevelInfoResponse, RaceData, RacesResponse } from "~/types/api/impl/enemy";

// ============================================================================
// Props Interface
// ============================================================================

interface EnemiesPageProps {
    enemies: Enemy[];
    races: Record<string, RaceData>;
    levelInfo: EnemyInfoList[];
    total: number;
}

// ============================================================================
// Data Fetching Utilities
// ============================================================================

/**
 * Fetch all enemies with pagination support
 * @param backendURL - The backend API URL
 * @param options - Optional parameters for filtering/pagination
 */
async function fetchAllEnemies(
    backendURL: string,
    options?: {
        cursor?: string;
        limit?: number;
        fields?: string[];
        level?: number;
    },
): Promise<EnemiesResponse> {
    const queryParams = new URLSearchParams();

    if (options?.cursor) queryParams.set("cursor", options.cursor);
    if (options?.limit) queryParams.set("limit", options.limit.toString());
    if (options?.fields && options.fields.length > 0) queryParams.set("fields", options.fields.join(","));
    if (options?.level !== undefined) queryParams.set("level", options.level.toString());

    const queryString = queryParams.toString();
    const endpoint = queryString ? `/static/enemies?${queryString}` : "/static/enemies";

    const response = await fetch(`${backendURL}${endpoint}`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch enemies: ${response.status}`);
    }

    return response.json() as Promise<EnemiesResponse>;
}

/**
 * Fetch a single enemy by ID
 * @param backendURL - The backend API URL
 * @param enemyId - The enemy ID to fetch
 */
async function _fetchEnemyById(backendURL: string, enemyId: string): Promise<Enemy> {
    const response = await fetch(`${backendURL}/static/enemies/${enemyId}`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch enemy ${enemyId}: ${response.status}`);
    }

    const data = (await response.json()) as { enemy: Enemy };
    return data.enemy;
}

/**
 * Fetch all enemy races
 * @param backendURL - The backend API URL
 */
async function fetchEnemyRaces(backendURL: string): Promise<Record<string, RaceData>> {
    const response = await fetch(`${backendURL}/static/enemies/races`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch enemy races: ${response.status}`);
    }

    const data = (await response.json()) as RacesResponse;
    return data.races;
}

/**
 * Fetch enemy level info list
 * @param backendURL - The backend API URL
 */
async function fetchEnemyLevelInfo(backendURL: string): Promise<EnemyInfoList[]> {
    const response = await fetch(`${backendURL}/static/enemies/levels`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!response.ok) {
        throw new Error(`Failed to fetch enemy level info: ${response.status}`);
    }

    const data = (await response.json()) as LevelInfoResponse;
    return data.levels;
}

/**
 * Fetch all enemies with automatic pagination
 * Fetches all pages until hasMore is false
 * @param backendURL - The backend API URL
 * @param options - Optional parameters for filtering
 */
async function fetchAllEnemiesPaginated(
    backendURL: string,
    options?: {
        fields?: string[];
        level?: number;
        batchSize?: number;
    },
): Promise<{ enemies: Enemy[]; total: number }> {
    const batchSize = options?.batchSize ?? 500;
    const allEnemies: Enemy[] = [];
    let cursor: string | undefined;
    let total = 0;

    do {
        const response = await fetchAllEnemies(backendURL, {
            cursor,
            limit: batchSize,
            fields: options?.fields,
            level: options?.level,
        });

        allEnemies.push(...response.enemies);
        total = response.total;
        cursor = response.nextCursor ?? undefined;
    } while (cursor);

    return { enemies: allEnemies, total };
}

// ============================================================================
// Server-Side Data Fetching
// ============================================================================

export const getServerSideProps: GetServerSideProps<EnemiesPageProps> = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch all data in parallel
        const [enemiesData, races, levelInfo] = await Promise.all([
            // Fetch all enemies (paginated automatically)
            fetchAllEnemiesPaginated(backendURL, { batchSize: 500 }),
            // Fetch enemy races
            fetchEnemyRaces(backendURL),
            // Fetch level info
            fetchEnemyLevelInfo(backendURL),
        ]);

        return {
            props: {
                enemies: enemiesData.enemies,
                races,
                levelInfo,
                total: enemiesData.total,
            },
        };
    } catch (error) {
        console.error("Failed to fetch enemy data:", error);
        return { notFound: true };
    }
};

// ============================================================================
// Page Component
// ============================================================================

const EnemiesPage: NextPage<EnemiesPageProps> = ({ enemies, races, levelInfo, total }) => {
    return (
        <>
            <SEO description="Browse and search enemies from Arknights with detailed stats and abilities. Filter by enemy level, damage type, and more." path="/tools/enemies" title="Enemy Database" />
            <EnemiesDatabase enemies={enemies} levelInfo={levelInfo} races={races} total={total} />
        </>
    );
};

export default EnemiesPage;
