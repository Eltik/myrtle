import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1010_demon = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 230, 50, null, null, null, null, null, null, null, null, null);
};
