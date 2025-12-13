"use client";

import { ChevronDown, Coins, Dna, Grid3X3, Heart, Info, MapPin, Palette, Shield, Swords, Timer, User } from "lucide-react";
import Image from "next/image";
import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { Range } from "~/types/api/impl/range";
import { OperatorRange } from "../ui/operator-range";
import { StatCard } from "../ui/stat-card";

interface InfoContentProps {
    operator: Operator;
}

export function InfoContent({ operator }: InfoContentProps) {
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

    // Fetch ranges
    useEffect(() => {
        const fetchRanges = async () => {
            const rangeIds = new Set<string>();
            operator.phases.forEach((phase) => {
                if (phase.RangeId) rangeIds.add(phase.RangeId);
            });

            const rangePromises = Array.from(rangeIds).map(async (rangeId) => {
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
            results.forEach((result) => {
                if (result?.range) {
                    rangeMap[result.id] = result.range;
                }
            });
            setRanges(rangeMap);
        };

        fetchRanges();
    }, [operator.phases]);

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

    return (
        <div className="p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Operator Information</h2>
                <p className="text-muted-foreground text-sm">{operator.description}</p>
            </div>

            {/* Profile Info */}
            <Collapsible onOpenChange={setShowProfile} open={showProfile}>
                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <div className="flex items-center gap-2">
                        <User className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">Profile</span>
                    </div>
                    <ChevronDown className={cn("h-4 w-4 transition-transform", showProfile && "rotate-180")} />
                </CollapsibleTrigger>
                <CollapsibleContent>
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
                </CollapsibleContent>
            </Collapsible>

            <Separator className="my-6" />

            {/* Controls */}
            <Collapsible onOpenChange={setShowControls} open={showControls}>
                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <div className="flex items-center gap-2">
                        <Grid3X3 className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">Operator Controls</span>
                    </div>
                    <ChevronDown className={cn("h-4 w-4 transition-transform", showControls && "rotate-180")} />
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="mt-3 space-y-4 rounded-lg border border-border/50 bg-card/30 p-4">
                        <p className="text-muted-foreground text-xs">Adjust these controls to see how stats change at different levels, promotions, and trust levels.</p>

                        {/* Elite Phase Selection */}
                        <div className="flex flex-wrap items-center gap-2">
                            <span className="text-muted-foreground text-sm">Promotion:</span>
                            {operator.phases.map((_, idx) => (
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
                            <Slider className="w-full" max={currentPhase?.MaxLevel ?? 1} min={1} onValueChange={(val) => setLevel(val[0] ?? 1)} step={1} value={[level]} />
                        </div>

                        {/* Trust Slider */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">Trust</span>
                                <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                            </div>
                            <Slider className="w-full" max={200} min={0} onValueChange={(val) => setTrustLevel(val[0] ?? 100)} step={1} value={[trustLevel]} />
                        </div>
                    </div>
                </CollapsibleContent>
            </Collapsible>

            <Separator className="my-6" />

            {/* Stats Grid */}
            <div className="mb-6">
                <h3 className="mb-4 font-medium text-foreground">Combat Stats</h3>
                <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                    <StatCard icon={Heart} label="Health" value={attributeStats?.MaxHp ?? 0} />
                    <StatCard icon={Swords} label="ATK" value={attributeStats?.Atk ?? 0} />
                    <StatCard icon={Shield} label="DEF" value={attributeStats?.Def ?? 0} />
                    <StatCard label="RES" value={attributeStats?.MagicResistance ?? 0} />
                    <StatCard label="Block" value={attributeStats?.BlockCnt ?? 0} />
                    <StatCard icon={Timer} label="Redeploy" value={`${attributeStats?.RespawnTime ?? 0}s`} />
                    <StatCard icon={Coins} label="DP Cost" value={attributeStats?.Cost ?? 0} />
                    <StatCard label="ATK Interval" value={`${attributeStats?.BaseAttackTime?.toFixed(2) ?? 0}s`} />
                </div>
            </div>

            {/* Tags */}
            {operator.tagList && operator.tagList.length > 0 && (
                <div className="mb-6">
                    <h3 className="mb-3 font-medium text-foreground">Tags</h3>
                    <div className="flex flex-wrap gap-2">
                        {operator.tagList.map((tag, idx) => (
                            <Badge className="bg-secondary/50" key={idx} variant="secondary">
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
                            if (!candidate) return null;
                            return (
                                <div className="rounded-lg border border-border bg-secondary/20 p-4" key={idx}>
                                    <h4 className="mb-1 font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                    <p className="text-muted-foreground text-sm" dangerouslySetInnerHTML={{ __html: candidate.Description ?? "" }} />
                                </div>
                            );
                        })}
                    </div>
                </div>
            )}
        </div>
    );
}

function ProfileItem({ icon: Icon, label, value }: { icon?: React.ElementType; label: string; value: string }) {
    return (
        <div className="rounded-lg border border-border/50 bg-secondary/20 p-3">
            <div className="flex items-center gap-2 text-muted-foreground text-xs">
                {Icon && <Icon className="h-3 w-3" />}
                {label}
            </div>
            <div className="mt-1 font-medium text-foreground text-sm">{value}</div>
        </div>
    );
}
