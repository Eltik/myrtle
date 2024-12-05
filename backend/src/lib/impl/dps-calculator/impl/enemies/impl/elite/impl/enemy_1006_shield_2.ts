import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1006_shield_2 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 1000, 0, null, null, null, null, null, null, null, null, null);
};
