"use client";

import { motion, useScroll, useTransform } from "motion/react";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import { RARITY_COLORS } from "~/components/operators/list/constants";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import { formatProfession, getOperatorImageUrl, getProfessionIconName, getRarityStarCount } from "~/lib/utils";
import type { CharacterData, CharacterStatic, UserCharacterModule, UserFavorKeyFrame, UserPotentialRank } from "~/types/api/impl/user";

// Linear interpolation helper
function linearInterpolateByLevel(level: number, maxLevel: number, baseValue: number, maxValue: number): number {
    if (maxLevel === 1) return baseValue;
    return Math.round(baseValue + ((level - 1) * (maxValue - baseValue)) / (maxLevel - 1));
}

// Get trust bonus stats (trust only affects HP, ATK, and DEF - not magic resistance)
function getStatIncreaseAtTrust(favorKeyFrames: UserFavorKeyFrame[] | undefined, rawTrust: number): { maxHp: number; atk: number; def: number } {
    if (!favorKeyFrames || favorKeyFrames.length === 0) {
        return { maxHp: 0, atk: 0, def: 0 };
    }

    // Trust is capped at 100 for stat calculations (even though favor points go to 200)
    const trust = Math.min(100, Math.floor(rawTrust / 2));
    const maxTrustFrame = favorKeyFrames[favorKeyFrames.length - 1]?.Data;

    return {
        maxHp: Math.round((trust * (maxTrustFrame?.MaxHp ?? 0)) / 100),
        atk: Math.round((trust * (maxTrustFrame?.Atk ?? 0)) / 100),
        def: Math.round((trust * (maxTrustFrame?.Def ?? 0)) / 100),
    };
}

// Get potential bonus stats
// Handles both camelCase (actual API response) and PascalCase (TypeScript types) property access
function getStatIncreaseAtPotential(
    potentialRanks: UserPotentialRank[] | undefined,
    potential: number,
): {
    health: number;
    attackPower: number;
    defense: number;
    artsResistance: number;
    dpCost: number;
    attackSpeed: number;
    blockCnt: number;
} {
    const statChanges = {
        health: 0,
        attackPower: 0,
        defense: 0,
        artsResistance: 0,
        dpCost: 0,
        attackSpeed: 0,
        blockCnt: 0,
    };

    if (!potentialRanks || potential === 0) {
        return statChanges;
    }

    // Aggregate bonuses from potential ranks 1 through current potential
    for (let p = 1; p <= potential; p++) {
        const pot = potentialRanks[p - 1];
        if (!pot) continue;

        // Access using both camelCase (actual API) and PascalCase (TypeScript types)
        // biome-ignore lint/suspicious/noExplicitAny: API may return camelCase but types are PascalCase
        const potAny = pot as any;

        // Try camelCase first (actual API), then PascalCase (TypeScript types)
        const buff = potAny.buff ?? pot.Buff;
        if (!buff) continue;

        const attributes = buff.attributes ?? buff.Attributes;
        if (!attributes) continue;

        const modifiers = attributes.attributeModifiers ?? attributes.AttributeModifiers;
        if (!modifiers || modifiers.length === 0) continue;

        const modifier = modifiers[0];
        if (!modifier) continue;

        const attribType = modifier.attributeType ?? modifier.AttributeType;
        const attribChange = modifier.value ?? modifier.Value ?? 0;

        // AttributeType is a string like "MAX_HP", "ATK", "DEF", etc.
        switch (attribType) {
            case "MAX_HP":
                statChanges.health += attribChange;
                break;
            case "ATK":
                statChanges.attackPower += attribChange;
                break;
            case "DEF":
                statChanges.defense += attribChange;
                break;
            case "MAGIC_RESISTANCE":
                statChanges.artsResistance += attribChange;
                break;
            case "COST":
                statChanges.dpCost += attribChange;
                break;
            case "ATTACK_SPEED":
                statChanges.attackSpeed += attribChange;
                break;
            case "RESPAWN_TIME":
                // Redeploy time reduction (e.g., -4 seconds)
                break;
            case "BLOCK_CNT":
                statChanges.blockCnt += attribChange;
                break;
        }
    }

    return statChanges;
}

