import type { Skill } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/skills";
import { STATIC_DATA } from "../../../handler";

export const getAll = (): Skill[] => {
    const data = STATIC_DATA?.SKILL_TABLE as Record<string, Skill>;
    const skills = Object.entries(data).map(([id, data]) => ({
        id,
        ...data,
        image: `/spritepack/skill_icon_${id}.png`,
    }));
    return skills;
};

export default (id: string): Skill | null => {
    const skills = getAll();
    const skill = skills.find((skill) => skill.id === id) ?? null;
    return skill;
};
