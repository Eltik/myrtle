import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1022_dmage_2 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 200, 50, null, null, null, null, null, null, null, null, null);
};
