import { useEffect, useRef, useState } from "react";
import { eliteIcon, potentialIcon } from "#/components/operators/detail/impl/assets";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "#/components/ui/accordion";
import { Card, CardContent } from "#/components/ui/card";
import { Progress } from "#/components/ui/progress";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Separator } from "#/components/ui/separator";
import { formatProfession } from "#/lib/utils";
import type { IEnrichedSkill, IModule } from "#/types/operators";
import { getAttributeStats, getTrustPercent, isMaxed, MAX_LEVEL_BY_RARITY, moduleIconURL, ownedHeroURL, rarityIcon, skillIconURL, specializedIcon } from "./helpers.card";
import type { IOwnedEntry } from "./types";

interface IDetailedCardProps {
    entry: IOwnedEntry;
    lastRef?: ((node: HTMLElement | null) => void) | null;
}

export function DetailedCard({ entry, lastRef }: IDetailedCardProps) {
    const op = entry.static;
    const star = entry.rarity;
    const rarityColor = RARITY_COLORS[star] ?? "#ffffff";
    const maxed = isMaxed(entry);
    const trustPct = getTrustPercent(entry.favor_point);

    const maxLvlForPhase = op?.phases?.[entry.elite]?.maxLevel ?? MAX_LEVEL_BY_RARITY[star]?.[entry.elite] ?? 90;

    const stats = op ? getAttributeStats(entry, op) : null;

    const [hovered, setHovered] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const cardRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const el = cardRef.current;
        if (!el) return;
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0]?.isIntersecting) {
                    setTimeout(() => {
                        setLevelProgress((entry.level / maxLvlForPhase) * 100);
                        setTrustProgress((trustPct / 200) * 100);
                    }, 300);
                    observer.disconnect();
                }
            },
            { threshold: 0.2 },
        );
        observer.observe(el);
        return () => observer.disconnect();
    }, [entry.level, maxLvlForPhase, trustPct]);

    const skills = (op?.skills ?? []).map((skill, i) => ({
        skill,
        index: i,
        mastery: entry.masteries.find((m) => m.index === i)?.mastery ?? 0,
    }));

    const modules = (op?.modules ?? [])
        .filter((m) => m.typeName1 !== "ORIGINAL")
        .map((module) => {
            const rosterMod = entry.modules.find((rm) => rm.id === module.uniEquipId);
            return {
                module,
                level: rosterMod?.level ?? 0,
                locked: rosterMod?.locked ?? true,
                isEquipped: entry.current_equip === module.uniEquipId,
            };
        })
        .filter((m) => !m.locked && m.level > 0);

    return (
        <div ref={lastRef ?? undefined}>
            <Card
                ref={cardRef}
                className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col gap-0 overflow-hidden border-2 border-muted/30 py-0 pb-1 transition-all
  duration-300 hover:border-muted hover:shadow-lg"
                onMouseEnter={() => setHovered(true)}
                onMouseLeave={() => setHovered(false)}
                style={maxed ? { boxShadow: `0 0 20px ${rarityColor}60, 0 0 40px ${rarityColor}40` } : undefined}
            >
                <div className="relative">
                    <button type="button" className="block w-full text-left">
                        <div className="relative h-64 w-full cursor-pointer overflow-hidden">
                            <img alt={entry.name} src={ownedHeroURL(entry)} className={`h-full w-full object-contain object-top transition-transform duration-300 ${hovered ? "scale-105" : "scale-100"}`} decoding="async" loading="lazy" />
                            <div className={`absolute inset-0 bg-linear-to-t from-black/50 to-transparent transition-opacity duration-300 ${hovered ? "opacity-90" : "opacity-70"}`} />
                            <div className="absolute right-0 bottom-0 left-0 p-4">
                                <h3 className={`mt-2 max-w-3/4 text-left font-bold text-white text-xl transition-all duration-300 ${hovered ? "translate-y-0" : "translate-y-1"}`}>{entry.name}</h3>
                                <div className={`flex items-center justify-between transition-all duration-300 ${hovered ? "translate-y-0" : "translate-y-1"}`}>
                                    <div className="flex items-center gap-2">
                                        <img alt={`${star} Star`} className="h-4.5 w-auto object-contain" decoding="async" height={18} loading="lazy" src={rarityIcon(star)} width={60} />
                                        {op && (
                                            <div className="flex flex-row items-center gap-1">
                                                <ClassIcon profession={op.profession} size={20} />
                                                <span className="text-sm text-white">{formatProfession(op.profession)}</span>
                                            </div>
                                        )}
                                    </div>
                                    <img alt={`Elite ${entry.elite}`} className="h-6 w-6 object-contain" decoding="async" height={24} loading="lazy" src={eliteIcon(entry.elite)} width={24} />
                                </div>
                            </div>
                        </div>
                    </button>
                    {maxed && (
                        <div className="absolute top-2 z-10 rounded-r-md px-2 py-0.5 text-center font-semibold text-xs shadow-md" style={{ color: rarityColor }}>
                            Maxed
                        </div>
                    )}
                </div>
                <CardContent className="min-w-0 flex-1 overflow-hidden px-4 pt-2 pb-2">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <div className="flex items-center justify-between">
                                <span className="font-medium text-sm">Level</span>
                                <span className="font-bold text-sm">{entry.level}</span>
                            </div>
                            <Progress className="h-1.5 transition-all duration-1000 ease-out" value={levelProgress} />
                        </div>
                        <div>
                            <div className="flex items-center justify-between">
                                <span className="font-medium text-sm">Trust</span>
                                <span className="font-bold text-sm">{trustPct}%</span>
                            </div>
                            <Progress className="h-1.5 transition-all duration-1000 ease-out" value={trustProgress} />
                        </div>
                    </div>
                    <Separator className="my-3" />
                    <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                        <Stat label="HP" value={stats?.maxHp} />
                        <Stat label="ATK" value={stats?.atk} />
                        <Stat label="DEF" value={stats?.def} />
                        <Stat label="RES" value={stats?.magicResistance} />
                        <Stat label="Cost" value={stats?.cost} />
                        <Stat label="Block" value={stats?.blockCnt} />
                    </div>
                    <Separator className="my-3" />
                    <Accordion className="w-full" multiple>
                        <AccordionItem className="border-b-0" value="potential">
                            <AccordionTrigger className="py-2 font-medium text-sm hover:underline">Potential</AccordionTrigger>
                            <AccordionContent>
                                <div className="flex items-center justify-between py-1">
                                    <span className="text-sm">Current Potential</span>
                                    <div className="flex items-center gap-1">
                                        <img alt={`Potential ${entry.potential + 1}`} className="h-6 w-6" decoding="async" height={24} loading="lazy" src={potentialIcon(entry.potential)} width={24} />
                                        <span className="text-muted-foreground text-sm">+{entry.potential}</span>
                                    </div>
                                </div>
                            </AccordionContent>
                        </AccordionItem>
                        <AccordionItem className="border-b-0" value="skills">
                            <AccordionTrigger className="py-2 font-medium text-sm hover:underline">Skills</AccordionTrigger>
                            <AccordionContent>
                                <ScrollArea className="max-h-45 w-full">
                                    {skills.length > 0 ? (
                                        <div className="w-full space-y-2 overflow-hidden">
                                            {skills.map(({ skill, index, mastery }) => (
                                                <SkillRow key={skill.skillId} skill={skill} index={index} mastery={mastery} isDefault={entry.default_skill === index} skillLevel={entry.skill_level} />
                                            ))}
                                        </div>
                                    ) : (
                                        <p className="text-muted-foreground text-sm">{op ? "No skills found." : "Loading skills…"}</p>
                                    )}
                                </ScrollArea>
                            </AccordionContent>
                        </AccordionItem>
                        <AccordionItem className="border-b-0" value="modules">
                            <AccordionTrigger className="py-2 font-medium text-sm hover:underline">Modules</AccordionTrigger>
                            <AccordionContent>
                                {modules.length > 0 ? (
                                    <div className="space-y-2">
                                        {modules.map((m) => (
                                            <ModuleRow key={m.module.uniEquipId} module={m.module} level={m.level} isEquipped={m.isEquipped} />
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-muted-foreground text-sm">{op ? "No modules unlocked." : "Loading modules…"}</p>
                                )}
                            </AccordionContent>
                        </AccordionItem>
                    </Accordion>
                </CardContent>
            </Card>
        </div>
    );
}

function Stat({ label, value }: { label: string; value: number | undefined }) {
    return (
        <div className="flex items-center justify-between">
            <span className="text-muted-foreground">{label}</span>
            <span className={value === undefined ? "font-medium text-muted-foreground" : "font-medium"}>{value ?? "--"}</span>
        </div>
    );
}

function SkillRow({ skill, index, mastery, isDefault, skillLevel }: { skill: IEnrichedSkill; index: number; mastery: number; isDefault: boolean; skillLevel: number }) {
    const name = skill.static?.levels?.[0]?.name ?? `Skill ${index + 1}`;
    return (
        <div className={`grid items-center gap-2 rounded-md px-2.5 py-1.5 ${isDefault ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`} style={{ gridTemplateColumns: "24px minmax(0, 1fr) auto" }}>
            <img alt="Skill" className="h-6 w-6 rounded" decoding="async" height={24} loading="lazy" src={skillIconURL(skill)} width={24} />
            <span className="truncate font-medium text-xs" title={name}>
                {name}
            </span>
            <div className="flex items-center gap-1 text-muted-foreground text-xs">
                <span>Lv.{skillLevel}</span>
                {mastery > 0 && <img alt={`M${mastery}`} className="h-4 w-4" decoding="async" height={16} loading="lazy" src={specializedIcon(mastery)} width={16} />}
            </div>
        </div>
    );
}

function ModuleRow({ module, level, isEquipped }: { module: IModule; level: number; isEquipped: boolean }) {
    return (
        <div className={`flex items-center gap-2 rounded-md px-2.5 py-1.5 ${isEquipped ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`}>
            <img alt="Module" className="h-6 w-6 shrink-0 object-contain" decoding="async" height={24} loading="lazy" src={moduleIconURL(module)} width={24} />
            <div className="min-w-0 flex-1">
                <span className="truncate font-medium text-xs" title={module.uniEquipName}>
                    {module.uniEquipName}
                </span>
            </div>
            <div className="flex shrink-0 items-center gap-1.5 text-muted-foreground text-xs">
                <span>{module.typeName1}</span>
                <span>Lv.{level}</span>
            </div>
        </div>
    );
}
