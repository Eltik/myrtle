import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1008_ghost = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 120, 35, null, null, null, null, null, null, null, null, null);
};
