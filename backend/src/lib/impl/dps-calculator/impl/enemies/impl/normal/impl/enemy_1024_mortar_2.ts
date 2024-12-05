import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1024_mortar_2 = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 150, 0, null, null, null, null, null, null, null, null, null);
};
