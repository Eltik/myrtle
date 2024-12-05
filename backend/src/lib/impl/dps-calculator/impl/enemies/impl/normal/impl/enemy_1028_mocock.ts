import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1028_mocock = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 50, 0, null, null, null, null, null, null, null, null, null);
};
