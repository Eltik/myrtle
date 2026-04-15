import type { GetServerSideProps, NextPage } from "next";
import { EnemyDatabase } from "~/components/collection/enemies";
import { SEO } from "~/components/seo";
import { backendFetch } from "~/lib/backend-fetch";
import type { Enemy, EnemyHandbook, EnemyInfoList, RaceData } from "~/types/api/impl/enemy";

interface EnemiesPageProps {
    enemies: Enemy[];
    races: Record<string, RaceData>;
    levelInfo: EnemyInfoList[];
    total: number;
}

const EnemiesPage: NextPage<EnemiesPageProps> = ({ enemies, races, levelInfo, total }) => {
    return (
        <>
            <SEO description="Browse and search enemies from Arknights with detailed stats and abilities. Filter by enemy level, damage type, and more." path="/collection/enemies" title="Enemy Database" />
            <EnemyDatabase enemies={enemies} levelInfo={levelInfo} races={races} total={total} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<EnemiesPageProps> = async () => {
    try {
        // v3: /static/enemies returns all enemy data in one call
        // The response is the full enemy handbook structure or a Record<string, Enemy>
        const response = await backendFetch("/static/enemies");

        if (!response.ok) {
            console.error("Failed to fetch enemies:", response.status);
            return { notFound: true };
        }

        const data = (await response.json()) as EnemyHandbook;

        // Extract enemies, races, and level info from the handbook structure
        const enemies = Object.values(data.enemyData ?? data);
        const races = data.raceData ?? {};
        const levelInfo = data.levelInfoList ?? [];

        return {
            props: {
                enemies: enemies as Enemy[],
                races,
                levelInfo,
                total: enemies.length,
            },
        };
    } catch (error) {
        console.error("Failed to fetch enemy data:", error);
        return { notFound: true };
    }
};

export default EnemiesPage;
