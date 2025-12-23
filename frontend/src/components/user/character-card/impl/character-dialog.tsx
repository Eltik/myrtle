"use client";

import { motion, useScroll, useTransform } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent } from "~/components/ui/motion-primitives/morphing-dialog";
import { Separator } from "~/components/ui/shadcn/separator";
import type { CharacterData, CharacterStatic } from "~/types/api/impl/user";
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
            <span className="font-medium text-sm tabular-nums">
                <AnimatedNumber springOptions={{ bounce: 0, duration: 400 }} value={value} />
            </span>
        </div>
    );
}

interface CharacterDialogProps {
    data: CharacterData;
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
                            <Image alt={operatorName} className="h-full w-full object-contain object-top" height={512} src={operatorImage || "/placeholder.svg"} unoptimized width={512} />
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
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Stats</h3>
                                <Separator className="flex-1" />
                            </div>
                            <AnimatedGroup className="grid grid-cols-2 gap-1.5 sm:grid-cols-3" preset="blur-slide">
                                <StatItem label="HP" value={stats.maxHp} />
                                <StatItem label="ATK" value={stats.atk} />
                                <StatItem label="DEF" value={stats.def} />
                                <StatItem label="RES" value={stats.magicResistance} />
                                <StatItem label="DP Cost" value={stats.cost} />
                                <StatItem label="Block" value={stats.blockCnt} />
                            </AnimatedGroup>
                        </div>
                    )}

                    {/* Skills */}
                    <div className="mb-6">
                        <h3 className="mb-3 font-semibold text-lg">Skills (Lv.{data.mainSkillLvl})</h3>
                        <div className="space-y-3">
                            {data.skills && data.skills.length > 0 ? (
                                data.skills.map((skill, index) => <SkillItem index={index} isDefaultSkill={data.defaultSkillIndex === index} key={skill.skillId} mainSkillLvl={data.mainSkillLvl} size="large" skill={skill} />)
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

                                            return <ModuleItem isEquipped={isEquipped} key={module.uniEquipId} module={module} moduleLevel={moduleLevel} size="large" />;
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
    );
}
