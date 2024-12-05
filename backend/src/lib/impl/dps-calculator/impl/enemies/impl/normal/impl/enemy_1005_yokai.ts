import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1005_yokai = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 50, 0, null, null, null, null, null, null, null, null, null);
};