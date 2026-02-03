"use client";

import Image from "next/image";
import { useTheme } from "next-themes";
import { memo, useMemo, useState } from "react";
import { Check, Shield, Swords, X } from "lucide-react";
import { MorphingDialogClose, MorphingDialogDescription, MorphingDialogTitle } from "~/components/ui/motion-primitives/morphing-dialog";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Badge } from "~/components/ui/shadcn/badge";
import { cn } from "~/lib/utils";
import { descriptionToHtml } from "~/lib/description-parser";
import type { AbilityInfo, Enemy } from "~/types/api";
import { APPLY_WAY_DISPLAY, DAMAGE_TYPE_DISPLAY, ENEMY_LEVEL_DISPLAY, LEVEL_BAR_COLORS, LEVEL_TEXT_COLORS, LEVEL_TEXT_COLORS_LIGHT } from "../../constants";
import { EnemyLevelLogo } from "./enemy-level-logo";

interface EnemyDetailDialogProps {
    enemy: Enemy;
}

interface GroupedAbilities {
    title: string | null;
    items: AbilityInfo[];
}

function groupAbilities(abilities: AbilityInfo[]): GroupedAbilities[] {
    const groups: GroupedAbilities[] = [];
    let currentGroup: GroupedAbilities = { title: null, items: [] };

    for (const ability of abilities) {
        if (ability.textFormat === "TITLE") {
            if (currentGroup.items.length > 0 || currentGroup.title !== null) {
                groups.push(currentGroup);
            }
            currentGroup = { title: ability.text, items: [] };
        } else {
            currentGroup.items.push(ability);
        }
    }

    if (currentGroup.items.length > 0 || currentGroup.title !== null) {
        groups.push(currentGroup);
    }

    return groups;
}

