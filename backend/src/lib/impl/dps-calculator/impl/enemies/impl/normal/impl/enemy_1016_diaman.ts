import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1016_diaman = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 500, 85, null, null, null, null, null, null, null, null, null);
};
