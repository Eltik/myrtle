import Image from "next/image";
import type { UserSkillStatic } from "~/types/api/impl/user";

interface SkillItemProps {
    skillId: string;
    specializeLevel: number;
    skillStatic?: UserSkillStatic | null;
    index: number;
    isDefaultSkill: boolean;
    mainSkillLvl: number;
    size?: "small" | "large";
}

export function SkillItem({ skillId, specializeLevel, skillStatic, index, isDefaultSkill, mainSkillLvl, size = "small" }: SkillItemProps) {
    const isSmall = size === "small";

    return (
        <div className={`grid items-center gap-2 rounded-md px-2.5 py-1.5 ${isDefaultSkill ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`} style={{ gridTemplateColumns: `${isSmall ? "24px" : "28px"} minmax(0, 1fr) auto` }}>
            <Image
                alt="Skill"
                className={isSmall ? "h-6 w-6 rounded" : "h-7 w-7 rounded-sm"}
                height={isSmall ? 24 : 28}
                loading="lazy"
                src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skillId}.png`}
                unoptimized
                width={isSmall ? 24 : 28}
            />
            <span className="truncate font-medium text-xs" title={skillStatic?.name ?? `Skill ${index + 1}`}>
                {skillStatic?.name ?? `Skill ${index + 1}`}
            </span>
            <div className="flex items-center gap-1 text-muted-foreground text-xs">
                <span>Lv.{mainSkillLvl}</span>
                {specializeLevel > 0 && <Image alt={`M${specializeLevel}`} className="h-4 w-4" height={16} loading="lazy" src={`/api/cdn/upk/arts/specialized_hub/specialized_${specializeLevel}.png`} unoptimized width={16} />}
            </div>
        </div>
    );
}
