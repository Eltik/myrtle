import type { Enemy } from "../../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Stats from "../../../../classes/stats";

export const enemy_1017_defdrn = (enemyData: Enemy) => {
    return new Stats(enemyData.name, 150, 20, null, null, null, null, null, null, null, null, null);
};
