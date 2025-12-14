"use client";

import { ChevronDown, Coins, Diamond, Dna, Grid3X3, Heart, Hourglass, Info, MapPin, Palette, Shield, ShieldBan, Swords, Timer, User } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import type React from "react";
import { memo, useCallback, useEffect, useMemo, useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Badge } from "~/components/ui/shadcn/badge";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { descriptionToHtml } from "~/lib/description-parser";
import { cn } from "~/lib/utils";
import type { Blackboard, Operator } from "~/types/api";
import type { Range } from "~/types/api/impl/range";
import type { InterpolatedValue } from "~/types/frontend/operators";
import { OperatorRange } from "../ui/operator-range";
import { StatCard } from "../ui/stat-card";

/**
 * Formats operator description with blackboard interpolation
 * Combines trait blackboard values for proper value substitution
 */
function formatOperatorDescription(description: string, blackboard: Blackboard[]): string {
    if (!description) return "";

    // Filter out blackboard entries with undefined/null keys
    const validBlackboard = blackboard.filter((b) => b.key != null);

    // Debug: log what we have if there are placeholders
    if (description.includes("{")) {
        const placeholderMatch = description.match(/\{([^}:]+)/g);
        const placeholderKeys = placeholderMatch?.map((m) => m.slice(1).toLowerCase()) ?? [];
        const blackboardKeys = validBlackboard.map((b) => b.key.toLowerCase());

        const missingKeys = placeholderKeys.filter((key) => !blackboardKeys.includes(key));

        if (missingKeys.length > 0) {
            console.warn("[formatOperatorDescription] Missing blackboard keys:", {
                missingKeys,
                availableKeys: blackboardKeys,
                blackboardValues: validBlackboard,
            });
        }
    }

    // Apply descriptionToHtml for tag coloring and value interpolation
    const interpolatedValues: InterpolatedValue[] = validBlackboard.map((b) => ({
        key: b.key,
        value: b.value,
    }));

    return descriptionToHtml(description, interpolatedValues);
}

interface InfoContentProps {
    operator: Operator;
}

