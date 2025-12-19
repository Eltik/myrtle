"use client";

import { ChevronDown, Coins, Diamond, Dna, Grid3X3, Heart, Hourglass, Info, MapPin, Package, Palette, Shield, ShieldBan, Swords, Timer, User } from "lucide-react";
import { motion } from "motion/react";
import Image from "next/image";
import type React from "react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Badge } from "~/components/ui/shadcn/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Separator } from "~/components/ui/shadcn/separator";
import { Slider } from "~/components/ui/shadcn/slider";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { descriptionToHtml } from "~/lib/description-parser";
import { blackboardToInterpolatedValues, formatAttributeKey, formatOperatorDescription, getActiveTalentCandidate } from "~/lib/operator-helpers";
import { getOperatorAttributeStats } from "~/lib/operator-stats";
import { cn } from "~/lib/utils";
import type { Blackboard, Operator } from "~/types/api";
import type { Range } from "~/types/api/impl/range";
import { OperatorRange } from "../ui/operator-range";
import { StatCard } from "../ui/stat-card";

interface InfoContentProps {
    operator: Operator;
}

export const InfoContent = memo(function InfoContent({ operator }: InfoContentProps) {
    // State for operator configuration
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.MaxLevel ?? 1);
    const [trustLevel, setTrustLevel] = useState(100);
    const [potentialRank, setPotentialRank] = useState(0);
    const [showControls, setShowControls] = useState(true);
    const [showProfile, setShowProfile] = useState(true);
    const [showModuleDetails, setShowModuleDetails] = useState(true);

    // Module state - use "none" as sentinel for user explicitly choosing no module
    const [currentModuleId, setCurrentModuleId] = useState<string>("");
    const [currentModuleLevel, setCurrentModuleLevel] = useState<number>(0);
    // Track if user has made an explicit module selection (to prevent auto-select from overriding)
    const userHasSelectedModule = useRef(false);

    // Get modules that are available
    const availableModules = useMemo(() => {
        return operator.modules.filter((m) => m.type !== "INITIAL");
    }, [operator.modules]);

    // Get current module data
    const currentModule = useMemo(() => {
        if (!currentModuleId) return null;
        return availableModules.find((m) => m.uniEquipId === currentModuleId) ?? null;
    }, [availableModules, currentModuleId]);

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

    // Calculate stats using the utility function from operator-stats.ts
    const attributeStats = useMemo(() => {
        return getOperatorAttributeStats(
            operator,
            {
                phaseIndex,
                favorPoint: trustLevel,
                potentialRank,
                moduleId: currentModuleId,
                moduleLevel: currentModuleLevel,
            },
            level,
        );
    }, [operator, phaseIndex, level, trustLevel, potentialRank, currentModuleId, currentModuleLevel]);

    // Update level when phase changes
    useEffect(() => {
        const maxLevel = operator.phases[phaseIndex]?.MaxLevel ?? 1;
        setLevel(maxLevel);
    }, [phaseIndex, operator.phases]);

    // Handle module selection when phase changes
    useEffect(() => {
        if (phaseIndex !== 2) {
            // Reset module selection when not at E2
            setCurrentModuleId("");
            setCurrentModuleLevel(0);
            userHasSelectedModule.current = false;
        } else if (availableModules.length > 0 && !currentModuleId && !userHasSelectedModule.current) {
            // Auto-select first module when reaching E2, only if user hasn't explicitly chosen
            const firstModule = availableModules[0];
            if (firstModule) {
                setCurrentModuleId(firstModule.uniEquipId);
                const maxModuleLevel = firstModule.data?.phases?.length ?? 0;
                setCurrentModuleLevel(maxModuleLevel);
            }
        }
    }, [phaseIndex, availableModules, currentModuleId]);

    // Handle module change
    const handleModuleChange = useCallback(
        (moduleId: string) => {
            userHasSelectedModule.current = true; // Mark that user has made an explicit selection
            if (moduleId === "none") {
                setCurrentModuleId("");
                setCurrentModuleLevel(0);
                return;
            }
            setCurrentModuleId(moduleId);
            const selectedModule = availableModules.find((m) => m.uniEquipId === moduleId);
            if (selectedModule) {
                const maxModuleLevel = selectedModule.data?.phases?.length ?? 0;
                setCurrentModuleLevel(maxModuleLevel);
            }
        },
        [availableModules],
    );

    // Handle module level change
    const handleModuleLevelChange = useCallback((levelStr: string) => {
        const newLevel = Number.parseInt(levelStr, 10);
        if (!Number.isNaN(newLevel)) {
            setCurrentModuleLevel(newLevel);
        }
    }, []);

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
                        <p className="text-muted-foreground text-xs">Adjust these controls to see how stats change at different levels, promotions, potentials, modules, and trust levels.</p>

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

                        {/* Potential Selection */}
                        <div className="space-y-2">
                            <div className="flex items-center gap-2">
                                <span className="text-muted-foreground text-sm">Potential:</span>
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            <p>Select the potential rank to see stat bonuses</p>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            </div>
                            <div className="flex flex-wrap items-center gap-2">
                                {/* Potential 0 (no potential) */}
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger asChild>
                                            <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", potentialRank === 0 ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")} onClick={() => setPotentialRank(0)} type="button">
                                                <Image alt="No Potential" height={28} src="/api/cdn/upk/arts/potential_hub/potential_0.png" width={28} />
                                            </button>
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            <p>No Potential</p>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                                {/* Potentials 1-5 */}
                                {operator.potentialRanks.map((rank, idx) => (
                                    <TooltipProvider key={rank.Description}>
                                        <Tooltip>
                                            <TooltipTrigger asChild>
                                                <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", potentialRank === idx + 1 ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")} onClick={() => setPotentialRank(idx + 1)} type="button">
                                                    <Image alt={`Potential ${idx + 1}`} height={28} src={`/api/cdn/upk/arts/potential_hub/potential_${idx + 1}.png`} width={28} />
                                                </button>
                                            </TooltipTrigger>
                                            <TooltipContent>
                                                <p>
                                                    Pot {idx + 1}: {rank.Description}
                                                </p>
                                            </TooltipContent>
                                        </Tooltip>
                                    </TooltipProvider>
                                ))}
                            </div>
                        </div>

                        {/* Trust Slider */}
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-1">
                                    <Heart className="h-3.5 w-3.5 text-muted-foreground" />
                                    <span className="text-muted-foreground text-sm">Trust</span>
                                </div>
                                <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                            </div>
                            <Slider className="w-full" max={200} min={0} onValueChange={handleTrustChange} step={1} value={[trustLevel]} />
                        </div>

                        {/* Module Selection (only at E2) */}
                        {phaseIndex === 2 && availableModules.length > 0 && (
                            <div className="rounded-lg border border-border/50 bg-secondary/10 p-3">
                                <div className="mb-3 flex items-center gap-2">
                                    <Package className="h-4 w-4 text-primary" />
                                    <span className="font-medium text-sm">Module Selection</span>
                                </div>
                                <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
                                    {/* Module Type */}
                                    <div className="space-y-1">
                                        <div className="flex items-center gap-1">
                                            <span className="text-muted-foreground text-xs">Module Type</span>
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Select a module to see how it affects stats</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        </div>
                                        <Select onValueChange={handleModuleChange} value={currentModuleId || "none"}>
                                            <SelectTrigger className="w-full">
                                                <SelectValue placeholder="Select Module" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="none">No Module</SelectItem>
                                                {availableModules.map((mod) => (
                                                    <SelectItem key={mod.uniEquipId} value={mod.uniEquipId}>
                                                        {mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : mod.uniEquipName}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>

                                    {/* Module Level */}
                                    {currentModule?.data?.phases && currentModule.data.phases.length > 0 && (
                                        <div className="space-y-1">
                                            <div className="flex items-center gap-1">
                                                <span className="text-muted-foreground text-xs">Module Level</span>
                                                <TooltipProvider>
                                                    <Tooltip>
                                                        <TooltipTrigger asChild>
                                                            <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                        </TooltipTrigger>
                                                        <TooltipContent>
                                                            <p>Select module level to see its effects</p>
                                                        </TooltipContent>
                                                    </Tooltip>
                                                </TooltipProvider>
                                            </div>
                                            <Select onValueChange={handleModuleLevelChange} value={String(currentModuleLevel)}>
                                                <SelectTrigger className="w-full">
                                                    <SelectValue placeholder="Select Level" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {currentModule.data.phases.map((phase) => (
                                                        <SelectItem key={phase.equipLevel} value={String(phase.equipLevel)}>
                                                            Level {phase.equipLevel}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
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
                    <StatCard icon={Timer} label="ATK Interval" value={`${attributeStats?.AttackSpeed?.toFixed(2) ?? 0}s`} />
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

            {/* Module Details */}
            {phaseIndex === 2 && currentModule && (
                <Disclosure onOpenChange={setShowModuleDetails} open={showModuleDetails} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                    <DisclosureTrigger>
                        <div className="mb-6 flex w-full cursor-pointer items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                            <div className="flex items-center gap-2">
                                <Package className="h-4 w-4 text-primary" />
                                <span className="font-medium text-sm">Module Details</span>
                            </div>
                            <motion.div animate={{ rotate: showModuleDetails ? 180 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                                <ChevronDown className="h-4 w-4" />
                            </motion.div>
                        </div>
                    </DisclosureTrigger>
                    <DisclosureContent>
                        <div className="mb-6 rounded-lg border border-border/50 bg-card/30 p-4">
                            {/* Module Header */}
                            <div className="mb-4 flex items-center gap-3">
                                {currentModule.image && <Image alt={currentModule.uniEquipName} className="rounded-md object-contain" height={64} src={`/api/cdn${currentModule.image}`} width={64} />}
                                <div>
                                    <h4 className="font-semibold text-foreground">{currentModule.uniEquipName}</h4>
                                    <div className="mt-1 flex gap-1">
                                        <Badge variant="outline">{currentModule.typeName1}</Badge>
                                        {currentModule.typeName2 && <Badge variant="outline">{currentModule.typeName2}</Badge>}
                                    </div>
                                </div>
                            </div>

                            {/* Module Lore/Description */}
                            {currentModule.uniEquipDesc && (
                                <div className="mb-4 max-h-32 overflow-y-auto rounded-md bg-secondary/20 p-3">
                                    <p className="whitespace-pre-line text-muted-foreground text-xs leading-relaxed">{currentModule.uniEquipDesc}</p>
                                </div>
                            )}

                            {/* Module Stats at Current Level */}
                            {currentModule.data?.phases && currentModuleLevel > 0 && (
                                <div className="space-y-3">
                                    <h5 className="font-medium text-foreground text-sm">Level {currentModuleLevel} Stats</h5>
                                    {(() => {
                                        const phase = currentModule.data.phases[currentModuleLevel - 1];
                                        if (!phase?.attributeBlackboard || phase.attributeBlackboard.length === 0) {
                                            return <p className="text-muted-foreground text-xs">No stat bonuses at this level.</p>;
                                        }
                                        return (
                                            <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                                                {phase.attributeBlackboard.map((attr) => (
                                                    <div className="rounded-md bg-secondary/30 p-2" key={attr.key}>
                                                        <span className="text-muted-foreground text-xs">{formatAttributeKey(attr.key)}:</span>
                                                        <span className="ml-1 font-medium text-foreground text-sm">+{attr.value}</span>
                                                    </div>
                                                ))}
                                            </div>
                                        );
                                    })()}

                                    {/* Module Talent/Trait Changes */}
                                    {(() => {
                                        const phase = currentModule.data.phases[currentModuleLevel - 1];
                                        if (!phase?.parts) return null;

                                        const talentChanges = phase.parts.flatMap((part) => part.addOrOverrideTalentDataBundle?.candidates?.filter((c) => c.upgradeDescription || c.description) ?? []);

                                        const traitChanges = phase.parts.flatMap((part) => part.overrideTraitDataBundle?.candidates?.filter((c) => c.additionalDescription) ?? []);

                                        if (talentChanges.length === 0 && traitChanges.length === 0) return null;

                                        return (
                                            <div className="mt-3 space-y-2">
                                                {talentChanges.length > 0 && (
                                                    <div>
                                                        <h6 className="mb-1 font-medium text-foreground text-xs">Talent Changes</h6>
                                                        {talentChanges.map((candidate, idx) => (
                                                            // biome-ignore lint/suspicious/noArrayIndexKey: Static array
                                                            <div className="rounded-md bg-secondary/20 p-2" key={idx}>
                                                                {candidate.name && <span className="font-medium text-foreground text-xs">{candidate.name}: </span>}
                                                                <span
                                                                    className="text-muted-foreground text-xs"
                                                                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Required for formatted description
                                                                    dangerouslySetInnerHTML={{
                                                                        __html: descriptionToHtml(candidate.upgradeDescription || candidate.description || "", []),
                                                                    }}
                                                                />
                                                            </div>
                                                        ))}
                                                    </div>
                                                )}
                                                {traitChanges.length > 0 && (
                                                    <div>
                                                        <h6 className="mb-1 font-medium text-foreground text-xs">Trait Changes</h6>
                                                        {traitChanges.map((candidate, idx) => (
                                                            // biome-ignore lint/suspicious/noArrayIndexKey: Static array
                                                            <div className="rounded-md bg-secondary/20 p-2" key={idx}>
                                                                <span
                                                                    className="text-muted-foreground text-xs"
                                                                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Required for formatted description
                                                                    dangerouslySetInnerHTML={{ __html: descriptionToHtml(candidate.additionalDescription || "", []) }}
                                                                />
                                                            </div>
                                                        ))}
                                                    </div>
                                                )}
                                            </div>
                                        );
                                    })()}
                                </div>
                            )}
                        </div>
                    </DisclosureContent>
                </Disclosure>
            )}

            {/* Talents */}
            {operator.talents && operator.talents.length > 0 && (
                <div>
                    <h3 className="mb-3 font-medium text-foreground">Talents</h3>
                    <div className="space-y-3">
                        {operator.talents.map((talent, idx) => {
                            const candidate = getActiveTalentCandidate(talent, phaseIndex, level, potentialRank);
                            if (!candidate || !candidate.Name) return null;

                            return (
                                // biome-ignore lint/suspicious/noArrayIndexKey: Static talent list
                                <div className="rounded-lg border border-border bg-secondary/20 p-4" key={idx}>
                                    <div className="mb-1 flex items-center gap-2">
                                        <h4 className="font-medium text-foreground">{candidate.Name ?? `Talent ${idx + 1}`}</h4>
                                        {candidate.RequiredPotentialRank > 0 && (
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <Image alt={`Potential ${candidate.RequiredPotentialRank}`} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/potential_hub/potential_${candidate.RequiredPotentialRank}.png`} width={20} />
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Requires Potential {candidate.RequiredPotentialRank}</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        )}
                                    </div>
                                    {/* biome-ignore lint/security/noDangerouslySetInnerHtml: Intentional HTML rendering for talent descriptions */}
                                    <p className="text-muted-foreground text-sm" dangerouslySetInnerHTML={{ __html: descriptionToHtml(candidate.Description ?? "", blackboardToInterpolatedValues(candidate.Blackboard ?? [])) }} />
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
