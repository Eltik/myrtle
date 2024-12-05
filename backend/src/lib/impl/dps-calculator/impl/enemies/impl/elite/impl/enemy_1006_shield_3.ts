import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1006_shield_3 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 1200, 0, null, null, null, null, null, null, null, null, null);
};
