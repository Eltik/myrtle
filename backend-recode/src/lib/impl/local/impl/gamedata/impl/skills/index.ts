import { RESOURCE_REPOSITORY } from "../..";
import type { Skill } from "../../../../../../../types/impl/lib/impl/local/impl/gamedata/impl/skills";
import { ExcelTables } from "../../../../../../../types/impl/lib/impl/local/impl/handler";
import { get as getSkills } from "../../../handler/impl/get";

export const getAll = async (): Promise<Skill[]> => {
    const data = (await getSkills(ExcelTables.SKILL_TABLE)) as Record<string, Skill>;
    const skills = Object.entries(data).map(([id, data]) => ({
        id,
        ...data,
        image: `https://raw.githubusercontent.com/${RESOURCE_REPOSITORY}/main/skill/skill_icon_${id}.png`,
    }));
    return skills;
};

export default async (id: string): Promise<Skill | null> => {
    const skills = await getAll();
    const skill = skills.find((skill) => skill.id === id) ?? null;
    return skill;
};
