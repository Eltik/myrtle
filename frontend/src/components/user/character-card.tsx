"use client";

import { useEffect, useRef, useState } from "react";
import Image from "next/image";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { Separator } from "~/components/ui/shadcn/separator";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Dialog, DialogContent } from "~/components/ui/shadcn/dialog";
import type { CharacterData } from "~/types/api/impl/user";

// Helper functions
function formatProfession(profession: string): string {
    switch (profession?.toUpperCase()) {
        case "PIONEER":
            return "Vanguard";
        case "TANK":
            return "Defender";
        case "SNIPER":
            return "Sniper";
        case "WARRIOR":
            return "Guard";
        case "CASTER":
            return "Caster";
        case "SUPPORT":
            return "Supporter";
        case "SPECIAL":
            return "Specialist";
        case "MEDIC":
            return "Medic";
        default:
            return profession ?? "Unknown";
    }
}

function getRarityStarCount(rarity: string): number {
    const match = rarity?.match(/TIER_(\d)/);
    return match ? Number.parseInt(match[1] ?? "", 10) : 0;
}

function getProfessionIconName(profession: string): string {
    const iconMap: Record<string, string> = {
        WARRIOR: "warrior",
        SNIPER: "sniper",
        TANK: "tank",
        MEDIC: "medic",
        SUPPORT: "support",
        CASTER: "caster",
        SPECIAL: "special",
        PIONEER: "pioneer",
    };
    return iconMap[profession?.toUpperCase()] ?? profession?.toLowerCase() ?? "warrior";
}

function getOperatorImageUrl(charId: string, skin: string, evolvePhase: number, currentTmpl?: string | null, tmpl?: Record<string, { skinId: string }> | null): string {
    let skinId = skin;

    if (currentTmpl && tmpl && tmpl[currentTmpl]) {
        skinId = tmpl[currentTmpl].skinId;
    }

    if (!skinId) {
        const suffix = evolvePhase >= 2 ? "_2" : "_1";
        return `/api/cdn/upk/chararts/${charId}/${charId}${suffix}.png`;
    }

    const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "_");

    if (skinId.includes("@")) {
        return `/api/cdn/upk/skinpack/${charId}/${normalizedSkinId}.png`;
    }

    return `/api/cdn/upk/chararts/${charId}/${normalizedSkinId}.png`;
}

