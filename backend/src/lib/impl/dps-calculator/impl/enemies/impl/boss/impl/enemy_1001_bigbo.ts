import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1001_bigbo = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 800, 30, null, null, null, null, null, null, null, null, null);
};