// Get module bonus stats
function getModuleStatIncrease(
    modules: UserCharacterModule[] | undefined,
    currentEquip: string | null,
    equipData: Record<string, { hide: number; locked: number; level: number }>,
): {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    attackSpeed: number;
    blockCnt: number;
} {
    const statChanges = {
        maxHp: 0,
        atk: 0,
        def: 0,
        magicResistance: 0,
        cost: 0,
        attackSpeed: 0,
        blockCnt: 0,
    };

    if (!modules || !currentEquip) {
        return statChanges;
    }

    const equippedModule = modules.find((m) => m.uniEquipId === currentEquip);
    if (!equippedModule?.data?.phases) {
        return statChanges;
    }

    const moduleLevel = equipData[currentEquip]?.level ?? 0;
    if (moduleLevel <= 0) {
        return statChanges;
    }

    // Module phases are 0-indexed, level 1 = phase[0], level 2 = phase[1], etc.
    const modulePhase = equippedModule.data.phases[moduleLevel - 1];
    if (!modulePhase?.attributeBlackboard) {
        return statChanges;
    }

    for (const attr of modulePhase.attributeBlackboard) {
        switch (attr.key) {
            case "atk":
                statChanges.atk += attr.value;
                break;
            case "max_hp":
                statChanges.maxHp += attr.value;
                break;
            case "def":
                statChanges.def += attr.value;
                break;
            case "magic_resistance":
                statChanges.magicResistance += attr.value;
                break;
            case "cost":
                statChanges.cost += attr.value;
                break;
            case "attack_speed":
                statChanges.attackSpeed += attr.value;
                break;
            case "block_cnt":
                statChanges.blockCnt += attr.value;
                break;
        }
    }

    return statChanges;
}

// Calculate operator stats based on level, phase, trust, potential, and modules
function getAttributeStats(data: CharacterData, operator: CharacterStatic | null) {
    const phase = operator?.phases?.[data.evolvePhase];
    const keyFrames = phase?.AttributesKeyFrames;

    if (!keyFrames || keyFrames.length === 0) return null;

    const firstFrame = keyFrames[0];
    const lastFrame = keyFrames[keyFrames.length - 1];

    if (!firstFrame || !lastFrame) return null;

    const maxLevel = phase.MaxLevel;

    // Base stats from keyframes
    const baseMaxHp = firstFrame.Data.MaxHp;
    const baseAtk = firstFrame.Data.Atk;
    const baseDef = firstFrame.Data.Def;
    const baseRes = firstFrame.Data.MagicResistance;
    const baseCost = firstFrame.Data.Cost;
    const baseBlockCnt = firstFrame.Data.BlockCnt;

    const finalMaxHp = lastFrame.Data.MaxHp;
    const finalAtk = lastFrame.Data.Atk;
    const finalDef = lastFrame.Data.Def;
    const finalRes = lastFrame.Data.MagicResistance;

    // Get trust bonuses
    const trustBonuses = getStatIncreaseAtTrust(operator?.favorKeyFrames, data.favorPoint);

    // Get potential bonuses
    const potBonuses = getStatIncreaseAtPotential(operator?.potentialRanks, data.potentialRank);

    // Get module bonuses (only at E2)
    const modBonuses = data.evolvePhase === 2 ? getModuleStatIncrease(operator?.modules, data.currentEquip, data.equip) : { maxHp: 0, atk: 0, def: 0, magicResistance: 0, cost: 0, attackSpeed: 0, blockCnt: 0 };

    // Calculate final stats with all bonuses
    const maxHp = linearInterpolateByLevel(data.level, maxLevel, baseMaxHp, finalMaxHp) + trustBonuses.maxHp + potBonuses.health + modBonuses.maxHp;
    const atk = linearInterpolateByLevel(data.level, maxLevel, baseAtk, finalAtk) + trustBonuses.atk + potBonuses.attackPower + modBonuses.atk;
    const def = linearInterpolateByLevel(data.level, maxLevel, baseDef, finalDef) + trustBonuses.def + potBonuses.defense + modBonuses.def;
    const magicResistance = linearInterpolateByLevel(data.level, maxLevel, baseRes, finalRes) + potBonuses.artsResistance + modBonuses.magicResistance;
    const cost = baseCost + potBonuses.dpCost + modBonuses.cost;
    const blockCnt = baseBlockCnt + potBonuses.blockCnt + modBonuses.blockCnt;

    return {
        maxHp,
        atk,
        def,
        magicResistance,
        cost,
        blockCnt,
    };
}