// Calculate operator stats based on level and phase
function getAttributeStats(
    data: CharacterData,
    operator: {
        phases?: {
            maxLevel: number;
            attributesKeyFrames?: { level: number; data: { maxHp: number; atk: number; def: number; magicResistance: number; cost: number; blockCnt: number } }[];
        }[];
    } | null,
) {
    const phase = operator?.phases?.[data.evolvePhase];
    const keyFrames = phase?.attributesKeyFrames;

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
        if (current && next && current.level <= data.level && next.level >= data.level) {
            lower = current;
            upper = next;
            break;
        }
    }

    // Linear interpolation
    const t = upper.level === lower.level ? 1 : (data.level - lower.level) / (upper.level - lower.level);

    return {
        maxHp: Math.round(lower.data.maxHp + (upper.data.maxHp - lower.data.maxHp) * t),
        atk: Math.round(lower.data.atk + (upper.data.atk - lower.data.atk) * t),
        def: Math.round(lower.data.def + (upper.data.def - lower.data.def) * t),
        magicResistance: Math.round(lower.data.magicResistance + (upper.data.magicResistance - lower.data.magicResistance) * t),
        cost: Math.round(lower.data.cost + (upper.data.cost - lower.data.cost) * t),
        blockCnt: lower.data.blockCnt,
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
            maxLevel: number;
            attributesKeyFrames?: { level: number; data: { maxHp: number; atk: number; def: number; magicResistance: number; cost: number; blockCnt: number } }[];
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
    const [isLoaded, setIsLoaded] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const [isDialogOpen, setIsDialogOpen] = useState(false);
    const cardRef = useRef<HTMLDivElement>(null);

    const trustPercentage = operator?.trust ? (operator.trust / 200) * 100 : 0;
    const maxLevel = operator?.phases?.[data.evolvePhase]?.maxLevel ?? 1;
    const stats = getAttributeStats(data, operator);

    useEffect(() => {
        setIsLoaded(true);
    }, []);

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

    const handleCardClick = (e: React.MouseEvent) => {
        const isAccordionClick = (e.target as HTMLElement).closest("[data-accordion]") !== null;
        if (!isAccordionClick) {
            setIsDialogOpen(true);
        }
    };

    const operatorName = operator?.name ?? "Unknown Operator";
    const operatorProfession = formatProfession(operator?.profession ?? "");
    const operatorRarity = operator?.rarity ?? "TIER_1";
    const starCount = getRarityStarCount(operatorRarity);
    const operatorImage = getOperatorImageUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);

    return (
        <>
            <Card
                ref={cardRef}
                className={`flex w-full cursor-pointer flex-col overflow-hidden border-2 border-muted/30 transition-all duration-300 hover:border-muted hover:shadow-lg ${isLoaded ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0"}`}
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
                onClick={handleCardClick}
            >
                <div className="relative">
                    {/* Operator Image */}
                    <div className="relative h-64 w-full overflow-hidden">
                        <Image
                            loading="lazy"
                            decoding="async"
                            src={operatorImage || "/placeholder.svg"}
                            alt={operatorName}
                            className={`h-full w-full object-contain transition-transform duration-300 ${isHovered ? "scale-105" : "scale-100"}`}
                            width={200}
                            height={200}
                            unoptimized
                        />
                        <div className={`absolute inset-0 bg-gradient-to-t from-black/70 to-transparent transition-opacity duration-300 ${isHovered ? "opacity-90" : "opacity-70"}`} />

                        {/* Operator Info Overlay */}
                        <div className="absolute bottom-0 left-0 right-0 p-4">
                            <h3 className={`mt-2 max-w-[75%] text-xl font-bold text-white transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>{operatorName}</h3>
                            <div className={`flex items-center justify-between transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>
                                <div className="flex items-center gap-2">
                                    <div className="flex flex-row gap-0">
                                        {Array(starCount)
                                            .fill(0)
                                            .map((_, i) => (
                                                <Image key={i} src="/api/cdn/upk/arts/ui/[uc]charcommon/icon_star_selected.png" width={16} height={16} alt="Star" className="h-4 w-4" unoptimized />
                                            ))}
                                    </div>
                                    <div className="flex flex-row items-center gap-1">
                                        <Image src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${getProfessionIconName(operator?.profession ?? "")}.png`} width={20} height={20} alt={operatorProfession} className="h-5 w-5" unoptimized />
                                        <span className="text-sm text-white">{operatorProfession}</span>
                                    </div>
                                </div>
                                <Image src={`/api/cdn/upk/arts/elite_hub/elite_${data.evolvePhase}.png`} width={24} height={24} alt="Elite" className="h-6 w-6" unoptimized />
                            </div>
                        </div>
                    </div>

                    {/* Operator Stats */}
                    <CardContent className="flex-1 p-4">
                        {/* Level and Trust Progress */}
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">Level</span>
                                    <span className="text-sm font-bold">{data.level}</span>
                                </div>
                                <Progress value={levelProgress} className="h-1.5 transition-all duration-1000 ease-out" />
                            </div>
                            <div>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">Trust</span>
                                    <span className="text-sm font-bold">{operator?.trust ?? 0}/200</span>
                                </div>
                                <Progress value={trustProgress} className="h-1.5 transition-all duration-1000 ease-out" />
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
                        <Accordion type="multiple" className="w-full" data-accordion>
                            {/* Potential */}
                            <AccordionItem value="potential" className="border-b-0">
                                <AccordionTrigger className="py-2 text-sm font-medium">Potential</AccordionTrigger>
                                <AccordionContent>
                                    <div className="flex items-center justify-between py-1">
                                        <span className="text-sm">Current Potential</span>
                                        <div className="flex items-center gap-1">
                                            <Image src={`/api/cdn/upk/arts/potential_hub/potential_${data.potentialRank}.png`} width={24} height={24} alt={`Potential ${data.potentialRank + 1}`} className="h-6 w-6" unoptimized />
                                            <span className="text-sm text-muted-foreground">+{data.potentialRank}</span>
                                        </div>
                                    </div>
                                </AccordionContent>
                            </AccordionItem>

                            {/* Skills */}
                            <AccordionItem value="skills" className="border-b-0">
                                <AccordionTrigger className="py-2 text-sm font-medium">Skills</AccordionTrigger>
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
                                                        <div key={skill.skillId} className={`flex items-center gap-2 rounded-md border p-2 ${isDefaultSkill ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                            <Image
                                                                src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`}
                                                                width={28}
                                                                height={28}
                                                                alt="Skill"
                                                                className="h-7 w-7 flex-shrink-0"
                                                                unoptimized
                                                            />
                                                            <div className="min-w-0 flex-1">
                                                                <div className="truncate text-sm font-medium">{skillStatic?.levels?.[0]?.name ?? `Skill ${index + 1}`}</div>
                                                                <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                                                    <span>Lv.{data.mainSkillLvl}</span>
                                                                    {skill.specializeLevel > 0 && (
                                                                        <span className="flex items-center gap-0.5">
                                                                            <Image src={`/m-${skill.specializeLevel}_0.webp`} width={16} height={16} alt={`M${skill.specializeLevel}`} className="h-4 w-4" unoptimized />
                                                                            M{skill.specializeLevel}
                                                                        </span>
                                                                    )}
                                                                </div>
                                                            </div>
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        ) : (
                                            <p className="text-sm text-muted-foreground">No skills found.</p>
                                        )}
                                    </ScrollArea>
                                </AccordionContent>
                            </AccordionItem>

                            {/* Modules */}
                            <AccordionItem value="modules" className="border-b-0">
                                <AccordionTrigger className="py-2 text-sm font-medium">Modules</AccordionTrigger>
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
                                                        <div key={module.uniEquipId} className={`flex items-center gap-2 rounded-md border p-2 ${isEquipped ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                            <Image
                                                                src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`}
                                                                width={28}
                                                                height={28}
                                                                alt="Module"
                                                                className="h-7 w-7 flex-shrink-0 object-contain"
                                                                unoptimized
                                                            />
                                                            <div className="min-w-0 flex-1">
                                                                <div className="truncate text-sm font-medium">{module.uniEquipName}</div>
                                                                <div className="text-xs text-muted-foreground">
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
                                            }) && <p className="text-sm text-muted-foreground">No modules unlocked.</p>}
                                        </div>
                                    ) : (
                                        <p className="text-sm text-muted-foreground">No modules available.</p>
                                    )}
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </CardContent>
                </div>
            </Card>

            {/* Full Details Dialog */}
            <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                <DialogContent className="max-h-[90vh] max-w-2xl overflow-hidden p-0">
                    <ScrollArea className="max-h-[90vh]">
                        <div className="p-6">
                            {/* Dialog Header */}
                            <div className="relative mb-6">
                                <div className="relative h-64 w-full overflow-hidden rounded-lg">
                                    <Image src={operatorImage || "/placeholder.svg"} alt={operatorName} className="h-full w-full object-contain" fill unoptimized />
                                </div>
                                <div className="absolute bottom-0 left-0 right-0 rounded-b-lg bg-gradient-to-t from-black to-transparent p-4">
                                    <h2 className="text-3xl font-bold text-white">{operatorName}</h2>
                                    <div className="mt-2 flex items-center gap-3">
                                        <div className="flex">
                                            {Array(starCount)
                                                .fill(0)
                                                .map((_, i) => (
                                                    <span key={i} className="text-lg text-yellow-400">
                                                        ★
                                                    </span>
                                                ))}
                                        </div>
                                        <span className="rounded bg-primary/20 px-2 py-0.5 text-sm text-primary">{operatorProfession}</span>
                                    </div>
                                </div>
                            </div>

                            {/* Stats Grid */}
                            <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
                                <div className="rounded-lg border p-3 text-center">
                                    <div className="text-2xl font-bold">E{data.evolvePhase}</div>
                                    <div className="text-xs text-muted-foreground">Elite</div>
                                </div>
                                <div className="rounded-lg border p-3 text-center">
                                    <div className="text-2xl font-bold">{data.level}</div>
                                    <div className="text-xs text-muted-foreground">Level</div>
                                </div>
                                <div className="rounded-lg border p-3 text-center">
                                    <div className="text-2xl font-bold">{data.potentialRank + 1}</div>
                                    <div className="text-xs text-muted-foreground">Potential</div>
                                </div>
                                <div className="rounded-lg border p-3 text-center">
                                    <div className="text-2xl font-bold">{operator?.trust ?? 0}%</div>
                                    <div className="text-xs text-muted-foreground">Trust</div>
                                </div>
                            </div>

                            {/* Battle Stats */}
                            {stats && (
                                <div className="mb-6">
                                    <h3 className="mb-3 text-lg font-semibold">Battle Stats</h3>
                                    <div className="grid grid-cols-3 gap-4">
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.maxHp}</div>
                                            <div className="text-xs text-muted-foreground">HP</div>
                                        </div>
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.atk}</div>
                                            <div className="text-xs text-muted-foreground">ATK</div>
                                        </div>
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.def}</div>
                                            <div className="text-xs text-muted-foreground">DEF</div>
                                        </div>
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.magicResistance}</div>
                                            <div className="text-xs text-muted-foreground">RES</div>
                                        </div>
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.cost}</div>
                                            <div className="text-xs text-muted-foreground">Cost</div>
                                        </div>
                                        <div className="rounded-lg border p-3">
                                            <div className="text-lg font-bold">{stats.blockCnt}</div>
                                            <div className="text-xs text-muted-foreground">Block</div>
                                        </div>
                                    </div>
                                </div>
                            )}

                            {/* Skills */}
                            <div className="mb-6">
                                <h3 className="mb-3 text-lg font-semibold">Skills (Lv.{data.mainSkillLvl})</h3>
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
                                                <div key={skill.skillId} className={`flex items-center gap-3 rounded-lg border p-3 ${isDefaultSkill ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                    <Image
                                                        src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`}
                                                        width={40}
                                                        height={40}
                                                        alt="Skill"
                                                        className="h-10 w-10"
                                                        unoptimized
                                                    />
                                                    <div className="flex-1">
                                                        <div className="font-medium">
                                                            {skillStatic?.levels?.[0]?.name ?? `Skill ${index + 1}`}
                                                            {isDefaultSkill && <span className="ml-2 text-xs text-blue-500">(Equipped)</span>}
                                                        </div>
                                                        <div className="flex items-center gap-2 text-sm text-muted-foreground">
                                                            <span>Lv.{data.mainSkillLvl}</span>
                                                            {skill.specializeLevel > 0 && (
                                                                <span className="flex items-center gap-1">
                                                                    <Image src={`/m-${skill.specializeLevel}_0.webp`} width={20} height={20} alt={`M${skill.specializeLevel}`} className="h-5 w-5" unoptimized />
                                                                    M{skill.specializeLevel}
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
                            <div className="mb-6">
                                <h3 className="mb-3 text-lg font-semibold">Modules</h3>
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
                                                        <div key={module.uniEquipId} className={`flex items-center gap-3 rounded-lg border p-3 ${isEquipped ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                            <Image
                                                                src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`}
                                                                width={40}
                                                                height={40}
                                                                alt="Module"
                                                                className="h-10 w-10 object-contain"
                                                                unoptimized
                                                            />
                                                            <div className="flex-1">
                                                                <div className="text-xs text-muted-foreground">
                                                                    {module.typeName1} {module.typeName2 ? `(${module.typeName2})` : ""}
                                                                </div>
                                                                <div className="font-medium">
                                                                    {module.uniEquipName}
                                                                    {isEquipped && <span className="ml-2 text-xs text-blue-500">(Equipped)</span>}
                                                                </div>
                                                                <div className="text-sm text-muted-foreground">Level {moduleLevel}</div>
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
                            <div>
                                <h3 className="mb-3 text-lg font-semibold">Info</h3>
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
                    </ScrollArea>
                </DialogContent>
            </Dialog>
        </>
    );
}
