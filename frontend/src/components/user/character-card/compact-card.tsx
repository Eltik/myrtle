"use client";

import Image from "next/image";
import { useCallback, useEffect, useRef } from "react";
import { RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { ImageWithSkeleton } from "~/components/ui/image-with-skeleton";
import { MorphingDialog, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { useCDNPrefetch } from "~/hooks/use-cdn-prefetch";
import { formatProfession, getRarityStarCount } from "~/lib/utils";
import type { CharacterData, CharacterStatic } from "~/types/api/impl/user";
import { CharacterDialog } from "./impl/character-dialog";
import { getAttributeStats } from "./impl/helpers";

interface CompactCharacterCardProps {
    data: CharacterData;
}

// Max level by rarity for each elite phase [E0, E1, E2]
const MAX_LEVEL_BY_RARITY: Record<number, number[]> = {
    6: [50, 80, 90],
    5: [50, 70, 80],
    4: [45, 60, 70],
    3: [40, 55, 55],
    2: [30, 30, 30],
    1: [30, 30, 30],
};

// Max elite phase by rarity
const MAX_ELITE_BY_RARITY: Record<number, number> = {
    6: 2,
    5: 2,
    4: 2,
    3: 1,
    2: 0,
    1: 0,
};

/**
 * Gets the avatar URL for an operator based on their current skin.
 * Uses the /api/cdn/avatar/ endpoint which the backend resolves.
 */
function getOperatorAvatarUrl(charId: string, skin: string, _evolvePhase: number, currentTmpl?: string | null, tmpl?: Record<string, { skinId: string }> | null): string {
    let skinId = skin;

    if (currentTmpl && tmpl && tmpl[currentTmpl]) {
        skinId = tmpl[currentTmpl].skinId;
    }

    if (!skinId) {
        return `/api/cdn/avatar/${charId}`;
    }

    if (skinId.includes("@")) {
        const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return `/api/cdn/avatar/${normalizedSkinId}`;
    }

    if (skinId.endsWith("#1") || skinId.endsWith("_1")) {
        return `/api/cdn/avatar/${charId}`;
    }

    if (skinId.endsWith("#2") || skinId.endsWith("_2")) {
        const normalizedSkinId = skinId.replaceAll("#", "_");
        return `/api/cdn/avatar/${normalizedSkinId}`;
    }

    const normalizedSkinId = skinId.replaceAll("#", "_");
    return `/api/cdn/avatar/${normalizedSkinId}`;
}

/**
 * Gets the full portrait URL for the dialog.
 */
function getOperatorPortraitUrl(charId: string, skin: string, evolvePhase: number, currentTmpl?: string | null, tmpl?: Record<string, { skinId: string }> | null): string {
    let skinId = skin;

    if (currentTmpl && tmpl && tmpl[currentTmpl]) {
        skinId = tmpl[currentTmpl].skinId;
    }

    if (!skinId) {
        const suffix = evolvePhase >= 2 ? "_2" : "_1";
        return `/api/cdn/upk/chararts/${charId}/${charId}${suffix}.png`;
    }

    if (skinId.includes("@")) {
        const normalizedSkinId = skinId.replaceAll("@", "_").replaceAll("#", "%23");
        return `/api/cdn/upk/skinpack/${charId}/${normalizedSkinId}.png`;
    }

    const normalizedSkinId = skinId.replaceAll("#", "_");
    return `/api/cdn/upk/chararts/${charId}/${normalizedSkinId}.png`;
}

export function CompactCharacterCard({ data }: CompactCharacterCardProps) {
    const operator = data.static as CharacterStatic | null;
    const cardWrapperRef = useRef<HTMLDivElement>(null);
    const hoverTimeoutRef = useRef<NodeJS.Timeout | null>(null);
    const hasPreloadedRef = useRef(false);
    const { prefetch } = useCDNPrefetch();

    const operatorName = operator?.name ?? "Unknown Operator";
    const operatorRarity = operator?.rarity ?? "TIER_1";
    const starCount = getRarityStarCount(operatorRarity);
    const rarityColor = RARITY_COLORS[starCount] ?? "#ffffff";

    const stats = getAttributeStats(data, operator);

    const preloadDialogImages = useCallback(() => {
        if (hasPreloadedRef.current) return;
        hasPreloadedRef.current = true;

        const portraitUrl = getOperatorPortraitUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);

        const imagesToPreload: string[] = [portraitUrl, `/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`, `/api/cdn/upk/arts/elite_hub/elite_${data.evolvePhase}.png`, `/api/cdn/upk/arts/potential_hub/potential_${data.potentialRank}.png`];

        for (const skill of data.skills) {
            const skillStatic = skill.static as { iconId?: string; skillId?: string; image?: string } | null;
            const skillIcon = skillStatic?.image ? `/api/cdn${skillStatic.image}` : `/api/cdn/upk/spritepack/skill_icons_0/skill_icon_${skillStatic?.iconId ?? skillStatic?.skillId ?? skill.skillId}.png`;
            imagesToPreload.push(skillIcon);

            if (skill.specializeLevel > 0) {
                imagesToPreload.push(`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`);
            }
        }

        if (operator?.modules) {
            for (const module of operator.modules) {
                const equipData = data.equip[module.uniEquipId];
                const moduleLevel = equipData?.level ?? 0;
                const isLocked = equipData?.locked === 1;

                if (module.typeName1 !== "ORIGINAL" && moduleLevel > 0 && !isLocked) {
                    const moduleImage = module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`;
                    imagesToPreload.push(moduleImage);
                }
            }
        }

        prefetch(imagesToPreload, "high");
    }, [data, operator, starCount, prefetch]);

    const handleMouseEnter = useCallback(() => {
        hoverTimeoutRef.current = setTimeout(preloadDialogImages, 150);
    }, [preloadDialogImages]);

    const handleMouseLeave = useCallback(() => {
        if (hoverTimeoutRef.current) {
            clearTimeout(hoverTimeoutRef.current);
            hoverTimeoutRef.current = null;
        }
    }, []);

    useEffect(() => {
        const element = cardWrapperRef.current;
        if (!element) return;

        element.addEventListener("mouseenter", handleMouseEnter);
        element.addEventListener("mouseleave", handleMouseLeave);

        return () => {
            element.removeEventListener("mouseenter", handleMouseEnter);
            element.removeEventListener("mouseleave", handleMouseLeave);
        };
    }, [handleMouseEnter, handleMouseLeave]);

    const reg = /( the )|\(/gi;
    const splitName = operatorName.replace(/\)$/, "").split(reg);
    const displayName = splitName[0] ?? operatorName;
    const subtitle = splitName[2] ?? null;
    const nameIsLong = displayName.split(" ").length > 1 && displayName.length >= 16;

    const isMaxed = (() => {
        const maxElite = MAX_ELITE_BY_RARITY[starCount] ?? 2;
        const maxLevel = MAX_LEVEL_BY_RARITY[starCount]?.[maxElite] ?? 90;

        if (data.evolvePhase !== maxElite) return false;
        if (data.level !== maxLevel) return false;
        if (starCount > 2 && data.mainSkillLvl !== 7) return false;

        if (maxElite === 2) {
            if (!data.skills.every((skill) => skill.specializeLevel === 3)) return false;
            const unlockedModules =
                operator?.modules?.filter((mod) => {
                    const equipData = data.equip[mod.uniEquipId];
                    return mod.typeName1 !== "ORIGINAL" && equipData && equipData.level > 0 && equipData.locked !== 1;
                }) ?? [];
            if (unlockedModules.length > 0 && !unlockedModules.every((mod) => data.equip[mod.uniEquipId]?.level === 3)) return false;
        }

        if (data.potentialRank !== 5) return false;
        return true;
    })();

    const avatarUrl = getOperatorAvatarUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);
    const portraitUrl = getOperatorPortraitUrl(data.charId, data.skin, data.evolvePhase, data.currentTmpl, data.tmpl as Record<string, { skinId: string }> | null);

    const unlockedModules =
        operator?.modules?.filter((module) => {
            const equipData = data.equip[module.uniEquipId];
            const moduleLevel = equipData?.level ?? 0;
            const isLocked = equipData?.locked === 1;
            return module.typeName1 !== "ORIGINAL" && moduleLevel > 0 && !isLocked;
        }) ?? [];

    const maxLevelForPhase = MAX_LEVEL_BY_RARITY[starCount]?.[data.evolvePhase] ?? 90;
    const isAtMaxLevel = data.level === maxLevelForPhase;

    return (
        <div ref={cardWrapperRef}>
            <MorphingDialog transition={{ type: "spring", bounce: 0.05, duration: 0.25 }}>
                <MorphingDialogTrigger className="block">
                    {/* Main Container */}
                    <div
                        className="fade-in slide-in-from-bottom-2 relative flex h-min w-min animate-in cursor-pointer flex-col rounded bg-card transition-transform hover:scale-102"
                        style={{
                            padding: "4px 8px 4px 6px",
                            margin: "2px 4px 4px 10px",
                            boxShadow: isMaxed ? `0 0 10px ${rarityColor}, 0 0 20px ${rarityColor}80` : "0 1px 3px 0 rgb(0 0 0 / 0.1)",
                        }}
                    >
                        {/* Operator Name - Top */}
                        <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                            {subtitle && <span className="text-1.75 text-foreground leading-1.5 sm:text-2.25 sm:leading-2">{subtitle}</span>}
                            <span
                                className="text-foreground"
                                style={{
                                    fontSize: nameIsLong ? "9px" : "12px",
                                    lineHeight: nameIsLong ? "9px" : "17px",
                                }}
                            >
                                {displayName}
                            </span>
                        </div>

                        {/* Avatar Container with bottom rarity border */}
                        <div
                            className="relative box-content aspect-square h-20 sm:h-30"
                            style={{
                                borderBottom: `4px solid ${rarityColor}`,
                                filter: isMaxed ? "drop-shadow(0px 0px 8px rgba(255,255,255,0.3))" : undefined,
                            }}
                        >
                            <ImageWithSkeleton alt={operatorName} className="h-full w-full object-contain" containerClassName="h-full w-full" height={120} src={avatarUrl} unoptimized width={120} />

                            {/* Maxed Badge */}
                            {isMaxed && (
                                <div className="absolute -bottom-1 left-0 rounded-t px-1 font-medium text-xs sm:text-sm" style={{ backgroundColor: rarityColor, color: "#121212" }}>
                                    Maxed
                                </div>
                            )}
                        </div>

                        {/* Left Side Badges */}
                        {!isMaxed && (
                            <div className="absolute -bottom-2 -left-3 z-10 flex flex-col gap-0.5">
                                {/* Potential Badge */}
                                {data.potentialRank > 0 && (
                                    <div className="relative mb-0.5 ml-1 h-4 w-3 sm:mb-2 sm:h-6 sm:w-5">
                                        <Image alt={`Potential ${data.potentialRank + 1}`} className="h-full w-full object-contain" height={24} src={`/api/cdn/upk/arts/potential_hub/potential_${data.potentialRank}.png`} unoptimized width={20} />
                                    </div>
                                )}

                                {/* Elite Badge */}
                                {data.evolvePhase > 0 && (
                                    <div className="mb-0 h-5 w-5 sm:mb-1 sm:h-8 sm:w-8">
                                        <Image alt={`Elite ${data.evolvePhase}`} className="icon-theme-aware h-full w-full object-contain" height={32} src={`/api/cdn/upk/arts/elite_hub/elite_${data.evolvePhase}.png`} unoptimized width={32} />
                                    </div>
                                )}

                                {/* Level Circle */}
                                {(data.evolvePhase > 0 || data.level > 1) && (
                                    <div
                                        className="flex aspect-square h-8 flex-col items-center justify-center rounded-full border-2 bg-secondary text-lg leading-none sm:h-12 sm:text-2xl"
                                        style={{
                                            borderColor: isAtMaxLevel ? "hsl(var(--primary))" : "hsl(var(--muted-foreground))",
                                        }}
                                    >
                                        <abbr className="hidden text-2.25 leading-1 no-underline sm:flex" title="Level">
                                            LV
                                        </abbr>
                                        {data.level}
                                    </div>
                                )}
                            </div>
                        )}

                        {/* Right Side - Skills */}
                        {!isMaxed && data.mainSkillLvl > 1 && (
                            <div className="absolute top-8 right-0 z-10 flex flex-col gap-0.5 sm:-right-3">
                                {data.skills.map((skill, idx) => {
                                    if (data.evolvePhase < idx) return null;

                                    const hasM = skill.specializeLevel > 0;
                                    return (
                                        <div className="relative flex h-4 w-4 items-center justify-center sm:h-6 sm:w-6" key={skill.skillId} style={{ marginLeft: `${idx * 4}px` }} title={hasM ? `Skill ${idx + 1}: M${skill.specializeLevel}` : `Skill ${idx + 1}: Lv.${data.mainSkillLvl}`}>
                                            {hasM ? (
                                                <Image alt={`M${skill.specializeLevel}`} className="h-full w-full object-contain" height={24} src={`/api/cdn/upk/arts/specialized_hub/specialized_${skill.specializeLevel}.png`} unoptimized width={24} />
                                            ) : (
                                                <div className="flex h-full w-full items-center justify-center rounded bg-secondary font-bold text-2.5 text-secondary-foreground sm:text-xs">{data.mainSkillLvl}</div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        )}

                        {/* Bottom Right - Modules */}
                        {!isMaxed && unlockedModules.length > 0 && (
                            <div className="absolute -right-2 -bottom-3 z-10 flex flex-row-reverse gap-1">
                                {unlockedModules.map((module) => {
                                    const moduleLevel = data.equip[module.uniEquipId]?.level ?? 0;
                                    const typeLetter = module.typeName1?.slice(-1) ?? "X";

                                    return (
                                        <div className="relative aspect-square h-6 overflow-hidden rounded bg-secondary sm:h-8" key={module.uniEquipId} title={`${module.typeName1} Stage ${moduleLevel}`}>
                                            <Image alt={module.typeName1 ?? "Module"} className="h-full w-full object-contain" height={32} src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`} unoptimized width={32} />
                                            <span className="absolute right-0 bottom-0 rounded-tl bg-secondary/90 px-0.5 font-medium text-2.25 leading-none sm:text-2.5">
                                                {typeLetter}
                                                {moduleLevel}
                                            </span>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </MorphingDialogTrigger>

                {/* Full Details Dialog */}
                <CharacterDialog data={data} operator={operator} operatorImage={portraitUrl} operatorName={operatorName} operatorProfession={formatProfession(operator?.profession ?? "")} starCount={starCount} stats={stats} />
            </MorphingDialog>
        </div>
    );
}
