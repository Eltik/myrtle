"use client";

import { Heart, Shield, Sparkles, Sword, Zap } from "lucide-react";
import Image from "next/image";
import { memo } from "react";
import { MorphingDialog, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import { cn } from "~/lib/utils";
import type { DamageType, Enemy } from "~/types/api/impl/enemy";
import { DAMAGE_TYPE_COLORS, DAMAGE_TYPE_DISPLAY, HOVER_DELAY, LEVEL_COLORS, LEVEL_DISPLAY } from "./constants";
import { EnemyDetailDialog } from "./enemy-detail-dialog";
import { formatStatNumber, getEnemyMaxStats } from "./utils";

interface EnemyCardProps {
    enemy: Enemy;
    viewMode: "grid" | "list";
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const EnemyCard = memo(function EnemyCard({ enemy, viewMode, isHovered = false, shouldGrayscale = false, onHoverChange }: EnemyCardProps) {
    if (viewMode === "list") {
        return <EnemyCardList enemy={enemy} isHovered={isHovered} shouldGrayscale={shouldGrayscale} />;
    }

    return <EnemyCardGrid enemy={enemy} isHovered={isHovered} onHoverChange={onHoverChange} shouldGrayscale={shouldGrayscale} />;
});

// ============================================================================
// Grid View Card
// ============================================================================

function EnemyCardGrid({ enemy, isHovered = false, shouldGrayscale = false, onHoverChange }: { enemy: Enemy; isHovered?: boolean; shouldGrayscale?: boolean; onHoverChange?: (isOpen: boolean) => void }) {
    const levelColors = LEVEL_COLORS[enemy.enemyLevel] ?? LEVEL_COLORS.NORMAL;
    const stats = getEnemyMaxStats(enemy);
    const primaryDamageType = enemy.damageType?.[0] ?? "NO_DAMAGE";

    const cardContent = (
        <div className="group card-hover-transition relative flex aspect-2/3 cursor-pointer overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg">
            {/* Portrait */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-105", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                {enemy.portrait ? (
                    <Image alt={`${enemy.name} Portrait`} className="h-full w-full rounded-lg object-cover" decoding="async" fill loading="lazy" src={`/api/cdn${enemy.portrait}`} />
                ) : (
                    <div className="flex h-full w-full items-center justify-center bg-secondary/30">
                        <Skull className="h-12 w-12 text-muted-foreground/30" />
                    </div>
                )}
            </div>

            {/* Level indicator badge - top left */}
            <div className={cn("absolute top-1.5 left-1.5 rounded-md border px-1.5 py-0.5 font-semibold text-[10px] uppercase", levelColors.bg, levelColors.text, levelColors.border)}>{LEVEL_DISPLAY[enemy.enemyLevel]}</div>

            {/* Bottom info bar */}
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1.5 line-clamp-2 max-w-[85%] font-bold text-xs uppercase opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm">{enemy.name}</h2>
                    {/* Damage type indicator */}
                    <div className="card-hover-transition absolute right-1.5 bottom-1 flex scale-75 items-center opacity-0 group-hover:scale-100 group-hover:opacity-100">
                        <DamageTypeIcon damageType={primaryDamageType} size="sm" />
                    </div>
                    {/* Level color bar */}
                    <div className={cn("absolute bottom-0 h-0.5 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0", enemy.enemyLevel === "BOSS" ? "bg-amber-500" : enemy.enemyLevel === "ELITE" ? "bg-purple-500" : "bg-muted-foreground/30")} />
                    <div className={cn("absolute -bottom-0.5 h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0", enemy.enemyLevel === "BOSS" ? "bg-amber-500/60" : enemy.enemyLevel === "ELITE" ? "bg-purple-500/60" : "bg-muted-foreground/20")} />
                </div>
            </div>
        </div>
    );

    return (
        <MorphingDialog>
            <HoverCard closeDelay={50} onOpenChange={onHoverChange} openDelay={HOVER_DELAY}>
                <HoverCardTrigger asChild>
                    <MorphingDialogTrigger className="w-full text-left">{cardContent}</MorphingDialogTrigger>
                </HoverCardTrigger>
                <HoverCardContent className="w-80 p-4" side="top">
                    <EnemyHoverContent enemy={enemy} stats={stats} />
                </HoverCardContent>
            </HoverCard>
            <EnemyDetailDialog enemy={enemy} />
        </MorphingDialog>
    );
}

// ============================================================================
// List View Card
// ============================================================================

function EnemyCardList({ enemy, isHovered = false, shouldGrayscale = false }: { enemy: Enemy; isHovered?: boolean; shouldGrayscale?: boolean }) {
    const levelColors = LEVEL_COLORS[enemy.enemyLevel] ?? LEVEL_COLORS.NORMAL;
    const stats = getEnemyMaxStats(enemy);
    const primaryDamageType = enemy.damageType?.[0] ?? "NO_DAMAGE";
    const damageTypeColors = DAMAGE_TYPE_COLORS[primaryDamageType] ?? DAMAGE_TYPE_COLORS.NO_DAMAGE;

    const cardContent = (
        <div className={cn("group flex w-full cursor-pointer items-center gap-3 rounded-lg border border-border bg-card/30 px-3 py-2 transition-colors hover:bg-card/60", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
            {/* Portrait thumbnail */}
            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md bg-secondary/30">
                {enemy.portrait ? (
                    <Image alt={`${enemy.name}`} className="h-full w-full object-cover" fill sizes="48px" src={`/api/cdn${enemy.portrait}`} />
                ) : (
                    <div className="flex h-full w-full items-center justify-center">
                        <Skull className="h-6 w-6 text-muted-foreground/30" />
                    </div>
                )}
            </div>

            {/* Name */}
            <div className="min-w-0 flex-1 text-left">
                <h3 className="truncate font-semibold text-foreground text-sm">{enemy.name}</h3>
                <p className="truncate text-muted-foreground text-xs">{enemy.enemyId}</p>
            </div>

            {/* Level badge */}
            <div className={cn("w-20 shrink-0 rounded-md border px-2 py-1 text-center font-medium text-xs", levelColors.bg, levelColors.text, levelColors.border)}>{LEVEL_DISPLAY[enemy.enemyLevel]}</div>

            {/* Damage type */}
            <div className={cn("flex w-24 shrink-0 items-center gap-1.5 rounded-md px-2 py-1", damageTypeColors.bg)}>
                <DamageTypeIcon damageType={primaryDamageType} size="sm" />
                <span className={cn("text-xs", damageTypeColors.text)}>{DAMAGE_TYPE_DISPLAY[primaryDamageType]}</span>
            </div>

            {/* Stats - desktop only */}
            <div className="hidden w-16 shrink-0 text-right font-mono text-muted-foreground text-sm lg:block">{stats ? formatStatNumber(stats.maxHp) : "-"}</div>
            <div className="hidden w-16 shrink-0 text-right font-mono text-muted-foreground text-sm lg:block">{stats ? formatStatNumber(stats.atk) : "-"}</div>
            <div className="hidden w-16 shrink-0 text-right font-mono text-muted-foreground text-sm xl:block">{stats ? formatStatNumber(stats.def) : "-"}</div>
            <div className="hidden w-16 shrink-0 text-right font-mono text-muted-foreground text-sm xl:block">{stats ? `${stats.magicResistance}%` : "-"}</div>
        </div>
    );

    return (
        <MorphingDialog>
            <MorphingDialogTrigger className="w-full">{cardContent}</MorphingDialogTrigger>
            <EnemyDetailDialog enemy={enemy} />
        </MorphingDialog>
    );
}

// ============================================================================
// Hover Content
// ============================================================================

function EnemyHoverContent({ enemy, stats }: { enemy: Enemy; stats: ReturnType<typeof getEnemyMaxStats> }) {
    const levelColors = LEVEL_COLORS[enemy.enemyLevel] ?? LEVEL_COLORS.NORMAL;

    return (
        <div className="space-y-3">
            {/* Header */}
            <div className="flex items-start gap-3">
                {/* Portrait */}
                <div className="relative h-16 w-16 shrink-0 overflow-hidden rounded-md bg-secondary/30">
                    {enemy.portrait ? (
                        <Image alt={`${enemy.name}`} className="object-cover" fill src={`/api/cdn${enemy.portrait}`} />
                    ) : (
                        <div className="flex h-full w-full items-center justify-center">
                            <Skull className="h-8 w-8 text-muted-foreground/30" />
                        </div>
                    )}
                </div>
                <div className="min-w-0 flex-1 space-y-1">
                    <h4 className="font-semibold text-base">{enemy.name}</h4>
                    <div className="flex items-center gap-2">
                        <span className={cn("rounded border px-1.5 py-0.5 font-medium text-[10px] uppercase", levelColors.bg, levelColors.text, levelColors.border)}>{LEVEL_DISPLAY[enemy.enemyLevel]}</span>
                        {enemy.damageType?.map((dt) => (
                            <DamageTypeBadge damageType={dt} key={dt} />
                        ))}
                    </div>
                </div>
            </div>

            {/* Description */}
            {enemy.description && <p className="line-clamp-2 text-muted-foreground text-xs">{enemy.description}</p>}

            {/* Stats Grid */}
            {stats && (
                <div className="grid grid-cols-4 gap-2">
                    <StatBox icon={Heart} label="HP" value={formatStatNumber(stats.maxHp)} />
                    <StatBox icon={Sword} label="ATK" value={formatStatNumber(stats.atk)} />
                    <StatBox icon={Shield} label="DEF" value={formatStatNumber(stats.def)} />
                    <StatBox icon={Sparkles} label="RES" value={`${stats.magicResistance}%`} />
                </div>
            )}

            {/* Immunities */}
            {stats && (
                <div className="flex flex-wrap gap-1">
                    {stats.stunImmune && <ImmunityBadge label="Stun Immune" />}
                    {stats.silenceImmune && <ImmunityBadge label="Silence Immune" />}
                    {stats.sleepImmune && <ImmunityBadge label="Sleep Immune" />}
                    {stats.frozenImmune && <ImmunityBadge label="Freeze Immune" />}
                    {stats.levitateImmune && <ImmunityBadge label="Levitate Immune" />}
                </div>
            )}

            {/* Abilities */}
            {enemy.ability && (
                <div className="rounded-md bg-secondary/30 p-2">
                    <p className="text-muted-foreground text-xs">{enemy.ability}</p>
                </div>
            )}
        </div>
    );
}

// ============================================================================
// Helper Components
// ============================================================================

function StatBox({ icon: Icon, label, value }: { icon: typeof Heart; label: string; value: string }) {
    return (
        <div className="flex flex-col items-center rounded-md bg-secondary/30 p-2">
            <Icon className="mb-1 h-3 w-3 text-muted-foreground" />
            <span className="font-mono font-semibold text-foreground text-xs">{value}</span>
            <span className="text-[10px] text-muted-foreground">{label}</span>
        </div>
    );
}

function DamageTypeIcon({ damageType, size = "md" }: { damageType: DamageType; size?: "sm" | "md" }) {
    const iconSize = size === "sm" ? "h-3.5 w-3.5" : "h-4 w-4";
    const colors = DAMAGE_TYPE_COLORS[damageType];

    switch (damageType) {
        case "PHYSIC":
            return <Sword className={cn(iconSize, colors.text)} />;
        case "MAGIC":
            return <Sparkles className={cn(iconSize, colors.text)} />;
        case "HEAL":
            return <Heart className={cn(iconSize, colors.text)} />;
        default:
            return <Zap className={cn(iconSize, colors.text)} />;
    }
}

function DamageTypeBadge({ damageType }: { damageType: DamageType }) {
    const colors = DAMAGE_TYPE_COLORS[damageType];

    return (
        <span className={cn("flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px]", colors.bg, colors.text)}>
            <DamageTypeIcon damageType={damageType} size="sm" />
            {DAMAGE_TYPE_DISPLAY[damageType]}
        </span>
    );
}

function ImmunityBadge({ label }: { label: string }) {
    return <span className="rounded bg-destructive/10 px-1.5 py-0.5 text-[10px] text-destructive">{label}</span>;
}

// Skull icon component for fallback
function Skull({ className }: { className?: string }) {
    return (
        <svg aria-label="Unknown enemy" className={className} fill="none" role="img" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <circle cx="9" cy="12" r="1" />
            <circle cx="15" cy="12" r="1" />
            <path d="M8 20v2h8v-2" />
            <path d="m12.5 17-.5-1-.5 1h1z" />
            <path d="M16 20a2 2 0 0 0 1.56-3.25 8 8 0 1 0-11.12 0A2 2 0 0 0 8 20" />
        </svg>
    );
}