interface CharacterCardProps {
    data: CharacterData;
}

export function CharacterCard({ data }: CharacterCardProps) {
    const operator = data.static as CharacterStatic | null;

    const [isHovered, setIsHovered] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const cardRef = useRef<HTMLDivElement>(null);
    const [dialogScrollElement, setDialogScrollElement] = useState<HTMLDivElement | null>(null);

    // Parallax effect for dialog hero image
    const { scrollYProgress } = useScroll({
        container: dialogScrollElement ? { current: dialogScrollElement } : undefined,
    });

    // Parallax: image moves up slower than scroll, creating depth
    const heroImageY = useTransform(scrollYProgress, [0, 0.3], [0, -30]);
    const heroImageScale = useTransform(scrollYProgress, [0, 0.3], [1, 1.08]);
    // Vignette gets more prominent when scrolling
    const vignetteOpacity = useTransform(scrollYProgress, [0, 0.2], [0.4, 1]);
    // Text fades out as user scrolls
    const heroContentOpacity = useTransform(scrollYProgress, [0, 0.15], [1, 0]);
    const heroContentY = useTransform(scrollYProgress, [0, 0.15], [0, -15]);

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

    // Check if operator is fully maxed
    const isMaxed = (() => {
        // Max potential is 5 (0-indexed, represents potential 6)
        const isMaxPotential = data.potentialRank === 5;

        // Max level depends on rarity - need to be at highest elite phase with max level
        const maxElitePhase = (operator?.phases?.length ?? 1) - 1;
        const isAtMaxElite = data.evolvePhase === maxElitePhase;
        const maxLevelForPhase = operator?.phases?.[maxElitePhase]?.MaxLevel ?? 1;
        const isMaxLevel = isAtMaxElite && data.level === maxLevelForPhase;

        // Max skills: M3 on all skills for 4+ stars, or skill level 7 for 3-stars
        const isThreeStar = starCount <= 3;
        const isMaxSkills = isThreeStar
            ? data.mainSkillLvl === 7 // 3-stars just need skill level 7
            : data.skills.length > 0 && data.skills.every((skill) => skill.specializeLevel === 3);

        return isMaxPotential && isMaxLevel && isMaxSkills;
    })();
    const operatorImage = getOperatorImageUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);
    const rarityColor = RARITY_COLORS[starCount] ?? "#ffffff";

    return (
        <MorphingDialog transition={{ type: "spring", bounce: 0.05, duration: 0.25 }}>
            <Card
                className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col gap-0 overflow-hidden border-2 border-muted/30 pb-1 transition-all duration-300 hover:border-muted hover:shadow-lg"
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
                ref={cardRef}
                style={isMaxed ? { boxShadow: `0 0 20px ${rarityColor}40, 0 0 40px ${rarityColor}20` } : undefined}
            >
                {/* Image area wrapper */}
                <div className="relative">
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

                    {/* Maxed indicator */}
                    {isMaxed && (
                        <div className="-top-3 absolute z-10 rounded-r-md px-2 py-0.5 text-center font-semibold text-xs shadow-md" style={{ color: rarityColor }}>
                            Maxed
                        </div>
                    )}
                </div>

                {/* Operator Stats */}
                <CardContent className="flex-1 px-4 pt-2 pb-2">
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
                                <span className="font-bold text-sm">{(operator?.trust ?? 0).toFixed(0)}%</span>
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
                                        <div className="space-y-2 overflow-hidden">
                                            {data.skills.map((skill, index) => {
                                                const skillStatic = skill.static as {
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
                                                    }
                                                } | null;
                                                const isDefaultSkill = data.defaultSkillIndex === index;

                                                return (
                                                    <div className={`flex items-center gap-2 overflow-hidden rounded-md border p-2 ${isDefaultSkill ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={skill.skillId}>
                                                        <Image alt="Skill" className="h-7 w-7 shrink-0" height={28} src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`} unoptimized width={28} />
                                                        <div className="w-0 flex-1 overflow-hidden">
                                                            <div className="truncate font-medium text-sm" title={skillStatic?.name ?? `Skill ${index + 1}`}>{skillStatic?.name ?? `Skill ${index + 1}`}</div>
                                                            <div className="flex items-center gap-1 text-muted-foreground text-xs">
                                                                <span>Lv.{data.mainSkillLvl}</span>
                                                                {skill.specializeLevel > 0 && (
                                                                    <span className="flex items-center gap-0.5">
                                                                        <Image alt={`M${skill.specializeLevel}`} className="h-4 w-4" height={16} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={16} />M{skill.specializeLevel}
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
                <MorphingDialogContent className="relative max-h-[90vh] w-full max-w-2xl rounded-xl border bg-background">
                    <div className="max-h-[90vh] overflow-y-auto p-6 pb-4" ref={setDialogScrollElement} style={{ scrollbarGutter: "stable" }}>
                        {/* Dialog Header with Parallax */}
                        <div className="relative mb-6 h-64 overflow-hidden rounded-lg">
                            {/* Parallax image */}
                            <motion.div
                                className="absolute inset-0"
                                style={{
                                    y: heroImageY,
                                    scale: heroImageScale,
                                }}
                            >
                                <Image alt={operatorName} className="h-full w-full object-contain" fill src={operatorImage || "/placeholder.svg"} unoptimized />
                            </motion.div>
                            {/* Vignette overlay - stays and intensifies */}
                            <motion.div className="pointer-events-none absolute inset-0 rounded-lg bg-linear-to-t from-black via-black/50 to-transparent" style={{ opacity: vignetteOpacity }} />
                            {/* Text content - fades out */}
                            <motion.div
                                className="absolute inset-x-0 bottom-0 p-4"
                                style={{
                                    opacity: heroContentOpacity,
                                    y: heroContentY,
                                }}
                            >
                                <h2 className="font-bold text-3xl text-white">{operatorName}</h2>
                                <div className="mt-2 flex items-center gap-3">
                                    <Image alt={`${starCount} Star`} className="h-6 w-auto object-contain" height={24} src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={80} />
                                    <span className="rounded bg-neutral-700/50 px-2 py-0.5 text-neutral-200 text-sm">{operatorProfession}</span>
                                </div>
                            </motion.div>
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
                                        } | null;
                                        const isDefaultSkill = data.defaultSkillIndex === index;

                                        return (
                                            <div className={`flex items-center gap-3 rounded-lg border p-3 ${isDefaultSkill ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`} key={skill.skillId}>
                                                <Image alt="Skill" className="h-10 w-10" height={40} src={skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`} unoptimized width={40} />
                                                <div className="min-w-0 flex-1">
                                                    <div className="truncate font-medium" title={skillStatic?.name ?? `Skill ${index + 1}`}>
                                                        {skillStatic?.name ?? `Skill ${index + 1}`}
                                                        {isDefaultSkill && <span className="ml-2 text-neutral-500 text-xs">(Equipped)</span>}
                                                    </div>
                                                    <div className="flex items-center gap-2 text-muted-foreground text-sm">
                                                        <span>Lv.{data.mainSkillLvl}</span>
                                                        {skill.specializeLevel > 0 && (
                                                            <span className="flex items-center gap-1">
                                                                <Image alt={`M${skill.specializeLevel}`} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={20} />M{skill.specializeLevel}
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
                                    <span className="text-muted-foreground">Voice</span>
                                    <span className="capitalize">{data.voiceLan === "JP" ? "Japanese" : data.voiceLan === "CN_MANDARIN" ? "Chinese" : data.voiceLan === "EN" ? "English" : data.voiceLan === "KR" ? "Korean" : (data.voiceLan?.toLowerCase().replace("_", " ") ?? "Japanese")}</span>
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
