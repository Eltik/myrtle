import EnemyUnit from "../classes/enemy-unit";
import fetchNormalEnemies from "./impl/normal";

export const ALL_ENEMIES: EnemyUnit[] = [];

export const init = async () => {
    await fetchNormalEnemies();
};
