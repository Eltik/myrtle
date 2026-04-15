"use client";

import { motion, useScroll, useTransform } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { ImageWithSkeleton } from "~/components/ui/image-with-skeleton";
import { MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent } from "~/components/ui/motion-primitives/morphing-dialog";
import { Separator } from "~/components/ui/shadcn/separator";
import type { CharacterStatic, EnrichedRosterEntry } from "~/types/api/impl/user";
import { getTrustPercent } from "./helpers";
import { ModuleItem } from "./module-item";
import { SkillItem } from "./skill-item";

interface StatItemProps {
    label: string;
    value: number;
}

function StatItem({ label, value }: StatItemProps) {
    return (
        <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
            <span className="text-muted-foreground text-xs">{label}</span>
            <span className="font-medium text-sm tabular-nums">{value}</span>
        </div>
    );
}

interface CharacterDialogProps {
    data: EnrichedRosterEntry;
    operator: CharacterStatic | null;
    operatorName: string;
    operatorProfession: string;
    operatorImage: string;
    starCount: number;
    stats: {
        maxHp: number;
        atk: number;
        def: number;
        magicResistance: number;
        cost: number;
        blockCnt: number;
    } | null;
}

export function CharacterDialog({ data, operator, operatorName, operatorProfession, operatorImage, starCount, stats }: CharacterDialogProps) {
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

    // Build module display data
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

    return (
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
                            <ImageWithSkeleton alt={operatorName} className="h-full w-full object-contain object-top" containerClassName="h-full w-full" height={512} priority skeletonClassName="rounded-lg" src={operatorImage || "/placeholder.svg"} unoptimized width={512} />
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
                                <Image alt={`${starCount} Star`} className="h-6 w-auto object-contain" height={24} loading="eager" src={`/api/cdn/upk/arts/rarity_hub/rarity_yellow_${starCount - 1}.png`} unoptimized width={80} />
                                <span className="rounded bg-black/50 px-2 py-0.5 text-sm text-white">{operatorProfession}</span>
                            </div>
                        </motion.div>
                    </div>

                    {/* Quick Stats */}
                    <div className="mb-5 grid grid-cols-4 gap-1.5">
                        <div className="flex items-center justify-center rounded-md bg-muted/30 py-2">
                            <Image alt={`Elite ${data.elite}`} className="icon-theme-aware h-6 w-6 object-contain" height={24} loading="eager" src={`/api/cdn/upk/arts/elite_hub/elite_${data.elite}.png`} unoptimized width={24} />
                        </div>
                        <div className="flex flex-col items-center justify-center rounded-md bg-muted/30 px-2.5 py-1.5 sm:flex-row sm:justify-between">
                            <span className="text-[0.625rem] text-muted-foreground sm:text-xs">Lvl</span>
                            <span className="font-medium text-sm tabular-nums">{data.level}</span>
                        </div>
                        <div className="flex items-center justify-center rounded-md bg-muted/30 py-2">
                            <Image alt={`Potential ${data.potential + 1}`} className="h-6 w-6 object-contain" height={24} loading="eager" src={`/api/cdn/upk/arts/potential_hub/potential_${data.potential}.png`} unoptimized width={24} />
                        </div>
                        <div className="flex flex-col items-center justify-center rounded-md bg-muted/30 px-2.5 py-1.5 sm:flex-row sm:justify-between">
                            <span className="text-[0.625rem] text-muted-foreground sm:text-xs">Trust</span>
                            <span className="font-medium text-sm tabular-nums">{getTrustPercent(data.favor_point ?? 0)}%</span>
                        </div>
                    </div>

                    {/* Battle Stats */}
                    {stats && (
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Stats</h3>
                                <Separator className="flex-1" />
                            </div>
                            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                                <StatItem label="HP" value={stats.maxHp} />
                                <StatItem label="ATK" value={stats.atk} />
                                <StatItem label="DEF" value={stats.def} />
                                <StatItem label="RES" value={stats.magicResistance} />
                                <StatItem label="DP Cost" value={stats.cost} />
                                <StatItem label="Block" value={stats.blockCnt} />
                            </div>
                        </div>
                    )}

                    {/* Skills */}
                    <div className="mb-5">
                        <div className="mb-2.5 flex items-center gap-2">
                            <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Skills</h3>
                            <span className="text-muted-foreground/60 text-xs">Lv.{data.skill_level}</span>
                            <Separator className="flex-1" />
                        </div>
                        {skillDisplayData.length > 0 ? (
                            <div className="space-y-1.5">
                                {skillDisplayData.map((skill) => (
                                    <SkillItem index={skill.index} isDefaultSkill={data.default_skill === skill.index} key={skill.skillId} mainSkillLvl={data.skill_level} skillId={skill.skillId} skillStatic={skill.skillStatic} specializeLevel={skill.specializeLevel} size="large" />
                                ))}
                            </div>
                        ) : (
                            <p className="text-muted-foreground text-xs">No skills found.</p>
                        )}
                    </div>

                    {/* Modules */}
                    <div className="mb-5">
                        <div className="mb-2.5 flex items-center gap-2">
                            <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Modules</h3>
                            <Separator className="flex-1" />
                        </div>
                        {moduleDisplayData.length > 0 ? (
                            <div className="space-y-1.5">
                                {moduleDisplayData.map((mod) => (
                                    <ModuleItem isEquipped={mod.isEquipped} key={mod.module.uniEquipId} module={mod.module} moduleLevel={mod.level} size="large" />
                                ))}
                            </div>
                        ) : operator?.modules && operator.modules.some((m) => m.typeName1 !== "ORIGINAL") ? (
                            <p className="text-muted-foreground text-xs">No modules unlocked.</p>
                        ) : (
                            <p className="text-muted-foreground text-xs">No modules available.</p>
                        )}
                    </div>

                    {/* Info */}
                    <div>
                        <div className="mb-2.5 flex items-center gap-2">
                            <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Info</h3>
                            <Separator className="flex-1" />
                        </div>
                        <div className="grid grid-cols-2 gap-1.5">
                            <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                <span className="text-muted-foreground text-xs">Recruited</span>
                                <span className="font-medium text-sm">{data.obtained_at ? new Date(data.obtained_at * 1000).toLocaleDateString() : "Unknown"}</span>
                            </div>
                            <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                <span className="text-muted-foreground text-xs">Voice</span>
                                <span className="font-medium text-sm">{data.voice_lan === "JP" ? "Japanese" : data.voice_lan === "CN_MANDARIN" ? "Chinese" : data.voice_lan === "EN" ? "English" : data.voice_lan === "KR" ? "Korean" : (data.voice_lan?.toLowerCase().replace("_", " ") ?? "Japanese")}</span>
                            </div>
                        </div>
                    </div>
                </div>
                <MorphingDialogClose className="text-muted-foreground hover:text-foreground" />
            </MorphingDialogContent>
        </MorphingDialogContainer>
    );
}
