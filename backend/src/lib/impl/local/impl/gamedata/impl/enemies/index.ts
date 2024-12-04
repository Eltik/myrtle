import type { Enemy, EnemyHandbook } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): EnemyHandbook => {
    const data = STATIC_DATA?.ENEMY_HANDBOOK_TABLE as EnemyHandbook;
    return data;
};

export default (id: string): Enemy | null => {
    const enemyData = getAll().enemyData;
    return Object.values(enemyData).find((item) => item.enemyId === id) ?? null;
};
