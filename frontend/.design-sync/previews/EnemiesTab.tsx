import { EnemiesTab } from "frontend";

// The enemy handbook is fetched through a stubbed server function in the design
// bundle, so the honest previews here are the tab's three non-data states.
const ENCOUNTERED = {
    encounteredCount: 187,
    handbookTotal: 1204,
    enemies: [
        { enemyId: "enemy_1000_gopro", name: "Original Rock", enemyIndex: "A-1", sortId: 1 },
        { enemyId: "enemy_1007_slime", name: "Mudrock Fragment", enemyIndex: "B-2", sortId: 2 },
    ],
};

export const Loading = () => <EnemiesTab encountered={undefined} isLoading />;

export const ProfilePrivate = () => <EnemiesTab encountered={null} />;

export const HandbookUnavailable = () => <EnemiesTab encountered={ENCOUNTERED} />;
