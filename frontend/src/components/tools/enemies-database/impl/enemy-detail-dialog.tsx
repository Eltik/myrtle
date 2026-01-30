"use client";

import { motion, useScroll, useTransform } from "motion/react";
import { useState } from "react";
import { ImageWithSkeleton } from "~/components/ui/image-with-skeleton";
import { MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent } from "~/components/ui/motion-primitives/morphing-dialog";
import { Separator } from "~/components/ui/shadcn/separator";
import { cn } from "~/lib/utils";
import type { DamageType, Enemy, EnemyLevelStats } from "~/types/api/impl/enemy";
import { DAMAGE_TYPE_COLORS, DAMAGE_TYPE_DISPLAY, LEVEL_COLORS, LEVEL_DISPLAY } from "./constants";
import { formatStatNumber, getEnemyMaxStats } from "./utils";

// ============================================================================
// Types
// ============================================================================

interface EnemyDetailDialogProps {
    enemy: Enemy;
}

// ============================================================================
// Main Dialog Component
// ============================================================================

export function EnemyDetailDialog({ enemy }: EnemyDetailDialogProps) {
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

    const levelColors = LEVEL_COLORS[enemy.enemyLevel] ?? LEVEL_COLORS.NORMAL;
    const stats = getEnemyMaxStats(enemy);
    const levelData = enemy.stats?.levels ?? [];

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
                            {enemy.portrait ? (
                                <ImageWithSkeleton alt={enemy.name} className="h-full w-full object-contain object-center" containerClassName="h-full w-full bg-secondary/20" height={512} priority skeletonClassName="rounded-lg" src={`/api/cdn${enemy.portrait}`} unoptimized width={512} />
                            ) : (
                                <div className="flex h-full w-full items-center justify-center bg-secondary/30">
                                    <SkullIcon className="h-24 w-24 text-muted-foreground/30" />
                                </div>
                            )}
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
                            <h2 className="font-bold text-2xl text-white sm:text-3xl">{enemy.name}</h2>
                            <div className="mt-2 flex flex-wrap items-center gap-2">
                                <span className={cn("rounded border px-2 py-0.5 font-semibold text-sm uppercase", levelColors.bg, levelColors.text, levelColors.border)}>{LEVEL_DISPLAY[enemy.enemyLevel]}</span>
                                {enemy.damageType?.map((dt) => (
                                    <DamageTypeBadge damageType={dt} key={dt} />
                                ))}
                            </div>
                        </motion.div>
                    </div>

                    {/* Description */}
                    {enemy.description && (
                        <div className="mb-5">
                            <p className="text-muted-foreground text-sm leading-relaxed">{enemy.description}</p>
                        </div>
                    )}

                    {/* Battle Stats */}
                    {stats && (
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Combat Stats</h3>
                                <span className="text-muted-foreground/60 text-xs">(Max Level)</span>
                                <Separator className="flex-1" />
                            </div>
                            <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-4">
                                <StatItem icon="heart" label="HP" value={formatStatNumber(stats.maxHp)} />
                                <StatItem icon="sword" label="ATK" value={formatStatNumber(stats.atk)} />
                                <StatItem icon="shield" label="DEF" value={formatStatNumber(stats.def)} />
                                <StatItem icon="sparkles" label="RES" value={`${stats.magicResistance}%`} />
                            </div>
                            <div className="mt-1.5 grid grid-cols-2 gap-1.5 sm:grid-cols-4">
                                <StatItem label="Move Speed" value={stats.moveSpeed.toFixed(1)} />
                                <StatItem label="Attack Speed" value={stats.attackSpeed.toString()} />
                                <StatItem label="Weight" value={stats.massLevel.toString()} />
                                <StatItem label="HP Regen" value={stats.hpRecoveryPerSec.toFixed(1)} />
                            </div>
                        </div>
                    )}

                    {/* Immunities */}
                    {stats && (
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Immunities</h3>
                                <Separator className="flex-1" />
                            </div>
                            <div className="flex flex-wrap gap-2">
                                <ImmunityIndicator immune={stats.stunImmune} label="Stun" />
                                <ImmunityIndicator immune={stats.silenceImmune} label="Silence" />
                                <ImmunityIndicator immune={stats.sleepImmune} label="Sleep" />
                                <ImmunityIndicator immune={stats.frozenImmune} label="Freeze" />
                                <ImmunityIndicator immune={stats.levitateImmune} label="Levitate" />
                            </div>
                        </div>
                    )}

                    {/* Abilities */}
                    {(enemy.ability || enemy.abilityList.length > 0) && (
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Abilities</h3>
                                <Separator className="flex-1" />
                            </div>
                            <div className="space-y-2">
                                {enemy.ability && (
                                    <div className="rounded-md bg-secondary/30 p-3">
                                        <p className="text-foreground text-sm">{enemy.ability}</p>
                                    </div>
                                )}
                                {enemy.abilityList.map((ability) => (
                                    <div className="rounded-md bg-secondary/30 p-3" key={ability.text}>
                                        <p className="text-foreground text-sm">{ability.text}</p>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Level Stats Table */}
                    {levelData.length > 1 && (
                        <div className="mb-5">
                            <div className="mb-2.5 flex items-center gap-2">
                                <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Stats by Level</h3>
                                <Separator className="flex-1" />
                            </div>
                            <div className="overflow-x-auto">
                                <table className="w-full text-sm">
                                    <thead>
                                        <tr className="border-border border-b text-muted-foreground">
                                            <th className="px-2 py-2 text-left font-medium text-xs">Level</th>
                                            <th className="px-2 py-2 text-right font-medium text-xs">HP</th>
                                            <th className="px-2 py-2 text-right font-medium text-xs">ATK</th>
                                            <th className="px-2 py-2 text-right font-medium text-xs">DEF</th>
                                            <th className="px-2 py-2 text-right font-medium text-xs">RES</th>
                                            <th className="hidden px-2 py-2 text-right font-medium text-xs sm:table-cell">Life</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {levelData.map((level) => (
                                            <LevelStatsRow key={level.level} level={level} />
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}

                    {/* Info */}
                    <div>
                        <div className="mb-2.5 flex items-center gap-2">
                            <h3 className="font-medium text-muted-foreground text-xs uppercase tracking-wide">Info</h3>
                            <Separator className="flex-1" />
                        </div>
                        <div className="grid grid-cols-2 gap-1.5">
                            <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                <span className="text-muted-foreground text-xs">ID</span>
                                <span className="font-medium font-mono text-sm">{enemy.enemyId}</span>
                            </div>
                            <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                <span className="text-muted-foreground text-xs">Index</span>
                                <span className="font-medium font-mono text-sm">{enemy.enemyIndex}</span>
                            </div>
                            {enemy.linkEnemies.length > 0 && (
                                <div className="col-span-2 flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
                                    <span className="text-muted-foreground text-xs">Linked Enemies</span>
                                    <span className="font-medium font-mono text-sm">{enemy.linkEnemies.join(", ")}</span>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
                <MorphingDialogClose className="text-muted-foreground hover:text-foreground" />
            </MorphingDialogContent>
        </MorphingDialogContainer>
    );
}

// ============================================================================
// Helper Components
// ============================================================================

function StatItem({ icon, label, value }: { icon?: string; label: string; value: string }) {
    return (
        <div className="flex items-center justify-between rounded-md bg-muted/30 px-2.5 py-1.5">
            <span className="flex items-center gap-1.5 text-muted-foreground text-xs">
                {icon && <StatIcon icon={icon} />}
                {label}
            </span>
            <span className="font-medium text-sm tabular-nums">{value}</span>
        </div>
    );
}

function StatIcon({ icon }: { icon: string }) {
    const iconClass = "h-3 w-3 text-muted-foreground";

    switch (icon) {
        case "heart":
            return (
                <svg aria-hidden="true" className={cn(iconClass, "text-rose-400")} fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24">
                    <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z" />
                </svg>
            );
        case "sword":
            return (
                <svg aria-hidden="true" className={cn(iconClass, "text-orange-400")} fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24">
                    <polyline points="14.5 17.5 3 6 3 3 6 3 17.5 14.5" />
                    <line x1="13" x2="19" y1="19" y2="13" />
                    <line x1="16" x2="20" y1="16" y2="20" />
                    <line x1="19" x2="21" y1="21" y2="19" />
                </svg>
            );
        case "shield":
            return (
                <svg aria-hidden="true" className={cn(iconClass, "text-sky-400")} fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24">
                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" />
                </svg>
            );
        case "sparkles":
            return (
                <svg aria-hidden="true" className={cn(iconClass, "text-purple-400")} fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24">
                    <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" />
                    <path d="M5 3v4" />
                    <path d="M19 17v4" />
                    <path d="M3 5h4" />
                    <path d="M17 19h4" />
                </svg>
            );
        default:
            return null;
    }
}

function DamageTypeBadge({ damageType }: { damageType: DamageType }) {
    const colors = DAMAGE_TYPE_COLORS[damageType];

    return <span className={cn("rounded px-2 py-0.5 text-sm", colors.bg, colors.text)}>{DAMAGE_TYPE_DISPLAY[damageType]}</span>;
}

function ImmunityIndicator({ label, immune }: { label: string; immune: boolean }) {
    return (
        <div className={cn("flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs", immune ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-400" : "border-border bg-muted/30 text-muted-foreground")}>
            <span className={cn("h-2.5 w-2.5 rounded-full", immune ? "bg-emerald-400" : "border border-muted-foreground/50")} />
            {label}
        </div>
    );
}

function LevelStatsRow({ level }: { level: EnemyLevelStats }) {
    const attrs = level.attributes;
    return (
        <tr className="border-border/50 border-b transition-colors hover:bg-muted/20">
            <td className="px-2 py-2 font-medium">{level.level}</td>
            <td className="px-2 py-2 text-right font-mono text-muted-foreground">{formatStatNumber(attrs.maxHp)}</td>
            <td className="px-2 py-2 text-right font-mono text-muted-foreground">{formatStatNumber(attrs.atk)}</td>
            <td className="px-2 py-2 text-right font-mono text-muted-foreground">{formatStatNumber(attrs.def)}</td>
            <td className="px-2 py-2 text-right font-mono text-muted-foreground">{attrs.magicResistance}%</td>
            <td className="hidden px-2 py-2 text-right font-mono text-muted-foreground sm:table-cell">{level.lifePointReduce}</td>
        </tr>
    );
}

function SkullIcon({ className }: { className?: string }) {
    return (
        <svg aria-label="Unknown enemy" className={className} fill="none" role="img" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24">
            <circle cx="9" cy="12" r="1" />
            <circle cx="15" cy="12" r="1" />
            <path d="M8 20v2h8v-2" />
            <path d="m12.5 17-.5-1-.5 1h1z" />
            <path d="M16 20a2 2 0 0 0 1.56-3.25 8 8 0 1 0-11.12 0A2 2 0 0 0 8 20" />
        </svg>
    );
}
