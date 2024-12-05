import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1019_jshoot = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 100, 20, null, null, null, null, null, null, null, null, null);
};
