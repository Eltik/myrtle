import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1001_bigbo_2 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 1000, 30, null, null, null, null, null, null, null, null, null);
};
