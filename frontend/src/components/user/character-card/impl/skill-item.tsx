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

    return (
        <div className={`flex items-center gap-2 rounded-md px-2.5 py-1.5 ${isDefaultSkill ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`}>
            <Image
                alt="Skill"
                className={isSmall ? "h-6 w-6 shrink-0 rounded" : "h-7 w-7 shrink-0 rounded-sm"}
                height={isSmall ? 24 : 28}
                src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`}
                unoptimized
                width={isSmall ? 24 : 28}
            />
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className="truncate font-medium text-sm" title={skillStatic?.name ?? `Skill ${index + 1}`}>
                        {skillStatic?.name ?? `Skill ${index + 1}`}
                    </span>
                    {isDefaultSkill && <span className="shrink-0 text-[10px] text-muted-foreground">(Equipped)</span>}
                </div>
            </div>
            <div className="flex shrink-0 items-center gap-1.5 text-muted-foreground text-xs">
                <span>Lv.{mainSkillLvl}</span>
                {skill.specializeLevel > 0 && (
                    <span className="flex items-center gap-0.5">
                        <Image alt={`M${skill.specializeLevel}`} className="h-4 w-4" height={16} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={16} />
                        <span>M{skill.specializeLevel}</span>
                    </span>
                )}
            </div>
        </div>
    );
}
