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
        <div className={`grid items-center gap-2 rounded-md px-2.5 py-1.5 ${isDefaultSkill ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`} style={{ gridTemplateColumns: `${isSmall ? "24px" : "28px"} minmax(0, 1fr) auto` }}>
            <Image
                alt="Skill"
                className={isSmall ? "h-6 w-6 rounded" : "h-7 w-7 rounded-sm"}
                height={isSmall ? 24 : 28}
                src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`}
                unoptimized
                width={isSmall ? 24 : 28}
            />
            <span className="truncate font-medium text-xs" title={skillStatic?.name ?? `Skill ${index + 1}`}>
                {skillStatic?.name ?? `Skill ${index + 1}`}
            </span>
            <div className="flex items-center gap-1 text-muted-foreground text-xs">
                <span>Lv.{mainSkillLvl}</span>
                {skill.specializeLevel > 0 && <Image alt={`M${skill.specializeLevel}`} className="h-4 w-4" height={16} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={16} />}
            </div>
        </div>
    );
}
