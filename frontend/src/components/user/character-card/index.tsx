"use client";

import Image from "next/image";
import { useCallback, useEffect, useRef, useState } from "react";
import { RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { ImageWithSkeleton } from "~/components/ui/image-with-skeleton";
import { MorphingDialog, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Progress } from "~/components/ui/shadcn/progress";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import { useCDNPrefetch } from "~/hooks/use-cdn-prefetch";
import { formatProfession, getOperatorImageURL, getProfessionIconName, getRarityStarCount } from "~/lib/utils";
import type { CharacterStatic, EnrichedRosterEntry } from "~/types/api/impl/user";
import { CharacterDialog } from "./impl/character-dialog";
import { getAttributeStats } from "./impl/helpers";
import { ModuleItem } from "./impl/module-item";
import { SkillItem } from "./impl/skill-item";

interface CharacterCardProps {
    data: EnrichedRosterEntry;
}

export function CharacterCard({ data }: CharacterCardProps) {
    const operator = data.static as CharacterStatic | null;

    const [isHovered, setIsHovered] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const cardRef = useRef<HTMLDivElement>(null);
    const hoverTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const hasPreloadedRef = useRef(false);
    const { prefetch } = useCDNPrefetch();

    const trustPercentage = operator?.trust ? (operator.trust / 200) * 100 : 0;
    const maxLevel = operator?.phases?.[data.elite]?.MaxLevel ?? 1;
    const stats = getAttributeStats(data, operator);

    // Build skill display data by merging roster masteries with static skill data
    const skillDisplayData = (operator?.skills ?? []).map((staticSkill, index) => {
        const mastery = data.masteries.find((m) => m.skill_id === staticSkill.skillId);
        return {
            skillId: staticSkill.skillId,
            specializeLevel: mastery?.specialize_level ?? 0,
            skillStatic: staticSkill.static ?? null,
            index,
        };
    });

    // Build module display data by merging static modules with roster modules
    const moduleDisplayData = (operator?.modules ?? [])
        .filter((mod) => mod.typeName1 !== "ORIGINAL")
        .map((mod) => {
            const rosterMod = data.modules.find((m) => m.equip_id === mod.uniEquipId);
            return {
                module: mod,
                level: rosterMod?.level ?? 0,
                isEquipped: data.current_equip === mod.uniEquipId,
            };
        })
        .filter((m) => m.level > 0);

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

    const preloadDialogImages = useCallback(() => {
        if (hasPreloadedRef.current) return;
        hasPreloadedRef.current = true;

        const operatorImage = getOperatorImageURL(data.operator_id, data.skin_id ?? "", data.elite, data.current_tmpl);

        const imagesToPreload: string[] = [operatorImage, `/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`, `/api/cdn/upk/arts/elite_hub/elite_${data.elite}.png`, `/api/cdn/upk/arts/potential_hub/potential_${data.potential}.png`];

        for (const skill of skillDisplayData) {
            const skillIcon = skill.skillStatic?.image ? `/api/cdn${skill.skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skill.skillStatic?.iconId ?? skill.skillStatic?.skillId ?? skill.skillId}.png`;
            imagesToPreload.push(skillIcon);

            if (skill.specializeLevel > 0) {
                imagesToPreload.push(`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`);
            }
        }

        for (const mod of moduleDisplayData) {
            const moduleImage = mod.module.image ? `/api/cdn${mod.module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${mod.module.uniEquipIcon}.png`;
            imagesToPreload.push(moduleImage);
        }

        prefetch(imagesToPreload, "high");
    }, [data, skillDisplayData, moduleDisplayData, starCount, prefetch]);

    const handleMouseEnter = useCallback(() => {
        setIsHovered(true);
        hoverTimeoutRef.current = setTimeout(preloadDialogImages, 150);
    }, [preloadDialogImages]);

    const handleMouseLeave = useCallback(() => {
        setIsHovered(false);
        if (hoverTimeoutRef.current) {
            clearTimeout(hoverTimeoutRef.current);
            hoverTimeoutRef.current = null;
        }
    }, []);

    const isMaxed = (() => {
        const isMaxPotential = data.potential === 5;

        const maxElitePhase = (operator?.phases?.length ?? 1) - 1;
        const isAtMaxElite = data.elite === maxElitePhase;
        const maxLevelForPhase = operator?.phases?.[maxElitePhase]?.MaxLevel ?? 1;
        const isMaxLevel = isAtMaxElite && data.level === maxLevelForPhase;

        const isThreeStar = starCount <= 3;
        const isMaxSkills = isThreeStar ? data.skill_level === 7 : data.masteries.length > 0 && data.masteries.every((m) => m.specialize_level === 3);

        return isMaxPotential && isMaxLevel && isMaxSkills;
    })();
    const operatorImage = getOperatorImageURL(data.operator_id, data.skin_id ?? "", data.elite, data.current_tmpl);
    const rarityColor = RARITY_COLORS[starCount] ?? "#ffffff";

    return (
        <MorphingDialog transition={{ type: "spring", bounce: 0.05, duration: 0.25 }}>
            <Card
                className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col gap-0 overflow-hidden border-2 border-muted/30 py-0 pb-1 transition-all duration-300 hover:border-muted hover:shadow-lg"
                onMouseEnter={handleMouseEnter}
                onMouseLeave={handleMouseLeave}
                ref={cardRef}
                style={isMaxed ? { boxShadow: `0 0 20px ${rarityColor}60, 0 0 40px ${rarityColor}40` } : undefined}
            >
                {/* Image area wrapper */}
                <div className="relative">
                    <MorphingDialogTrigger className="block w-full">
                        <div className="relative h-64 w-full cursor-pointer overflow-hidden">
                            <ImageWithSkeleton alt={operatorName} className={`h-full w-full object-contain object-top transition-transform duration-300 ${isHovered ? "scale-105" : "scale-100"}`} containerClassName="h-full w-full" height={512} src={operatorImage || "/placeholder.svg"} unoptimized width={512} />
                            <div className={`absolute inset-0 bg-linear-to-t from-black/50 to-transparent transition-opacity duration-300 ${isHovered ? "opacity-90" : "opacity-70"}`} />

                            {/* Operator Info Overlay */}
                            <div className="absolute right-0 bottom-0 left-0 p-4">
                                <h3 className={`mt-2 max-w-3/4 text-left font-bold text-white text-xl transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>{operatorName}</h3>
                                <div className={`flex items-center justify-between transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>
                                    <div className="flex items-center gap-2">
                                        <Image alt={`${starCount} Star`} className="h-4.5 w-auto object-contain" height={18} src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={60} />
                                        <div className="flex flex-row items-center gap-1">
                                            <Image alt={operatorProfession} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${getProfessionIconName(operator?.profession ?? "")}.png`} unoptimized width={20} />
                                            <span className="text-sm text-white">{operatorProfession}</span>
                                        </div>
                                    </div>
                                    <Image alt="Elite" className="h-6 w-6 object-contain" height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${data.elite}.png`} unoptimized width={24} />
                                </div>
                            </div>
                        </div>
                    </MorphingDialogTrigger>

                    {/* Maxed indicator */}
                    {isMaxed && (
                        <div className="absolute top-2 z-10 rounded-r-md px-2 py-0.5 text-center font-semibold text-xs shadow-md" style={{ color: rarityColor }}>
                            Maxed
                        </div>
                    )}
                </div>

                {/* Operator Stats */}
                <CardContent className="min-w-0 flex-1 overflow-hidden px-4 pt-2 pb-2">
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
                                        <Image alt={`Potential ${data.potential + 1}`} className="h-6 w-6" height={24} src={`/api/cdn/upk/arts/potential_hub/potential_${data.potential}.png`} unoptimized width={24} />
                                        <span className="text-muted-foreground text-sm">+{data.potential}</span>
                                    </div>
                                </div>
                            </AccordionContent>
                        </AccordionItem>

                        {/* Skills */}
                        <AccordionItem className="border-b-0" value="skills">
                            <AccordionTrigger className="py-2 font-medium text-sm">Skills</AccordionTrigger>
                            <AccordionContent>
                                <ScrollArea className="max-h-45 w-full">
                                    {skillDisplayData.length > 0 ? (
                                        <div className="w-full space-y-2 overflow-hidden">
                                            {skillDisplayData.map((skill) => (
                                                <SkillItem index={skill.index} isDefaultSkill={data.default_skill === skill.index} key={skill.skillId} mainSkillLvl={data.skill_level} skillId={skill.skillId} skillStatic={skill.skillStatic} specializeLevel={skill.specializeLevel} size="small" />
                                            ))}
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
                                {moduleDisplayData.length > 0 ? (
                                    <div className="space-y-2">
                                        {moduleDisplayData.map((mod) => (
                                            <ModuleItem isEquipped={mod.isEquipped} key={mod.module.uniEquipId} module={mod.module} moduleLevel={mod.level} size="small" />
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-muted-foreground text-sm">No modules unlocked.</p>
                                )}
                            </AccordionContent>
                        </AccordionItem>
                    </Accordion>
                </CardContent>
            </Card>

            {/* Full Details Dialog */}
            <CharacterDialog data={data} operator={operator} operatorImage={operatorImage} operatorName={operatorName} operatorProfession={operatorProfession} starCount={starCount} stats={stats} />
        </MorphingDialog>
    );
}
