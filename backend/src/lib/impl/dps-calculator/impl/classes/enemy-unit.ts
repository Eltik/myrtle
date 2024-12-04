import Skills from "./skills";
import Stats from "./stats";

export default class EnemyUnit {
    public stats: Stats;
    public skills: Skills[];

    constructor(stats: Stats, skills: Skills[]) {
        this.stats = stats;
        this.skills = skills;
    }
}
