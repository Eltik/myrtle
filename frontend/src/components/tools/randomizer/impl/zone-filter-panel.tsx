"use client";

import { Check } from "lucide-react";
import { useMemo } from "react";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Label } from "~/components/ui/shadcn/label";
import { Switch } from "~/components/ui/shadcn/switch";
import type { Stage } from "~/types/api/impl/stage";
import type { User } from "~/types/api/impl/user";
import type { Zone } from "~/types/api/impl/zone";

interface ZoneFilterPanelProps {
    allowedZoneTypes: string[];
    setAllowedZoneTypes: (types: string[]) => void;
    hasProfile?: boolean;
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

// Natural sort function for stage codes (e.g., "1-1" < "1-2" < "1-10")
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

export function ZoneFilterPanel({ allowedZoneTypes, setAllowedZoneTypes, hasProfile, onlyCompletedStages, setOnlyCompletedStages, stages, zones, selectedStages, setSelectedStages, user }: ZoneFilterPanelProps) {
    const safeAllowedZoneTypes = allowedZoneTypes ?? [];
    const safeSelectedStages = selectedStages ?? [];

    const availableStages = useMemo(() => {
        let filtered = stages;

        // Filter by zone type
        if (safeAllowedZoneTypes.length > 0) {
            filtered = filtered.filter((stage) => {
                const zone = zones.find((z) => z.zoneId === stage.zoneId);
                return zone && safeAllowedZoneTypes.includes(zone.type);
            });
        }

        // Filter by completion status
        if (onlyCompletedStages && user) {
            filtered = filtered.filter((stage) => {
                const stageData = user.dungeon.stages[stage.stageId];
                return stageData && stageData.completeTimes > 0;
            });
        }

        return filtered;
    }, [stages, zones, safeAllowedZoneTypes, onlyCompletedStages, user]);

    const stagesByZone = useMemo(() => {
        const grouped = new Map<string, Stage[]>();
        for (const stage of availableStages) {
            const existing = grouped.get(stage.zoneId) ?? [];
            existing.push(stage);
            grouped.set(stage.zoneId, existing);
        }
        // Sort stages within each zone by code
        for (const [zoneId, zoneStages] of grouped.entries()) {
            grouped.set(
                zoneId,
                zoneStages.sort((a, b) => naturalSortCompare(a.code, b.code)),
            );
        }
        return grouped;
    }, [availableStages]);

    const filteredSelectedCount = useMemo(() => {
        return safeSelectedStages.filter((stageId) => availableStages.some((s) => s.stageId === stageId)).length;
    }, [safeSelectedStages, availableStages]);

    const toggleZoneType = (type: string) => {
        if (safeAllowedZoneTypes.includes(type)) {
            // Don't allow deselecting if it's the last one
            if (safeAllowedZoneTypes.length > 1) {
                setAllowedZoneTypes(safeAllowedZoneTypes.filter((t) => t !== type));
            }
        } else {
            setAllowedZoneTypes([...safeAllowedZoneTypes, type]);
        }
    };

    const selectAll = () => {
        setAllowedZoneTypes(ZONE_TYPES.map((zt) => zt.value));
    };

    const toggleStage = (stageId: string) => {
        if (safeSelectedStages.includes(stageId)) {
            setSelectedStages(safeSelectedStages.filter((id) => id !== stageId));
        } else {
            setSelectedStages([...safeSelectedStages, stageId]);
        }
    };

    const selectAllStages = () => {
        setSelectedStages(availableStages.map((s) => s.stageId));
    };

    const deselectAllStages = () => {
        setSelectedStages([]);
    };

    const allSelected = safeAllowedZoneTypes.length === ZONE_TYPES.length;

    return (
        <Card className="border-border/50 bg-card/50 backdrop-blur-sm">
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div>
                        <CardTitle className="font-semibold text-foreground text-lg">Zone & Stage Filters</CardTitle>
                        <CardDescription className="text-muted-foreground text-sm">Filter which zones and stages can be selected</CardDescription>
                    </div>
                    <Button className="bg-transparent text-xs" disabled={allSelected} onClick={selectAll} size="sm" variant="outline">
                        Select All Types
                    </Button>
                </div>
            </CardHeader>
            <CardContent>
                <div className="space-y-4">
                    {/* Zone Types */}
                    <div className="space-y-2">
                        {ZONE_TYPES.map((zoneType) => {
                            const isSelected = safeAllowedZoneTypes.includes(zoneType.value);
                            return (
                                <button className={`w-full rounded-lg border px-4 py-3 text-left transition-all ${isSelected ? "border-primary/50 bg-primary/10" : "border-border/50 bg-card/30 hover:border-border hover:bg-card/50"}`} key={zoneType.value} onClick={() => toggleZoneType(zoneType.value)} type="button">
                                    <div className="flex items-start justify-between gap-3">
                                        <div className="flex-1">
                                            <div className="flex items-center gap-2">
                                                <span className="font-medium text-foreground text-sm">{zoneType.label}</span>
                                            </div>
                                            <p className="mt-0.5 text-muted-foreground text-xs">{zoneType.description}</p>
                                        </div>
                                        {isSelected && (
                                            <div className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-primary">
                                                <Check className="h-3 w-3 text-primary-foreground" />
                                            </div>
                                        )}
                                    </div>
                                </button>
                            );
                        })}
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

                    <div className="space-y-2 rounded-lg border border-border/50 bg-secondary/30 p-3">
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label className="font-medium text-foreground text-sm">Manual Stage Selection</Label>
                                <p className="text-muted-foreground text-xs">{safeSelectedStages.length > 0 ? `${filteredSelectedCount} stages selected` : `${availableStages.length} stages available`}</p>
                            </div>
                            <div className="flex gap-2">
                                <Button className="h-7 text-xs" onClick={selectAllStages} size="sm" variant="ghost">
                                    All
                                </Button>
                                <Button className="h-7 text-xs" onClick={deselectAllStages} size="sm" variant="ghost">
                                    None
                                </Button>
                            </div>
                        </div>

                        <Accordion className="w-full" collapsible type="single">
                            {Array.from(stagesByZone.entries())
                                .sort(([aZoneId], [bZoneId]) => {
                                    const aZone = zones.find((z) => z.zoneId === aZoneId);
                                    const bZone = zones.find((z) => z.zoneId === bZoneId);
                                    if (!aZone || !bZone) return aZoneId.localeCompare(bZoneId);

                                    // Sort by zone type order first (MAINLINE, SIDESTORY, BRANCHLINE, ACTIVITY)
                                    const typeOrder = ZONE_TYPES.map((t) => t.value);
                                    const aTypeIndex = typeOrder.indexOf(aZone.type);
                                    const bTypeIndex = typeOrder.indexOf(bZone.type);
                                    if (aTypeIndex !== bTypeIndex) {
                                        return aTypeIndex - bTypeIndex;
                                    }

                                    // Then sort by zoneIndex within the same type
                                    return aZone.zoneIndex - bZone.zoneIndex;
                                })
                                .map(([zoneId, zoneStages]) => {
                                    const zone = zones.find((z) => z.zoneId === zoneId);
                                    const zoneName = zone?.zoneNameFirst ?? zone?.zoneNameSecond ?? zoneId;
                                    const selectedInZone = zoneStages.filter((s) => safeSelectedStages.includes(s.stageId)).length;

                                    return (
                                        <AccordionItem className="border-border/50" key={zoneId} value={zoneId}>
                                            <AccordionTrigger className="py-2 text-sm hover:no-underline">
                                                <div className="flex w-full items-center justify-between pr-2">
                                                    <span className="text-foreground">{zoneName}</span>
                                                    <span className="rounded-full bg-muted px-2 py-0.5 text-muted-foreground text-xs">{selectedInZone > 0 ? `${selectedInZone} / ${zoneStages.length}` : zoneStages.length}</span>
                                                </div>
                                            </AccordionTrigger>
                                            <AccordionContent>
                                                <div className="max-h-48 space-y-1 overflow-y-auto pr-2">
                                                    {zoneStages.map((stage) => {
                                                        const isSelected = safeSelectedStages.includes(stage.stageId);
                                                        return (
                                                            <button
                                                                className={`w-full rounded px-3 py-2 text-left text-sm transition-colors ${isSelected ? "bg-primary/20 text-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground"}`}
                                                                key={stage.stageId}
                                                                onClick={() => toggleStage(stage.stageId)}
                                                                type="button"
                                                            >
                                                                <div className="flex items-center justify-between gap-2">
                                                                    <span className="truncate">{stage.code}</span>
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
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
