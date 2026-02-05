"use client";

import { Check, ChevronDown, Map as MapIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/motion-primitives/accordion";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Label } from "~/components/ui/shadcn/label";
import { Switch } from "~/components/ui/shadcn/switch";
import type { Stage } from "~/types/api/impl/stage";
import type { User } from "~/types/api/impl/user";
import type { Zone } from "~/types/api/impl/zone";
import { ACTIVITY_NAMES, formatEventDate, getActivityEventTimes, getActivityIdFromZoneId, getActivityStartTime, getPermanentEventInfo, isActivityCurrentlyOpen, isRerunActivity } from "./activity-names";

interface ZoneFilterPanelProps {
    allowedZoneTypes: string[];
    setAllowedZoneTypes: (types: string[]) => void;
    hasProfile?: boolean;
    onlyAvailableStages?: boolean;
    setOnlyAvailableStages?: (value: boolean) => void;
    onlyCompletedStages?: boolean;
    setOnlyCompletedStages?: (value: boolean) => void;
    stages: Stage[];
    zones: Zone[];
    selectedStages: string[];
    setSelectedStages: (stages: string[]) => void;
    user?: User | null;
}

const ZONE_TYPES = [
    { value: "MAINLINE", label: "Main Story", description: "Main story chapters" },
    { value: "ACTIVITY", label: "Side Stories & Events", description: "Side stories, intermezzis, and events" },
];

function naturalSortCompare(a: string, b: string): number {
    const aParts = a.split(/(\d+)/);
    const bParts = b.split(/(\d+)/);
    const maxLen = Math.max(aParts.length, bParts.length);

    for (let i = 0; i < maxLen; i++) {
        const aPart = aParts[i] ?? "";
        const bPart = bParts[i] ?? "";

        const aNum = Number.parseInt(aPart, 10);
        const bNum = Number.parseInt(bPart, 10);

        if (!Number.isNaN(aNum) && !Number.isNaN(bNum)) {
            if (aNum !== bNum) return aNum - bNum;
        } else {
            const cmp = aPart.localeCompare(bPart);
            if (cmp !== 0) return cmp;
        }
    }
    return 0;
}

function getStageDisplayName(stage: Stage): string {
    if (stage.difficulty === "FOUR_STAR") {
        return `${stage.code} CM`;
    }
    return stage.code;
}

function getMainZoneSortNumber(zoneId: string): number {
    const mainMatch = zoneId.match(/^main_(\d+)/);
    if (mainMatch?.[1]) {
        return Number.parseInt(mainMatch[1], 10);
    }
    return 0;
}

interface GroupedEvent {
    activityId: string;
    eventName: string;
    startTime: number;
    endTime: number;
    isOpen: boolean;
    dateRange: string;
    isPermanent: boolean;
    permanentType?: "SIDESTORY" | "BRANCHLINE";
    zones: Array<{
        zoneId: string;
        zone: Zone | undefined;
        stages: Stage[];
        zoneName: string;
    }>;
    totalStages: number;
}

export function ZoneFilterPanel({ allowedZoneTypes, setAllowedZoneTypes, hasProfile, onlyAvailableStages, setOnlyAvailableStages, onlyCompletedStages, setOnlyCompletedStages, stages, zones, selectedStages, setSelectedStages, user }: ZoneFilterPanelProps) {
    const safeAllowedZoneTypes = allowedZoneTypes ?? [];
    const safeSelectedStages = selectedStages ?? [];
    const [openZoneTypes, setOpenZoneTypes] = useState<string[]>([]);
    const [openEventId, setOpenEventId] = useState<string | null>(null);
    const [openZoneId, setOpenZoneId] = useState<string | null>(null);

    const stagesByZoneType = useMemo(() => {
        const result = new Map<string, Map<string, Stage[]>>();

        for (const zoneType of ZONE_TYPES) {
            result.set(zoneType.value, new Map());
        }

        for (const stage of stages) {
            const zone = zones.find((z) => z.zoneId === stage.zoneId);
            if (!zone) continue;

            if (onlyCompletedStages && user) {
                const stageData = user.dungeon.stages[stage.stageId];
                if (!stageData || stageData.completeTimes <= 0) continue;
            }

            const zoneTypeMap = result.get(zone.type);
            if (!zoneTypeMap) continue;

            const existing = zoneTypeMap.get(stage.zoneId) ?? [];
            existing.push(stage);
            zoneTypeMap.set(stage.zoneId, existing);
        }

        for (const [, zoneTypeMap] of result.entries()) {
            for (const [zoneId, zoneStages] of zoneTypeMap.entries()) {
                zoneTypeMap.set(
                    zoneId,
                    zoneStages.sort((a, b) => naturalSortCompare(a.code, b.code)),
                );
            }
        }

        return result;
    }, [stages, zones, onlyCompletedStages, user]);

    const getSortedMainlineZones = () => {
        const zoneTypeMap = stagesByZoneType.get("MAINLINE");
        if (!zoneTypeMap) return [];

        return Array.from(zoneTypeMap.entries())
            .map(([zoneId, zoneStages]) => ({
                zoneId,
                zone: zones.find((z) => z.zoneId === zoneId),
                stages: zoneStages,
            }))
            .filter((entry) => entry.zone)
            .sort((a, b) => {
                const aNum = getMainZoneSortNumber(a.zoneId);
                const bNum = getMainZoneSortNumber(b.zoneId);
                return aNum - bNum;
            });
    };

    const getGroupedEvents = (): GroupedEvent[] => {
        const zoneTypeMap = stagesByZoneType.get("ACTIVITY");
        if (!zoneTypeMap) return [];

        const eventMap = new Map<string, GroupedEvent>();

        for (const [zoneId, zoneStages] of zoneTypeMap.entries()) {
            const activityId = getActivityIdFromZoneId(zoneId);
            if (!activityId) continue;

            const zone = zones.find((z) => z.zoneId === zoneId);
            if (!zone) continue;

            const permanentInfo = getPermanentEventInfo(activityId);
            const isRerun = isRerunActivity(zoneId);

            if (isRerun && !permanentInfo) continue;

            const groupKey = permanentInfo?.retroId ?? activityId;
            const eventName = permanentInfo?.name ?? ACTIVITY_NAMES[activityId] ?? activityId;
            const zoneName = zone.zoneNameSecond ?? zone.zoneNameFirst ?? zoneId;

            if (!eventMap.has(groupKey)) {
                const times = getActivityEventTimes(activityId);
                const startTime = times?.start ?? getActivityStartTime(activityId);
                const endTime = times?.end ?? 0;
                const isOpen = permanentInfo ? true : isActivityCurrentlyOpen(activityId);
                const dateRange = permanentInfo ? "" : times ? `${formatEventDate(times.start)} - ${formatEventDate(times.end)}` : "";

                eventMap.set(groupKey, {
                    activityId: groupKey,
                    eventName,
                    startTime,
                    endTime,
                    isOpen,
                    dateRange,
                    isPermanent: !!permanentInfo,
                    permanentType: permanentInfo?.type,
                    zones: [],
                    totalStages: 0,
                });
            }

            const event = eventMap.get(groupKey);
            if (!event) continue;
            event.zones.push({
                zoneId,
                zone,
                stages: zoneStages,
                zoneName,
            });
            event.totalStages += zoneStages.length;
        }

        for (const event of eventMap.values()) {
            event.zones.sort((a, b) => (a.zone?.zoneIndex ?? 0) - (b.zone?.zoneIndex ?? 0));
        }

        let events = Array.from(eventMap.values());
        if (onlyAvailableStages) {
            events = events.filter((event) => event.isOpen || event.isPermanent);
        }

        return events.sort((a, b) => {
            const aOpenNonPerm = a.isOpen && !a.isPermanent;
            const bOpenNonPerm = b.isOpen && !b.isPermanent;
            if (aOpenNonPerm !== bOpenNonPerm) {
                return aOpenNonPerm ? -1 : 1;
            }
            if (a.isPermanent !== b.isPermanent) {
                return a.isPermanent ? -1 : 1;
            }
            return b.startTime - a.startTime;
        });
    };

    const getStageCountForType = (zoneType: string) => {
        const zoneTypeMap = stagesByZoneType.get(zoneType);
        if (!zoneTypeMap) return 0;
        let count = 0;
        for (const stagesInZone of zoneTypeMap.values()) {
            count += stagesInZone.length;
        }
        return count;
    };

    const getSelectedCountForType = (zoneType: string) => {
        const zoneTypeMap = stagesByZoneType.get(zoneType);
        if (!zoneTypeMap) return 0;
        let count = 0;
        for (const stagesInZone of zoneTypeMap.values()) {
            count += stagesInZone.filter((s) => safeSelectedStages.includes(s.stageId)).length;
        }
        return count;
    };

    const toggleZoneType = (type: string) => {
        if (safeAllowedZoneTypes.includes(type)) {
            if (safeAllowedZoneTypes.length > 1) {
                setAllowedZoneTypes(safeAllowedZoneTypes.filter((t) => t !== type));
            }
        } else {
            setAllowedZoneTypes([...safeAllowedZoneTypes, type]);
        }
    };

    const toggleStage = (stageId: string) => {
        if (safeSelectedStages.includes(stageId)) {
            setSelectedStages(safeSelectedStages.filter((id) => id !== stageId));
        } else {
            setSelectedStages([...safeSelectedStages, stageId]);
        }
    };

    const selectAllStagesForType = (zoneType: string) => {
        const zoneTypeMap = stagesByZoneType.get(zoneType);
        if (!zoneTypeMap) return;

        const stageIds: string[] = [];
        for (const stagesInZone of zoneTypeMap.values()) {
            for (const stage of stagesInZone) {
                stageIds.push(stage.stageId);
            }
        }

        const newSelected = new Set([...safeSelectedStages, ...stageIds]);
        setSelectedStages(Array.from(newSelected));
    };

    const deselectAllStagesForType = (zoneType: string) => {
        const zoneTypeMap = stagesByZoneType.get(zoneType);
        if (!zoneTypeMap) return;

        const stageIdsToRemove = new Set<string>();
        for (const stagesInZone of zoneTypeMap.values()) {
            for (const stage of stagesInZone) {
                stageIdsToRemove.add(stage.stageId);
            }
        }

        setSelectedStages(safeSelectedStages.filter((id) => !stageIdsToRemove.has(id)));
    };

    const toggleZoneTypeOpen = (zoneType: string) => {
        setOpenZoneTypes((prev) => (prev.includes(zoneType) ? prev.filter((t) => t !== zoneType) : [...prev, zoneType]));
    };

    const totalSelected = safeSelectedStages.length;
    const totalStages = stages.filter((stage) => {
        if (onlyCompletedStages && user) {
            const stageData = user.dungeon.stages[stage.stageId];
            return stageData && stageData.completeTimes > 0;
        }
        return true;
    }).length;

    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="space-y-4 p-6">
                <div className="flex items-center gap-3">
                    <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-md">
                        <MapIcon className="h-4.5 w-4.5 text-primary" />
                    </div>
                    <div>
                        <h2 className="font-semibold text-foreground text-lg">Zone & Stage Filters</h2>
                        <p className="text-muted-foreground text-xs">{totalSelected > 0 ? `${totalSelected} stages selected` : `${totalStages} stages available`}</p>
                    </div>
                </div>
                {/* Completion Filter */}
                {hasProfile && setOnlyCompletedStages && (
                    <div className="flex items-center justify-between rounded-lg border border-border/50 bg-secondary/30 p-3">
                        <div className="space-y-0.5">
                            <Label className="font-medium text-foreground text-sm" htmlFor="only-completed">
                                Only Completed Stages
                            </Label>
                            <p className="text-muted-foreground text-xs">Only include stages you've cleared</p>
                        </div>
                        <Switch checked={onlyCompletedStages ?? false} id="only-completed" onCheckedChange={setOnlyCompletedStages} />
                    </div>
                )}

                {/* Available Stages Filter */}
                {setOnlyAvailableStages && (
                    <div className="flex items-center justify-between rounded-lg border border-border/50 bg-secondary/30 p-3">
                        <div className="space-y-0.5">
                            <Label className="font-medium text-foreground text-sm" htmlFor="only-available">
                                Only Available Stages
                            </Label>
                            <p className="text-muted-foreground text-xs">Only show stages from currently open or permanent events</p>
                        </div>
                        <Switch checked={onlyAvailableStages ?? false} id="only-available" onCheckedChange={setOnlyAvailableStages} />
                    </div>
                )}

                {/* Zone Type Disclosures with smooth animation */}
                <div className="space-y-2">
                    {ZONE_TYPES.map((zoneType) => {
                        const isEnabled = safeAllowedZoneTypes.includes(zoneType.value);
                        const isOpen = openZoneTypes.includes(zoneType.value);
                        const stageCount = getStageCountForType(zoneType.value);
                        const selectedCount = getSelectedCountForType(zoneType.value);
                        const isActivityType = zoneType.value === "ACTIVITY";
                        const isMainlineType = zoneType.value === "MAINLINE";
                        const groupedEvents = isActivityType ? getGroupedEvents() : [];
                        const mainlineZones = isMainlineType ? getSortedMainlineZones() : [];

                        return (
                            <Disclosure key={zoneType.value} onOpenChange={() => toggleZoneTypeOpen(zoneType.value)} open={isOpen} transition={{ duration: 0.2, ease: "easeInOut" }}>
                                <div className={`rounded-lg border transition-colors ${isEnabled ? "border-primary/50 bg-primary/5" : "border-border/50 bg-card/30"}`}>
                                    <div className="flex items-center justify-between p-3">
                                        <DisclosureTrigger>
                                            <div className="flex flex-1 cursor-pointer items-center gap-2">
                                                <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${isOpen ? "rotate-180" : ""}`} />
                                                <div className="flex-1 text-left">
                                                    <div className="flex items-center gap-2">
                                                        <span className="font-medium text-foreground text-sm">{zoneType.label}</span>
                                                        <span className="rounded-full bg-muted px-2 py-0.5 text-muted-foreground text-xs">{selectedCount > 0 ? `${selectedCount} / ${stageCount}` : stageCount}</span>
                                                    </div>
                                                    <p className="mt-0.5 text-muted-foreground text-xs">{zoneType.description}</p>
                                                </div>
                                            </div>
                                        </DisclosureTrigger>
                                        <button
                                            className={`ml-2 flex h-6 w-6 shrink-0 items-center justify-center rounded-full transition-colors ${isEnabled ? "bg-primary" : "border border-border bg-secondary hover:bg-secondary/80"}`}
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                toggleZoneType(zoneType.value);
                                            }}
                                            type="button"
                                        >
                                            {isEnabled && <Check className="h-3.5 w-3.5 text-primary-foreground" />}
                                        </button>
                                    </div>

                                    <DisclosureContent>
                                        <div className="border-border/50 border-t px-3 pb-3">
                                            <div className="flex items-center justify-between py-2">
                                                <span className="text-muted-foreground text-xs">Manual Selection</span>
                                                <div className="flex gap-1">
                                                    <Button className="h-6 px-2 text-xs" onClick={() => selectAllStagesForType(zoneType.value)} size="sm" variant="ghost">
                                                        All
                                                    </Button>
                                                    <Button className="h-6 px-2 text-xs" onClick={() => deselectAllStagesForType(zoneType.value)} size="sm" variant="ghost">
                                                        None
                                                    </Button>
                                                </div>
                                            </div>

                                            {/* ACTIVITY zones: grouped by event */}
                                            {isActivityType && (
                                                <Accordion
                                                    className="w-full"
                                                    expandedValue={openEventId}
                                                    onValueChange={(value) => {
                                                        setOpenEventId(value as string | null);
                                                        setOpenZoneId(null);
                                                    }}
                                                    transition={{ duration: 0.2, ease: "easeInOut" }}
                                                >
                                                    {groupedEvents.map((event) => {
                                                        const eventSelectedCount = event.zones.reduce((sum, z) => sum + z.stages.filter((s) => safeSelectedStages.includes(s.stageId)).length, 0);
                                                        const hasMultipleZones = event.zones.length > 1;

                                                        return (
                                                            <AccordionItem className="border-border/50 border-b last:border-b-0" key={event.activityId} value={event.activityId}>
                                                                <AccordionTrigger className="flex w-full items-center justify-between py-2 text-sm">
                                                                    <div className="flex flex-col items-start gap-0.5 text-left">
                                                                        <span className="text-foreground">{event.eventName}</span>
                                                                        {event.dateRange && <span className="text-2.5 text-muted-foreground">{event.dateRange}</span>}
                                                                    </div>
                                                                    <div className="flex shrink-0 items-center gap-2">
                                                                        {event.isPermanent ? (
                                                                            <span className="whitespace-nowrap rounded-full bg-blue-500/20 px-1.5 py-0.5 font-medium text-2.5 text-blue-500">{event.permanentType === "BRANCHLINE" ? "INTERMEZZO" : "SIDE STORY"}</span>
                                                                        ) : event.isOpen ? (
                                                                            <span className="whitespace-nowrap rounded-full bg-green-500/20 px-1.5 py-0.5 font-medium text-2.5 text-green-500">OPEN</span>
                                                                        ) : null}
                                                                        <span className="rounded-full bg-muted px-2 py-0.5 text-muted-foreground text-xs">{eventSelectedCount > 0 ? `${eventSelectedCount} / ${event.totalStages}` : event.totalStages}</span>
                                                                        <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openEventId === event.activityId ? "rotate-180" : ""}`} />
                                                                    </div>
                                                                </AccordionTrigger>
                                                                <AccordionContent className="overflow-hidden">
                                                                    {hasMultipleZones ? (
                                                                        <Accordion className="w-full pl-2" expandedValue={openZoneId} onValueChange={(value) => setOpenZoneId(value as string | null)} transition={{ duration: 0.2, ease: "easeInOut" }}>
                                                                            {event.zones.map((zoneData) => {
                                                                                const zoneSelectedCount = zoneData.stages.filter((s) => safeSelectedStages.includes(s.stageId)).length;
                                                                                return (
                                                                                    <AccordionItem className="border-border/30 border-b last:border-b-0" key={zoneData.zoneId} value={zoneData.zoneId}>
                                                                                        <AccordionTrigger className="flex w-full items-center justify-between py-1.5 text-left text-sm">
                                                                                            <span className="text-muted-foreground text-xs">{zoneData.zoneName}</span>
                                                                                            <div className="flex items-center gap-2">
                                                                                                <span className="rounded-full bg-muted px-2 py-0.5 text-muted-foreground text-xs">{zoneSelectedCount > 0 ? `${zoneSelectedCount} / ${zoneData.stages.length}` : zoneData.stages.length}</span>
                                                                                                <ChevronDown className={`h-3 w-3 text-muted-foreground transition-transform duration-200 ${openZoneId === zoneData.zoneId ? "rotate-180" : ""}`} />
                                                                                            </div>
                                                                                        </AccordionTrigger>
                                                                                        <AccordionContent className="overflow-hidden">
                                                                                            <div className="max-h-48 space-y-1 overflow-y-auto pr-2 pb-2">
                                                                                                {zoneData.stages.map((stage) => {
                                                                                                    const isSelected = safeSelectedStages.includes(stage.stageId);
                                                                                                    const isChallengeMode = stage.difficulty === "FOUR_STAR";
                                                                                                    return (
                                                                                                        <button
                                                                                                            className={`w-full rounded px-3 py-2 text-left text-sm transition-colors ${isSelected ? "bg-primary/20 text-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                                                                                            key={stage.stageId}
                                                                                                            onClick={() => toggleStage(stage.stageId)}
                                                                                                            type="button"
                                                                                                        >
                                                                                                            <div className="flex items-center justify-between gap-2">
                                                                                                                <span className="flex items-center gap-1.5 truncate">
                                                                                                                    {getStageDisplayName(stage)}
                                                                                                                    {isChallengeMode && <span className="rounded bg-amber-500/20 px-1 py-0.5 font-medium text-2.5 text-amber-500">CM</span>}
                                                                                                                </span>
                                                                                                                {isSelected && <Check className="h-3 w-3 shrink-0 text-primary" />}
                                                                                                            </div>
                                                                                                        </button>
                                                                                                    );
                                                                                                })}
                                                                                            </div>
                                                                                        </AccordionContent>
                                                                                    </AccordionItem>
                                                                                );
                                                                            })}
                                                                        </Accordion>
                                                                    ) : (
                                                                        <div className="max-h-48 space-y-1 overflow-y-auto pr-2 pb-2">
                                                                            {event.zones[0]?.stages.map((stage) => {
                                                                                const isSelected = safeSelectedStages.includes(stage.stageId);
                                                                                const isChallengeMode = stage.difficulty === "FOUR_STAR";
                                                                                return (
                                                                                    <button
                                                                                        className={`w-full rounded px-3 py-2 text-left text-sm transition-colors ${isSelected ? "bg-primary/20 text-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                                                                        key={stage.stageId}
                                                                                        onClick={() => toggleStage(stage.stageId)}
                                                                                        type="button"
                                                                                    >
                                                                                        <div className="flex items-center justify-between gap-2">
                                                                                            <span className="flex items-center gap-1.5 truncate">
                                                                                                {getStageDisplayName(stage)}
                                                                                                {isChallengeMode && <span className="rounded bg-amber-500/20 px-1 py-0.5 font-medium text-2.5 text-amber-500">CM</span>}
                                                                                            </span>
                                                                                            {isSelected && <Check className="h-3 w-3 shrink-0 text-primary" />}
                                                                                        </div>
                                                                                    </button>
                                                                                );
                                                                            })}
                                                                        </div>
                                                                    )}
                                                                </AccordionContent>
                                                            </AccordionItem>
                                                        );
                                                    })}
                                                </Accordion>
                                            )}

                                            {/* MAINLINE zones: simple list */}
                                            {isMainlineType && (
                                                <Accordion className="w-full" expandedValue={openZoneId} onValueChange={(value) => setOpenZoneId(value as string | null)} transition={{ duration: 0.2, ease: "easeInOut" }}>
                                                    {mainlineZones.map(({ zoneId, zone, stages: zoneStages }) => {
                                                        const zoneName = zone?.zoneNameFirst ?? zone?.zoneNameSecond ?? zoneId;
                                                        const selectedInZone = zoneStages.filter((s) => safeSelectedStages.includes(s.stageId)).length;

                                                        return (
                                                            <AccordionItem className="border-border/50 border-b last:border-b-0" key={zoneId} value={zoneId}>
                                                                <AccordionTrigger className="flex w-full items-center justify-between py-2 text-sm">
                                                                    <span className="text-foreground">{zoneName}</span>
                                                                    <div className="flex items-center gap-2">
                                                                        <span className="rounded-full bg-muted px-2 py-0.5 text-muted-foreground text-xs">{selectedInZone > 0 ? `${selectedInZone} / ${zoneStages.length}` : zoneStages.length}</span>
                                                                        <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openZoneId === zoneId ? "rotate-180" : ""}`} />
                                                                    </div>
                                                                </AccordionTrigger>
                                                                <AccordionContent className="overflow-hidden">
                                                                    <div className="max-h-48 space-y-1 overflow-y-auto pr-2 pb-2">
                                                                        {zoneStages.map((stage) => {
                                                                            const isSelected = safeSelectedStages.includes(stage.stageId);
                                                                            const isChallengeMode = stage.difficulty === "FOUR_STAR";
                                                                            return (
                                                                                <button
                                                                                    className={`w-full rounded px-3 py-2 text-left text-sm transition-colors ${isSelected ? "bg-primary/20 text-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                                                                    key={stage.stageId}
                                                                                    onClick={() => toggleStage(stage.stageId)}
                                                                                    type="button"
                                                                                >
                                                                                    <div className="flex items-center justify-between gap-2">
                                                                                        <span className="flex items-center gap-1.5 truncate">
                                                                                            {getStageDisplayName(stage)}
                                                                                            {isChallengeMode && <span className="rounded bg-amber-500/20 px-1 py-0.5 font-medium text-2.5 text-amber-500">CM</span>}
                                                                                        </span>
                                                                                        {isSelected && <Check className="h-3 w-3 shrink-0 text-primary" />}
                                                                                    </div>
                                                                                </button>
                                                                            );
                                                                        })}
                                                                    </div>
                                                                </AccordionContent>
                                                            </AccordionItem>
                                                        );
                                                    })}
                                                </Accordion>
                                            )}
                                        </div>
                                    </DisclosureContent>
                                </div>
                            </Disclosure>
                        );
                    })}
                </div>
            </CardContent>
        </Card>
    );
}