export const InfoContent = memo(function InfoContent({ operator }: InfoContentProps) {
    // State for operator configuration
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.MaxLevel ?? 1);
    const [trustLevel, setTrustLevel] = useState(100);
    const [showControls, setShowControls] = useState(true);
    const [showProfile, setShowProfile] = useState(true);

    // Range state
    const [ranges, setRanges] = useState<Record<string, Range>>({});
    const currentPhase = operator.phases[phaseIndex];
    const currentRangeId = currentPhase?.RangeId ?? "";

    const rangeIds = useMemo(() => {
        const ids = new Set<string>();
        for (const phase of operator.phases) {
            if (phase.RangeId) ids.add(phase.RangeId);
        }
        return Array.from(ids);
    }, [operator.phases]);

    useEffect(() => {
        const fetchRanges = async () => {
            if (rangeIds.length === 0) return;

            const rangePromises = rangeIds.map(async (rangeId) => {
                try {
                    const res = await fetch("/api/static", {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ type: "ranges", id: rangeId }),
                    });
                    const data = await res.json();
                    return { id: rangeId, range: data.data };
                } catch {
                    return null;
                }
            });

            const results = await Promise.all(rangePromises);
            const rangeMap: Record<string, Range> = {};
            for (const result of results) {
                if (result?.range) {
                    rangeMap[result.id] = result.range;
                }
            }
            setRanges(rangeMap);
        };

        fetchRanges();
    }, [rangeIds]);

    // Calculate stats based on level and phase
    const attributeStats = useMemo(() => {
        const phase = operator.phases[phaseIndex];
        if (!phase?.AttributesKeyFrames || phase.AttributesKeyFrames.length === 0) return null;

        const maxLevel = phase.MaxLevel;
        const startFrame = phase.AttributesKeyFrames[0]?.Data;
        const endFrame = phase.AttributesKeyFrames[phase.AttributesKeyFrames.length - 1]?.Data;

        if (!startFrame || !endFrame) return null;

        // Linear interpolation
        const interpolate = (start: number, end: number) => {
            if (maxLevel <= 1) return start;
            return Math.round(start + ((level - 1) * (end - start)) / (maxLevel - 1));
        };

        // Apply trust bonus (simplified)
        const trustMultiplier = Math.min(trustLevel, 100) / 100;
        const trustBonus = operator.favorKeyFrames?.[operator.favorKeyFrames.length - 1]?.Data;

        return {
            MaxHp: interpolate(startFrame.MaxHp, endFrame.MaxHp) + Math.round((trustBonus?.MaxHp ?? 0) * trustMultiplier),
            Atk: interpolate(startFrame.Atk, endFrame.Atk) + Math.round((trustBonus?.Atk ?? 0) * trustMultiplier),
            Def: interpolate(startFrame.Def, endFrame.Def) + Math.round((trustBonus?.Def ?? 0) * trustMultiplier),
            MagicResistance: interpolate(startFrame.MagicResistance, endFrame.MagicResistance),
            Cost: startFrame.Cost,
            BlockCnt: startFrame.BlockCnt,
            RespawnTime: startFrame.RespawnTime,
            BaseAttackTime: startFrame.BaseAttackTime,
        };
    }, [operator, phaseIndex, level, trustLevel]);

    // Update level when phase changes
    useEffect(() => {
        const maxLevel = operator.phases[phaseIndex]?.MaxLevel ?? 1;
        setLevel(maxLevel);
    }, [phaseIndex, operator.phases]);

    const currentRange = ranges[currentRangeId];

    // Get blackboard values for description interpolation
    // The blackboard contains values like {atk}, {max_stack_cnt} that need to be substituted
    // Check trait first, then fall back to talents if trait blackboard is empty
    const descriptionBlackboard: Blackboard[] = useMemo(() => {
        // Try trait blackboard first
        const traitCandidate = operator.trait?.Candidates?.[operator.trait.Candidates.length - 1];
        const traitBlackboard = traitCandidate?.Blackboard ?? [];

        if (traitBlackboard.length > 0) {
            return traitBlackboard;
        }

        // Fall back to combining all talent blackboards
        const talentBlackboards: Blackboard[] = [];
        for (const talent of operator.talents ?? []) {
            const candidate = talent.Candidates?.[talent.Candidates.length - 1];
            if (candidate?.Blackboard) {
                talentBlackboards.push(...candidate.Blackboard);
            }
        }

        return talentBlackboards;
    }, [operator.trait, operator.talents]);

    const formattedDescription = useMemo(() => formatOperatorDescription(operator.description, descriptionBlackboard), [operator.description, descriptionBlackboard]);

    const handleLevelChange = useCallback((val: number[]) => {
        setLevel(val[0] ?? 1);
    }, []);

    const handleTrustChange = useCallback((val: number[]) => {
        setTrustLevel(val[0] ?? 100);
    }, []);

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Operator Information</h2>
                {/* biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for operator descriptions */}
                <p className="wrap-break-word text-muted-foreground text-sm" dangerouslySetInnerHTML={{ __html: formattedDescription }} />
            </div>

            {/* Profile Info */}
            <Disclosure onOpenChange={setShowProfile} open={showProfile} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                <DisclosureTrigger>
                    <div className="flex w-full cursor-pointer items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                        <div className="flex items-center gap-2">
                            <User className="h-4 w-4 text-primary" />
                            <span className="font-medium text-sm">Profile</span>
                        </div>
                        <motion.div animate={{ rotate: showProfile ? 180 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </div>
                </DisclosureTrigger>
                <DisclosureContent>
                    <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
                        {operator.profile?.basicInfo && (
                            <>
                                <ProfileItem icon={MapPin} label="Place of Birth" value={operator.profile.basicInfo.placeOfBirth ?? "Unknown"} />
                                <ProfileItem icon={Dna} label="Race" value={operator.profile.basicInfo.race ?? "Unknown"} />
                                <ProfileItem icon={User} label="Gender" value={operator.profile.basicInfo.gender ?? "Unknown"} />
                                <ProfileItem icon={Info} label="Height" value={operator.profile.basicInfo.height ?? "Unknown"} />
                            </>
                        )}
                        {operator.artists && operator.artists.length > 0 && <ProfileItem icon={Palette} label="Artist" value={operator.artists.join(", ")} />}
                    </div>
                </DisclosureContent>
            </Disclosure>

            <Separator className="my-6" />

            {/* Controls */}
            <Disclosure onOpenChange={setShowControls} open={showControls} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                <DisclosureTrigger>
                    <div className="flex w-full cursor-pointer items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                        <div className="flex items-center gap-2">
                            <Grid3X3 className="h-4 w-4 text-primary" />
                            <span className="font-medium text-sm">Operator Controls</span>
                        </div>
                        <motion.div animate={{ rotate: showControls ? 180 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <ChevronDown className="h-4 w-4" />
                        </motion.div>
                    </div>
                </DisclosureTrigger>
                <DisclosureContent>
                    <div className="mt-3 space-y-4 rounded-lg border border-border/50 bg-card/30 p-4">
                        <p className="text-muted-foreground text-xs">Adjust these controls to see how stats change at different levels, promotions, and trust levels.</p>

                        {/* Elite Phase Selection */}
                        <div className="flex flex-wrap items-center gap-2">
                            <span className="text-muted-foreground text-sm">Promotion:</span>
                            {operator.phases.map((_, idx) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static array of promotion phases
                                <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", phaseIndex === idx ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")} key={idx} onClick={() => setPhaseIndex(idx)} type="button">
                                    <Image alt={`Elite ${idx}`} height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx}.png`} width={24} />
                                </button>
                            ))}
                        </div>

                        {/* Level Slider */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">Level</span>
                                <span className="font-mono text-foreground text-sm">
                                    {level} / {currentPhase?.MaxLevel ?? 1}
                                </span>
                            </div>
                            <Slider className="w-full" max={currentPhase?.MaxLevel ?? 1} min={1} onValueChange={handleLevelChange} step={1} value={[level]} />
                        </div>

                        {/* Trust Slider */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">Trust</span>
                                <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                            </div>
                            <Slider className="w-full" max={200} min={0} onValueChange={handleTrustChange} step={1} value={[trustLevel]} />
                        </div>
                    </div>
                </DisclosureContent>
            </Disclosure>

            <Separator className="my-6" />

            {/* Stats Grid */}
            <div className="mb-6">
                <h3 className="mb-4 font-medium text-foreground">Combat Stats</h3>
                <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                    <StatCard icon={Heart} label="Health" value={attributeStats?.MaxHp ?? 0} />
                    <StatCard icon={Swords} label="ATK" value={attributeStats?.Atk ?? 0} />
                    <StatCard icon={Shield} label="DEF" value={attributeStats?.Def ?? 0} />
                    <StatCard icon={Diamond} label="RES" value={attributeStats?.MagicResistance ?? 0} />
                    <StatCard icon={ShieldBan} label="Block" value={attributeStats?.BlockCnt ?? 0} />
                    <StatCard icon={Hourglass} label="Redeploy" value={`${attributeStats?.RespawnTime ?? 0}s`} />
                    <StatCard icon={Coins} label="DP Cost" value={attributeStats?.Cost ?? 0} />
                    <StatCard icon={Timer} label="ATK Interval" value={`${attributeStats?.BaseAttackTime?.toFixed(2) ?? 0}s`} />
                </div>
            </div>

            {/* Tags */}
            {operator.tagList && operator.tagList.length > 0 && (
                <div className="mb-6">
                    <h3 className="mb-3 font-medium text-foreground">Tags</h3>
                    <div className="flex flex-wrap gap-2">
                        {operator.tagList.map((tag, idx) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: Static tag list
                            <Badge className="bg-accent" key={idx} variant="secondary">
                                {tag}
                            </Badge>
                        ))}
                    </div>
                </div>
            )}

            {/* Range */}
            <div className="mb-6">
                <h3 className="mb-3 font-medium text-foreground">Attack Range</h3>
                {currentRange ? <OperatorRange range={currentRange} /> : <p className="text-muted-foreground text-sm">No range data available.</p>}
            </div>

            {/* Talents */}
            {operator.talents && operator.talents.length > 0 && (
                <div>
                    <h3 className="mb-3 font-medium text-foreground">Talents</h3>
                    <div className="space-y-3">
                        {operator.talents.map((talent, idx) => {
                            const candidate = talent.Candidates?.[talent.Candidates.length - 1];
                            if (!candidate || !candidate.Name) return null;
                            return (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static talent list
                                <div className="rounded-lg border border-border bg-secondary/20 p-4" key={idx}>
                                    <h4 className="mb-1 font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                    {/* biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for talent descriptions */}
                                    <p className="text-muted-foreground text-sm" dangerouslySetInnerHTML={{ __html: descriptionToHtml(candidate.Description ?? "", []) }} />
                                </div>
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
});

const ProfileItem = memo(function ProfileItem({ icon: Icon, label, value }: { icon?: React.ElementType; label: string; value: string }) {
    return (
        <div className="rounded-lg border border-border/50 bg-secondary/20 p-3">
            <div className="flex items-center gap-2 text-muted-foreground text-xs">
                {Icon && <Icon className="h-3 w-3" />}
                {label}
            </div>
            <div className="mt-1 font-medium text-foreground text-sm">{value}</div>
        </div>
    );
});
