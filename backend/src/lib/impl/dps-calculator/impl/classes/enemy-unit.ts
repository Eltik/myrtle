import { Enemy } from "../../../../../types/impl/lib/impl/local/impl/gamedata/impl/enemies";
import Skills from "./skills";
import Stats from "./stats";

export default class EnemyUnit {
    public stats: Stats;
    public enemy: Enemy;
    public skills: Skills[];

    constructor(enemy: Enemy, stats: Stats, skills: Skills[]) {
        this.enemy = enemy;
        this.stats = stats;
        this.skills = skills;
    }
}
