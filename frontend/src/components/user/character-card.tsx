"use client";

import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import { formatProfession, getOperatorImageUrl, getProfessionIconName, getRarityStarCount } from "~/lib/utils";
import type { CharacterData } from "~/types/api/impl/user";

// Calculate operator stats based on level and phase
// Uses PascalCase property names matching the updated UserCharacterPhase types
function getAttributeStats(
    data: CharacterData,
    operator: {
        phases?: {
            MaxLevel: number;
            AttributesKeyFrames?: { Level: number; Data: { MaxHp: number; Atk: number; Def: number; MagicResistance: number; Cost: number; BlockCnt: number } }[];
        }[];
    } | null,
) {
    const phase = operator?.phases?.[data.evolvePhase];
    const keyFrames = phase?.AttributesKeyFrames;

    if (!keyFrames || keyFrames.length === 0) return null;

    // Find the two keyframes to interpolate between
    const firstFrame = keyFrames[0];
    const lastFrame = keyFrames[keyFrames.length - 1];

    if (!firstFrame || !lastFrame) return null;

    let lower = firstFrame;
    let upper = lastFrame;

    for (let i = 0; i < keyFrames.length - 1; i++) {
        const current = keyFrames[i];
        const next = keyFrames[i + 1];
        if (current && next && current.Level <= data.level && next.Level >= data.level) {
            lower = current;
            upper = next;
            break;
        }
    }

    // Linear interpolation
    const t = upper.Level === lower.Level ? 1 : (data.level - lower.Level) / (upper.Level - lower.Level);

    return {
        maxHp: Math.round(lower.Data.MaxHp + (upper.Data.MaxHp - lower.Data.MaxHp) * t),
        atk: Math.round(lower.Data.Atk + (upper.Data.Atk - lower.Data.Atk) * t),
        def: Math.round(lower.Data.Def + (upper.Data.Def - lower.Data.Def) * t),
        magicResistance: Math.round(lower.Data.MagicResistance + (upper.Data.MagicResistance - lower.Data.MagicResistance) * t),
        cost: Math.round(lower.Data.Cost + (upper.Data.Cost - lower.Data.Cost) * t),
        blockCnt: lower.Data.BlockCnt,
    };
}

interface CharacterCardProps {
    data: CharacterData;
}

