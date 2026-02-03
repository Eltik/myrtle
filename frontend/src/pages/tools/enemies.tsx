import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { EnemyDatabase } from "~/components/tools/enemy-database";
import { env } from "~/env";
import type { EnemiesResponse, Enemy, EnemyInfoList, LevelInfoResponse, RaceData, RacesResponse } from "~/types/api/impl/enemy";

interface EnemiesPageProps {
    enemies: Enemy[];
    races: Record<string, RaceData>;
    levelInfo: EnemyInfoList[];
    total: number;
}

// Raw API response type (handles both snake_case and camelCase)
interface RawEnemiesResponse {
    enemies: Enemy[];
    hasMore?: boolean;
    has_more?: boolean;
    nextCursor?: string | null;
    next_cursor?: string | null;
    total: number;
}

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

    const raw = (await response.json()) as RawEnemiesResponse;

    // Normalize response to handle both snake_case and camelCase
    return {
        enemies: raw.enemies,
        hasMore: raw.hasMore ?? raw.has_more ?? false,
        nextCursor: raw.nextCursor ?? raw.next_cursor ?? null,
        total: raw.total,
    };
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
 * Fetch all enemies with cursor-based pagination
 * Uses large batch size for efficiency and continues until all enemies are fetched
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
    // Use large batch size to minimize number of requests
    const batchSize = options?.batchSize ?? 1000;
    const allEnemies: Enemy[] = [];
    let cursor: string | undefined;
    let total = 0;

    // Keep fetching until no more enemies are returned
    while (true) {
        const response = await fetchAllEnemies(backendURL, {
            cursor,
            limit: batchSize,
            fields: options?.fields,
            level: options?.level,
        });

        allEnemies.push(...response.enemies);
        total = response.total;
        cursor = response.nextCursor ?? undefined;

        // Stop if hasMore is false, no cursor, or no enemies returned
        if (!response.hasMore || !cursor || response.enemies.length === 0) {
            break;
        }
    }
    return { enemies: allEnemies, total };
}

const EnemiesPage: NextPage<EnemiesPageProps> = ({ enemies, races, levelInfo, total }) => {
    return (
        <>
            <SEO description="Browse and search enemies from Arknights with detailed stats and abilities. Filter by enemy level, damage type, and more." path="/tools/enemies" title="Enemy Database" />
            <EnemyDatabase enemies={enemies} levelInfo={levelInfo} races={races} total={total} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<EnemiesPageProps> = async () => {
    const backendURL = env.BACKEND_URL;

    try {
        // Fetch all data in parallel
        const [enemiesData, races, levelInfo] = await Promise.all([
            // Fetch all enemies with large batch size to minimize requests
            fetchAllEnemiesPaginated(backendURL, { batchSize: 1000 }),
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

export default EnemiesPage;
