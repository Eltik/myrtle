import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1018_aoemag = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 120, 50, null, null, null, null, null, null, null, null, null);
};
