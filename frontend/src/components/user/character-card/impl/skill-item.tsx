import Image from "next/image";
import type { CharacterData } from "~/types/api/impl/user";

interface SkillStatic {
    iconId?: string;
    skillId?: string;
    image?: string;
    name?: string;
    description?: string;
    duration?: number;
    hidden?: boolean;
    spData?: {
        increment?: number;
        initSp?: number;
        levelUpCost?: null;
        maxChargeTime?: number;
        spCost?: number;
        spType?: string;
    };
}

interface SkillItemProps {
    skill: CharacterData["skills"][0];
    index: number;
    isDefaultSkill: boolean;
    mainSkillLvl: number;
    size?: "small" | "large";
}

export function SkillItem({ skill, index, isDefaultSkill, mainSkillLvl, size = "small" }: SkillItemProps) {
    const skillStatic = skill.static as SkillStatic | null;
    const isSmall = size === "small";

    const iconSize = isSmall ? 28 : 40;
    const containerClass = isSmall ? "flex items-center gap-2 overflow-hidden rounded-md border p-2" : "flex items-center gap-3 rounded-lg border p-3";
    const imageClass = isSmall ? "h-7 w-7 shrink-0" : "h-10 w-10";
    const titleClass = isSmall ? "truncate font-medium text-sm" : "truncate font-medium";
    const masteryIconSize = isSmall ? 16 : 20;
    const masteryClass = isSmall ? "h-4 w-4" : "h-5 w-5";

    return (
        <div className={`${containerClass} ${isDefaultSkill ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`}>
            <Image alt="Skill" className={imageClass} height={iconSize} src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`} unoptimized width={iconSize} />
            <div className={isSmall ? "w-0 flex-1 overflow-hidden" : "min-w-0 flex-1"}>
                <div className={titleClass} title={skillStatic?.name ?? `Skill ${index + 1}`}>
                    {skillStatic?.name ?? `Skill ${index + 1}`}
                    {!isSmall && isDefaultSkill && <span className="ml-2 text-neutral-500 text-xs">(Equipped)</span>}
                </div>
                <div className={`flex items-center gap-${isSmall ? "1" : "2"} text-muted-foreground text-${isSmall ? "xs" : "sm"}`}>
                    <span>Lv.{mainSkillLvl}</span>
                    {skill.specializeLevel > 0 && (
                        <span className={`flex items-center gap-${isSmall ? "0.5" : "1"}`}>
                            <Image alt={`M${skill.specializeLevel}`} className={masteryClass} height={masteryIconSize} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={masteryIconSize} />M{skill.specializeLevel}
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
}
