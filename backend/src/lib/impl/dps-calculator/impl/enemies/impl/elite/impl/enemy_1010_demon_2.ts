import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1010_demon_2 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 250, 50, null, null, null, null, null, null, null, null, null);
};
