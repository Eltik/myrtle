import { SkillData } from "../../../../../types/impl/lib/impl/dps-calculator";

export default class Skills {
    public name: string;

    public skillData: SkillData;

    constructor(name: string, skillData: SkillData) {
        this.name = name;
        this.skillData = skillData;
    }
}