export const EnemyDetailDialog = memo(function EnemyDetailDialog({ enemy }: EnemyDetailDialogProps) {
    const { resolvedTheme } = useTheme();
    const levelColor = LEVEL_BAR_COLORS[enemy.enemyLevel] ?? "#71717a";
    const levelTextColor = (resolvedTheme === "light" ? LEVEL_TEXT_COLORS_LIGHT : LEVEL_TEXT_COLORS)[enemy.enemyLevel] ?? "#a1a1aa";

    const levels = enemy.stats?.levels ?? [];
    const [selectedLevel, setSelectedLevel] = useState<number>(levels.length > 0 ? levels.length - 1 : 0);
    const currentLevelStats = levels[selectedLevel];

    const groupedAbilities = useMemo(() => groupAbilities(enemy.abilityList), [enemy.abilityList]);

    return (
        <div className="relative flex h-[85vh] max-h-[700px] w-[95vw] max-w-3xl flex-col overflow-hidden rounded-xl border border-border bg-card shadow-2xl">
            {/* Close button */}
            <MorphingDialogClose className="absolute top-3 right-3 z-10 rounded-full bg-background/80 p-1.5 backdrop-blur-sm hover:bg-background" />

            {/* Header */}
            <div className="relative flex shrink-0 gap-4 border-border border-b bg-muted/30 p-4">
                {/* Portrait */}
                <div className="relative h-24 w-24 shrink-0 overflow-hidden rounded-lg border border-border bg-background">
                    <Image alt={`${enemy.name} Portrait`} className="object-contain" fill src={`/api/cdn${enemy.portrait}`} />
                    {/* Level color bar */}
                    <div className="absolute inset-x-0 bottom-0 h-1" style={{ backgroundColor: levelColor }} />
                </div>

                {/* Basic Info */}
                <div className="flex min-w-0 flex-1 flex-col justify-center gap-1">
                    <MorphingDialogTitle className="flex items-center gap-2">
                        <h2 className="truncate font-bold text-xl">{enemy.name}</h2>
                        {enemy.enemyLevel !== "NORMAL" && (
                            <div className="h-6 w-6 shrink-0">
                                <EnemyLevelLogo level={enemy.enemyLevel} size={24} />
                            </div>
                        )}
                    </MorphingDialogTitle>

                    <div className="flex flex-wrap items-center gap-2 text-sm">
                        <span className="font-semibold" style={{ color: levelTextColor }}>
                            {ENEMY_LEVEL_DISPLAY[enemy.enemyLevel]}
                        </span>
                        <span className="text-muted-foreground">•</span>
                        <span className="text-muted-foreground">{enemy.enemyIndex}</span>
                    </div>

                    {/* Damage Types */}
                    <div className="mt-1 flex flex-wrap gap-1.5">
                        {enemy.damageType.map((type) => (
                            <Badge key={type} variant="secondary" className="text-xs">
                                {DAMAGE_TYPE_DISPLAY[type]}
                            </Badge>
                        ))}
                        {enemy.attackType && (
                            <Badge variant="outline" className="text-xs">
                                {enemy.attackType}
                            </Badge>
                        )}
                    </div>
                </div>
            </div>

            {/* Content with Tabs */}
            <Tabs defaultValue="overview" className="flex min-h-0 flex-1 flex-col">
                <TabsList className="mx-4 mt-3 w-fit">
                    <TabsTrigger value="overview">Overview</TabsTrigger>
                    <TabsTrigger value="stats">Stats</TabsTrigger>
                    {levels.some((l) => l.skills.length > 0) && <TabsTrigger value="skills">Skills</TabsTrigger>}
                </TabsList>

                <div className="min-h-0 flex-1">
                    {/* Overview Tab */}
                    <TabsContent value="overview" className="h-full">
                        <ScrollArea className="h-full">
                            <div className="space-y-4 p-4">
                                {/* Description */}
                                {enemy.description && (
                                    <section>
                                        <h3 className="mb-2 font-semibold text-muted-foreground text-sm">Description</h3>
                                        <MorphingDialogDescription>
                                            <p className="text-sm leading-relaxed">{enemy.description}</p>
                                        </MorphingDialogDescription>
                                    </section>
                                )}

                                {/* Abilities - formatted with titles and bullet points */}
                                {groupedAbilities.length > 0 && (
                                    <section>
                                        <h3 className="mb-2 font-semibold text-muted-foreground text-sm">Traits</h3>
                                        <div className="space-y-3">
                                            {groupedAbilities.map((group, groupIndex) => (
                                                <div key={groupIndex}>
                                                    {group.title && <h4 className="mb-1.5 font-semibold text-red-500 text-sm">{group.title}</h4>}
                                                    {group.items.length > 0 && (
                                                        <ul className="space-y-1.5">
                                                            {group.items.map((ability, index) => (
                                                                <li key={index} className="flex gap-2 text-sm">
                                                                    <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-muted-foreground/50" />
                                                                    <span
                                                                        className={cn("leading-relaxed", ability.textFormat === "SILENCE" && "text-muted-foreground")}
                                                                        dangerouslySetInnerHTML={{
                                                                            __html: descriptionToHtml(ability.text, []),
                                                                        }}
                                                                    />
                                                                </li>
                                                            ))}
                                                        </ul>
                                                    )}
                                                </div>
                                            ))}
                                        </div>
                                    </section>
                                )}

                                {/* Tags */}
                                {enemy.enemyTags && enemy.enemyTags.length > 0 && (
                                    <section>
                                        <h3 className="mb-2 font-semibold text-muted-foreground text-sm">Tags</h3>
                                        <div className="flex flex-wrap gap-1.5">
                                            {enemy.enemyTags.map((tag) => (
                                                <Badge key={tag} variant="outline" className="text-xs">
                                                    {tag}
                                                </Badge>
                                            ))}
                                        </div>
                                    </section>
                                )}

                                {/* Linked Enemies */}
                                {enemy.linkEnemies.length > 0 && (
                                    <section>
                                        <h3 className="mb-2 font-semibold text-muted-foreground text-sm">Related Enemies</h3>
                                        <div className="flex flex-wrap gap-1.5">
                                            {enemy.linkEnemies.map((linkedId) => (
                                                <Badge key={linkedId} variant="secondary" className="text-xs">
                                                    {linkedId}
                                                </Badge>
                                            ))}
                                        </div>
                                    </section>
                                )}

                                {/* Metadata */}
                                <section>
                                    <h3 className="mb-2 font-semibold text-muted-foreground text-sm">Metadata</h3>
                                    <div className="grid grid-cols-2 gap-2 text-xs sm:grid-cols-3">
                                        <MetadataItem label="Enemy ID" value={enemy.enemyId} />
                                        <MetadataItem label="Sort ID" value={String(enemy.sortId)} />
                                        <MetadataItem label="Invalid Kill" value={enemy.isInvalidKilled ? "Yes" : "No"} />
                                        <MetadataItem label="Hidden in Handbook" value={enemy.hideInHandbook ? "Yes" : "No"} />
                                        <MetadataItem label="Hidden in Stage" value={enemy.hideInStage ? "Yes" : "No"} />
                                        <MetadataItem label="Invisible Detail" value={enemy.invisibleDetail ? "Yes" : "No"} />
                                    </div>
                                </section>
                            </div>
                        </ScrollArea>
                    </TabsContent>

                    {/* Stats Tab */}
                    <TabsContent value="stats" className="h-full">
                        <ScrollArea className="h-full">
                            <div className="space-y-4 p-4">
                                {levels.length > 0 ? (
                                    <>
                                        {/* Level Selector - only show if multiple levels */}
                                        {levels.length > 1 && (
                                            <div className="flex items-center gap-2">
                                                <span className="font-medium text-sm">Level:</span>
                                                <div onMouseDown={(e) => e.stopPropagation()}>
                                                    <Select value={String(selectedLevel)} onValueChange={(value) => setSelectedLevel(Number(value))}>
                                                        <SelectTrigger className="w-32">
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                        <SelectContent position="popper" sideOffset={4} className="z-[200]">
                                                            {levels.map((_, index) => (
                                                                <SelectItem key={index} value={String(index)}>
                                                                    Level {index}
                                                                </SelectItem>
                                                            ))}
                                                        </SelectContent>
                                                    </Select>
                                                </div>
                                            </div>
                                        )}

                                        {currentLevelStats && (
                                            <>
                                                {/* Combat Stats */}
                                                <section>
                                                    <h3 className="mb-3 font-semibold text-muted-foreground text-sm">Combat Stats</h3>
                                                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
                                                        <StatCard label="HP" value={currentLevelStats.attributes.maxHp} icon={<Shield className="h-4 w-4" />} />
                                                        <StatCard label="ATK" value={currentLevelStats.attributes.atk} icon={<Swords className="h-4 w-4" />} />
                                                        <StatCard label="DEF" value={currentLevelStats.attributes.def} />
                                                        <StatCard label="RES" value={`${currentLevelStats.attributes.magicResistance}%`} />
                                                        <StatCard label="Move Speed" value={currentLevelStats.attributes.moveSpeed.toFixed(2)} />
                                                        <StatCard label="ATK Speed" value={currentLevelStats.attributes.attackSpeed.toFixed(2)} />
                                                        <StatCard label="Base ATK Time" value={currentLevelStats.attributes.baseAttackTime.toFixed(2)} />
                                                        <StatCard label="Weight" value={currentLevelStats.attributes.massLevel} />
                                                        {currentLevelStats.attributes.hpRecoveryPerSec > 0 && <StatCard label="HP Regen" value={`${currentLevelStats.attributes.hpRecoveryPerSec}/s`} />}
                                                    </div>
                                                </section>

                                                {/* Attack Info */}
                                                <section>
                                                    <h3 className="mb-3 font-semibold text-muted-foreground text-sm">Attack Info</h3>
                                                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                                                        <StatCard label="Attack Type" value={currentLevelStats.applyWay ? APPLY_WAY_DISPLAY[currentLevelStats.applyWay] : "None"} />
                                                        {currentLevelStats.rangeRadius !== null && <StatCard label="Range" value={currentLevelStats.rangeRadius.toFixed(2)} />}
                                                        <StatCard label="Life Seal" value={currentLevelStats.lifePointReduce} />
                                                        {currentLevelStats.motion && <StatCard label="Motion" value={currentLevelStats.motion} />}
                                                    </div>
                                                </section>

                                                {/* Immunities */}
                                                <section>
                                                    <h3 className="mb-3 font-semibold text-muted-foreground text-sm">Immunities</h3>
                                                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-5">
                                                        <ImmunityBadge label="Stun" immune={currentLevelStats.attributes.stunImmune} />
                                                        <ImmunityBadge label="Silence" immune={currentLevelStats.attributes.silenceImmune} />
                                                        <ImmunityBadge label="Sleep" immune={currentLevelStats.attributes.sleepImmune} />
                                                        <ImmunityBadge label="Frozen" immune={currentLevelStats.attributes.frozenImmune} />
                                                        <ImmunityBadge label="Levitate" immune={currentLevelStats.attributes.levitateImmune} />
                                                    </div>
                                                </section>
                                            </>
                                        )}
                                    </>
                                ) : (
                                    <p className="text-center text-muted-foreground text-sm">No stats available</p>
                                )}
                            </div>
                        </ScrollArea>
                    </TabsContent>

                    {/* Skills Tab */}
                    <TabsContent value="skills" className="h-full">
                        <ScrollArea className="h-full">
                            <div className="space-y-4 p-4">
                                {levels.length > 1 && (
                                    <div className="flex items-center gap-2">
                                        <span className="font-medium text-sm">Level:</span>
                                        <div onMouseDown={(e) => e.stopPropagation()}>
                                            <Select value={String(selectedLevel)} onValueChange={(value) => setSelectedLevel(Number(value))}>
                                                <SelectTrigger className="w-32">
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent position="popper" sideOffset={4} className="z-[200]">
                                                    {levels.map((_, index) => (
                                                        <SelectItem key={index} value={String(index)}>
                                                            Level {index}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>
                                    </div>
                                )}

                                {currentLevelStats?.skills.length ? (
                                    <div className="space-y-3">
                                        {currentLevelStats.skills.map((skill, index) => (
                                            <div key={`${skill.prefabKey}-${index}`} className="rounded-lg border border-border bg-muted/30 p-4">
                                                <div className="mb-3 flex items-center justify-between">
                                                    <h4 className="font-semibold text-sm">{skill.prefabKey}</h4>
                                                    <Badge variant="outline" className="text-xs">
                                                        Priority: {skill.priority}
                                                    </Badge>
                                                </div>

                                                <div className="grid grid-cols-2 gap-2 text-xs sm:grid-cols-4">
                                                    <div className="rounded bg-background/50 p-2">
                                                        <span className="block text-muted-foreground">Cooldown</span>
                                                        <span className="font-medium">{skill.cooldown}s</span>
                                                    </div>
                                                    <div className="rounded bg-background/50 p-2">
                                                        <span className="block text-muted-foreground">Init CD</span>
                                                        <span className="font-medium">{skill.initCooldown}s</span>
                                                    </div>
                                                    <div className="rounded bg-background/50 p-2">
                                                        <span className="block text-muted-foreground">SP Cost</span>
                                                        <span className="font-medium">{skill.spCost}</span>
                                                    </div>
                                                    <div className="rounded bg-background/50 p-2">
                                                        <span className="block text-muted-foreground">Priority</span>
                                                        <span className="font-medium">{skill.priority}</span>
                                                    </div>
                                                </div>

                                                {/* Blackboard entries */}
                                                {skill.blackboard.length > 0 && (
                                                    <div className="mt-3">
                                                        <span className="mb-2 block font-medium text-muted-foreground text-xs">Parameters</span>
                                                        <div className="grid grid-cols-1 gap-1.5 text-xs sm:grid-cols-2">
                                                            {skill.blackboard.map((entry) => (
                                                                <div key={entry.key} className="flex items-center justify-between gap-2 rounded bg-background/50 px-2 py-1.5">
                                                                    <span className="min-w-0 truncate text-muted-foreground" title={entry.key}>
                                                                        {entry.key}
                                                                    </span>
                                                                    <span className="shrink-0 font-medium font-mono">{entry.valueStr ?? entry.value}</span>
                                                                </div>
                                                            ))}
                                                        </div>
                                                    </div>
                                                )}
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-center text-muted-foreground text-sm">No skills at this level</p>
                                )}
                            </div>
                        </ScrollArea>
                    </TabsContent>
                </div>
            </Tabs>
        </div>
    );
});

// Helper Components

function MetadataItem({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded bg-muted/50 px-2 py-1.5">
            <span className="block text-muted-foreground">{label}</span>
            <span className="font-medium">{value}</span>
        </div>
    );
}

function StatCard({ label, value, icon }: { label: string; value: string | number; icon?: React.ReactNode }) {
    return (
        <div className="rounded-lg border border-border bg-muted/30 p-3">
            <div className="flex items-center gap-1.5 text-muted-foreground">
                {icon}
                <span className="text-xs">{label}</span>
            </div>
            <span className="mt-1 block font-bold text-lg">{value}</span>
        </div>
    );
}

function ImmunityBadge({ label, immune }: { label: string; immune: boolean }) {
    return (
        <div className={cn("flex items-center gap-1.5 rounded-md border px-2 py-1.5 text-xs", immune ? "border-green-500/50 bg-green-500/10 text-green-500" : "border-border bg-muted/30 text-muted-foreground")}>
            {immune ? <Check className="h-3 w-3" /> : <X className="h-3 w-3" />}
            <span>{label}</span>
        </div>
    );
}