export function CharacterCard({ data }: CharacterCardProps) {
    const operator = data.static as {
        name?: string;
        profession?: string;
        rarity?: string;
        trust?: number;
        phases?: {
            MaxLevel: number;
            AttributesKeyFrames?: { Level: number; Data: { MaxHp: number; Atk: number; Def: number; MagicResistance: number; Cost: number; BlockCnt: number } }[];
        }[];
        modules?: {
            uniEquipId: string;
            uniEquipName: string;
            typeName1: string;
            typeName2?: string;
            uniEquipIcon: string;
            image?: string;
        }[];
    } | null;

    const [isHovered, setIsHovered] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const cardRef = useRef<HTMLDivElement>(null);

    const trustPercentage = operator?.trust ? (operator.trust / 200) * 100 : 0;
    const maxLevel = operator?.phases?.[data.evolvePhase]?.MaxLevel ?? 1;
    const stats = getAttributeStats(data, operator);

    useEffect(() => {
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries[0]?.isIntersecting) {
                    setTimeout(() => {
                        setLevelProgress((data.level / maxLevel) * 100);
                        setTrustProgress(trustPercentage);
                    }, 300);
                    observer.disconnect();
                }
            },
            { threshold: 0.2 },
        );

        if (cardRef.current) {
            observer.observe(cardRef.current);
        }

        return () => observer.disconnect();
    }, [data.level, maxLevel, trustPercentage]);

    const operatorName = operator?.name ?? "Unknown Operator";
    const operatorProfession = formatProfession(operator?.profession ?? "");
    const operatorRarity = operator?.rarity ?? "TIER_1";
    const starCount = getRarityStarCount(operatorRarity);
    const operatorImage = getOperatorImageUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);

    return (
        <MorphingDialog transition={{ type: "spring", bounce: 0.05, duration: 0.25 }}>
            <Card className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col overflow-hidden border-2 border-muted/30 transition-all duration-300 hover:border-muted hover:shadow-lg" onMouseEnter={() => setIsHovered(true)} onMouseLeave={() => setIsHovered(false)} ref={cardRef}>
                {/* Image area wrapped in MorphingDialogTrigger for the morph effect */}
                <MorphingDialogTrigger>
                    <div className="relative h-64 w-full cursor-pointer overflow-hidden">
                        <Image alt={operatorName} className={`h-full w-full object-contain transition-transform duration-300 ${isHovered ? "scale-105" : "scale-100"}`} height={200} src={operatorImage || "/placeholder.svg"} unoptimized width={200} />
                        <div className={`absolute inset-0 bg-linear-to-t from-black/50 to-transparent transition-opacity duration-300 ${isHovered ? "opacity-90" : "opacity-70"}`} />

                        {/* Operator Info Overlay */}
                        <div className="absolute right-0 bottom-0 left-0 p-4">
                            <h3 className={`mt-2 max-w-[75%] text-left font-bold text-white text-xl transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>{operatorName}</h3>
                            <div className={`flex items-center justify-between transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>
                                <div className="flex items-center gap-2">
                                    <Image alt={`${starCount} Star`} className="h-[18px] w-auto object-contain" height={18} src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={60} />
                                    <div className="flex flex-row items-center gap-1">
                                        <Image alt={operatorProfession} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${getProfessionIconName(operator?.profession ?? "")}.png`} unoptimized width={20} />
                                        <span className="text-sm text-white">{operatorProfession}</span>
                                    </div>
                                </div>
                                <Image alt="Elite" className="h-6 w-6 object-contain" height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${data.evolvePhase}.png`} unoptimized width={24} />
                            </div>
                        </div>
                    </div>
                </MorphingDialogTrigger>

                {/* Operator Stats */}
                <CardContent className="flex-1 px-4 pt-4 pb-2">
                    {/* Level and Trust Progress */}
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <div className="flex items-center justify-between">
                                <span className="font-medium text-sm">Level</span>
                                <span className="font-bold text-sm">{data.level}</span>
                            </div>
                            <Progress className="h-1.5 transition-all duration-1000 ease-out" value={levelProgress} />
                        </div>
                        <div>
                            <div className="flex items-center justify-between">
                                <span className="font-medium text-sm">Trust</span>
                                <span className="font-bold text-sm">{((operator?.trust ?? 0) / 2).toFixed(0)}%</span>
                            </div>
                            <Progress className="h-1.5 transition-all duration-1000 ease-out" value={trustProgress} />
                        </div>
                    </div>

                    <Separator className="my-3" />

                    {/* Stats Grid */}
                    {stats && (
                        <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">HP</span>
                                <span className="font-medium">{stats.maxHp}</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">ATK</span>
                                <span className="font-medium">{stats.atk}</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">DEF</span>
                                <span className="font-medium">{stats.def}</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">RES</span>
                                <span className="font-medium">{stats.magicResistance}</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">Cost</span>
                                <span className="font-medium">{stats.cost}</span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground">Block</span>
                                <span className="font-medium">{stats.blockCnt}</span>
                            </div>
                        </div>
                    )}

                    <Separator className="my-3" />

                    {/* Accordion Sections */}
                    <Accordion className="w-full" data-accordion type="multiple">
                        {/* Potential */}
                        <AccordionItem className="border-b-0" value="potential">
                            <AccordionTrigger className="py-2 font-medium text-sm">Potential</AccordionTrigger>
                            <AccordionContent>
                                <div className="flex items-center justify-between py-1">
                                    <span className="text-sm">Current Potential</span>
                                    <div className="flex items-center gap-1">
                                        <Image alt={`Potential ${data.potentialRank + 1}`} className="h-6 w-6" height={24} src={`/api/cdn/upk/arts/potential_hub/potential_${data.potentialRank}.png`} unoptimized width={24} />
                                        <span className="text-muted-foreground text-sm">+{data.potentialRank}</span>
                                    </div>
                                </div>
                            </AccordionContent>
                        </AccordionItem>

                        {/* Skills */}
                        <AccordionItem className="border-b-0" value="skills">
                            <AccordionTrigger className="py-2 font-medium text-sm">Skills</AccordionTrigger>
                            <AccordionContent>
                                <ScrollArea className="max-h-[180px]">
                                    {data.skills && data.skills.length > 0 ? (
                                        <div className="space-y-2">
                                            {data.skills.map((skill, index) => {
                                                const skillStatic = skill.static as {
                                                    iconId?: string;
                                                    skillId?: string;
                                                    image?: string;
                                                    levels?: { name?: string }[];
                                                } | null;
                                                const isDefaultSkill = data.defaultSkillIndex === index;

                                                return (
                                                    <div className={`flex items-center gap-2 rounded-md border p-2 ${isDefaultSkill ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={skill.skillId}>
                                                        <Image alt="Skill" className="h-7 w-7 shrink-0" height={28} src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`} unoptimized width={28} />
                                                        <div className="min-w-0 flex-1">
                                                            <div className="truncate font-medium text-sm">{skillStatic?.levels?.[0]?.name ?? `Skill ${index + 1}`}</div>
                                                            <div className="flex items-center gap-1 text-muted-foreground text-xs">
                                                                <span>Lv.{data.mainSkillLvl}</span>
                                                                {skill.specializeLevel > 0 && (
                                                                    <span className="flex items-center gap-0.5">
                                                                        <Image alt={`M${skill.specializeLevel}`} className="h-4 w-4" height={16} src={`/m-${skill.specializeLevel}_0.webp`} unoptimized width={16} />M{skill.specializeLevel}
                                                                    </span>
                                                                )}
                                                            </div>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    ) : (
                                        <p className="text-muted-foreground text-sm">No skills found.</p>
                                    )}
                                </ScrollArea>
                            </AccordionContent>
                        </AccordionItem>

                        {/* Modules */}
                        <AccordionItem className="border-b-0" value="modules">
                            <AccordionTrigger className="py-2 font-medium text-sm">Modules</AccordionTrigger>
                            <AccordionContent>
                                {operator?.modules && operator.modules.length > 0 ? (
                                    <div className="space-y-2">
                                        {operator.modules
                                            .map((module) => {
                                                const equipData = data.equip[module.uniEquipId];
                                                const moduleLevel = equipData?.level ?? 0;
                                                const isEquipped = data.currentEquip === module.uniEquipId;
                                                const isLocked = equipData?.locked === 1;

                                                if (module.typeName1 === "ORIGINAL" || moduleLevel === 0 || isLocked) {
                                                    return null;
                                                }

                                                return (
                                                    <div className={`flex items-center gap-2 rounded-md border p-2 ${isEquipped ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={module.uniEquipId}>
                                                        <Image alt="Module" className="h-7 w-7 shrink-0 object-contain" height={28} src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`} unoptimized width={28} />
                                                        <div className="min-w-0 flex-1">
                                                            <div className="truncate font-medium text-sm">{module.uniEquipName}</div>
                                                            <div className="text-muted-foreground text-xs">
                                                                {module.typeName1} Lv.{moduleLevel}
                                                            </div>
                                                        </div>
                                                    </div>
                                                );
                                            })
                                            .filter(Boolean)}
                                        {!operator.modules.some((module) => {
                                            const equipData = data.equip[module.uniEquipId];
                                            const moduleLevel = equipData?.level ?? 0;
                                            const isLocked = equipData?.locked === 1;
                                            return module.typeName1 !== "ORIGINAL" && moduleLevel > 0 && !isLocked;
                                        }) && <p className="text-muted-foreground text-sm">No modules unlocked.</p>}
                                    </div>
                                ) : (
                                    <p className="text-muted-foreground text-sm">No modules available.</p>
                                )}
                            </AccordionContent>
                        </AccordionItem>
                    </Accordion>
                </CardContent>
            </Card>

            {/* Full Details Dialog */}
            <MorphingDialogContainer>
                <MorphingDialogContent className="relative max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-xl border bg-background">
                    <div className="p-6 pb-4">
                        {/* Dialog Header */}
                        <div className="relative mb-6">
                            <div className="relative h-64 w-full overflow-hidden rounded-lg">
                                <Image alt={operatorName} className="h-full w-full object-contain" fill src={operatorImage || "/placeholder.svg"} unoptimized />
                            </div>
                            <div className="absolute inset-x-0 bottom-0 rounded-b-lg bg-linear-to-t from-black/70 via-black/40 to-transparent p-4 pt-16">
                                <h2 className="font-bold text-3xl text-white">{operatorName}</h2>
                                <div className="mt-2 flex items-center gap-3">
                                    <Image alt={`${starCount} Star`} className="h-6 w-auto object-contain" height={24} src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={80} />
                                    <span className="rounded bg-neutral-700/50 px-2 py-0.5 text-neutral-200 text-sm">{operatorProfession}</span>
                                </div>
                            </div>
                        </div>

                        {/* Stats Grid */}
                        <div className="mb-4 grid grid-cols-4 gap-2">
                            <div className="flex items-center justify-center rounded-lg border p-2">
                                <Image alt={`Elite ${data.evolvePhase}`} className="h-8 w-8 object-contain" height={32} src={`/api/cdn/upk/arts/elite_hub/elite_${data.evolvePhase}.png`} unoptimized width={32} />
                            </div>
                            <div className="rounded-lg border p-2 text-center">
                                <div className="font-bold text-lg">{data.level}</div>
                                <div className="text-muted-foreground text-xs">Level</div>
                            </div>
                            <div className="flex items-center justify-center rounded-lg border p-2">
                                <Image alt={`Potential ${data.potentialRank + 1}`} className="h-8 w-8 object-contain" height={32} src={`/api/cdn/upk/arts/potential_hub/potential_${data.potentialRank}.png`} unoptimized width={32} />
                            </div>
                            <div className="rounded-lg border p-2 text-center">
                                <div className="font-bold text-lg">{operator?.trust ?? 0}%</div>
                                <div className="text-muted-foreground text-xs">Trust</div>
                            </div>
                        </div>

                        {/* Battle Stats */}
                        {stats && (
                            <div className="mb-6">
                                <h3 className="mb-3 font-semibold text-lg">Battle Stats</h3>
                                <div className="grid grid-cols-3 gap-4">
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.maxHp}</div>
                                        <div className="text-muted-foreground text-xs">HP</div>
                                    </div>
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.atk}</div>
                                        <div className="text-muted-foreground text-xs">ATK</div>
                                    </div>
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.def}</div>
                                        <div className="text-muted-foreground text-xs">DEF</div>
                                    </div>
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.magicResistance}</div>
                                        <div className="text-muted-foreground text-xs">RES</div>
                                    </div>
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.cost}</div>
                                        <div className="text-muted-foreground text-xs">Cost</div>
                                    </div>
                                    <div className="rounded-lg border p-3">
                                        <div className="font-bold text-lg">{stats.blockCnt}</div>
                                        <div className="text-muted-foreground text-xs">Block</div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Skills */}
                        <div className="mb-6">
                            <h3 className="mb-3 font-semibold text-lg">Skills (Lv.{data.mainSkillLvl})</h3>
                            <div className="space-y-3">
                                {data.skills && data.skills.length > 0 ? (
                                    data.skills.map((skill, index) => {
                                        const skillStatic = skill.static as {
                                            iconId?: string;
                                            skillId?: string;
                                            image?: string;
                                            levels?: { name?: string }[];
                                        } | null;
                                        const isDefaultSkill = data.defaultSkillIndex === index;

                                        return (
                                            <div className={`flex items-center gap-3 rounded-lg border p-3 ${isDefaultSkill ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={skill.skillId}>
                                                <Image alt="Skill" className="h-10 w-10" height={40} src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`} unoptimized width={40} />
                                                <div className="flex-1">
                                                    <div className="font-medium">
                                                        {skillStatic?.levels?.[0]?.name ?? `Skill ${index + 1}`}
                                                        {isDefaultSkill && <span className="ml-2 text-neutral-500 text-xs">(Equipped)</span>}
                                                    </div>
                                                    <div className="flex items-center gap-2 text-muted-foreground text-sm">
                                                        <span>Lv.{data.mainSkillLvl}</span>
                                                        {skill.specializeLevel > 0 && (
                                                            <span className="flex items-center gap-1">
                                                                <Image alt={`M${skill.specializeLevel}`} className="h-5 w-5" height={20} src={`/m-${skill.specializeLevel}_0.webp`} unoptimized width={20} />M{skill.specializeLevel}
                                                            </span>
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
                                        );
                                    })
                                ) : (
                                    <p className="text-muted-foreground">No skills found.</p>
                                )}
                            </div>
                        </div>

                        {/* Modules */}
                        <div className="mb-4">
                            <h3 className="mb-3 font-semibold text-lg">Modules</h3>
                            <div className="space-y-3">
                                {operator?.modules && operator.modules.length > 0 ? (
                                    <>
                                        {operator.modules
                                            .map((module) => {
                                                const equipData = data.equip[module.uniEquipId];
                                                const moduleLevel = equipData?.level ?? 0;
                                                const isEquipped = data.currentEquip === module.uniEquipId;
                                                const isLocked = equipData?.locked === 1;

                                                if (module.typeName1 === "ORIGINAL" || moduleLevel === 0 || isLocked) {
                                                    return null;
                                                }

                                                return (
                                                    <div className={`flex items-center gap-3 rounded-lg border p-3 ${isEquipped ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={module.uniEquipId}>
                                                        <Image alt="Module" className="h-10 w-10 object-contain" height={40} src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`} unoptimized width={40} />
                                                        <div className="flex-1">
                                                            <div className="text-muted-foreground text-xs">
                                                                {module.typeName1} {module.typeName2 ? `(${module.typeName2})` : ""}
                                                            </div>
                                                            <div className="font-medium">
                                                                {module.uniEquipName}
                                                                {isEquipped && <span className="ml-2 text-neutral-500 text-xs">(Equipped)</span>}
                                                            </div>
                                                            <div className="text-muted-foreground text-sm">Level {moduleLevel}</div>
                                                        </div>
                                                    </div>
                                                );
                                            })
                                            .filter(Boolean)}
                                        {!operator.modules.some((module) => {
                                            const equipData = data.equip[module.uniEquipId];
                                            const moduleLevel = equipData?.level ?? 0;
                                            const isLocked = equipData?.locked === 1;
                                            return module.typeName1 !== "ORIGINAL" && moduleLevel > 0 && !isLocked;
                                        }) && <p className="text-muted-foreground">No modules unlocked.</p>}
                                    </>
                                ) : (
                                    <p className="text-muted-foreground">No modules available.</p>
                                )}
                            </div>
                        </div>

                        {/* Info */}
                        <div className="mb-0">
                            <h3 className="mb-2 font-semibold text-lg">Info</h3>
                            <div className="space-y-2 text-sm">
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Recruited</span>
                                    <span>{new Date(data.gainTime * 1000).toLocaleDateString()}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-muted-foreground">Trust</span>
                                    <span>
                                        {operator?.trust ?? 0}/200 ({((operator?.trust ?? 0) / 2).toFixed(0)}%)
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                    <MorphingDialogClose className="text-muted-foreground hover:text-foreground" />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
}
