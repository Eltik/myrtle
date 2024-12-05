import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1025_reveng = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 200, 50, null, null, null, null, null, null, null, null, null);
};
