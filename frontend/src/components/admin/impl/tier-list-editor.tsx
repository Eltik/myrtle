"use client";

import { closestCenter, DndContext, type DragEndEvent, type DragOverEvent, DragOverlay, type DragStartEvent, KeyboardSensor, PointerSensor, useDroppable, useSensor, useSensors } from "@dnd-kit/core";
import { arrayMove, rectSortingStrategy, SortableContext, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ArrowLeft, Check, ChevronDown, ChevronUp, GripVertical, Plus, Save, Settings, Trash2, X } from "lucide-react";
import Image from "next/image";
import { useMemo, useState } from "react";
import { cn, rarityToNumber } from "~/lib/utils";
import type { Tier, TierListResponse, TierPlacement, TierWithPlacements } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "../../ui/motion-primitives/disclosure";
import { Badge } from "../../ui/shadcn/badge";
import { Button } from "../../ui/shadcn/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/shadcn/card";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "../../ui/shadcn/dialog";
import { Input } from "../../ui/shadcn/input";
import { Label } from "../../ui/shadcn/label";
import { Popover, PopoverContent, PopoverTrigger } from "../../ui/shadcn/popover";
import { ScrollArea } from "../../ui/shadcn/scroll-area";
import { Separator } from "../../ui/shadcn/separator";
import { Switch } from "../../ui/shadcn/switch";
import { Textarea } from "../../ui/shadcn/textarea";

// Default tier colors
const DEFAULT_TIER_COLORS: Record<string, string> = {
    "S+": "#ff7f7f",
    S: "#ff9f7f",
    "A+": "#ffbf7f",
    A: "#ffdf7f",
    "B+": "#ffff7f",
    B: "#bfff7f",
    C: "#7fff7f",
    D: "#7fffff",
};

const PRESET_COLORS = ["#ff7f7f", "#ff9f7f", "#ffbf7f", "#ffdf7f", "#ffff7f", "#bfff7f", "#7fff7f", "#7fffff", "#7fbfff", "#7f7fff", "#bf7fff", "#ff7fbf"];

const RARITY_COLORS: Record<number, string> = {
    1: "#9e9e9e",
    2: "#dce537",
    3: "#00b2eb",
    4: "#dbb1db",
    5: "#ffcc00",
    6: "#ff6600",
};

interface TierListEditorProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
    allOperators: OperatorFromList[];
    operatorsLoading?: boolean;
    onBack: () => void;
    onSave?: (data: TierListResponse) => Promise<void>;
}

interface EditableTier extends TierWithPlacements {
    isNew?: boolean;
    isModified?: boolean;
}

// Sortable operator card component
interface SortableOperatorCardProps {
    placement: TierPlacement;
    operator: OperatorFromList;
    onRemove: () => void;
}

function SortableOperatorCard({ placement, operator, onRemove }: SortableOperatorCardProps) {
    const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({ id: placement.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0.5 : 1,
    };

    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";

    return (
        <div className="group relative aspect-square overflow-hidden rounded-md border bg-card" ref={setNodeRef} style={style}>
            {/* Drag handle overlay */}
            <div className="absolute inset-0 z-10 cursor-grab active:cursor-grabbing" {...attributes} {...listeners} />

            <Image alt={operator.name} className="h-full w-full object-cover" fill src={`/api/cdn${operator.portrait}`} />

            {/* Vignette overlay for name visibility */}
            <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-black/70 to-transparent opacity-0 transition-opacity group-hover:opacity-100" />

            <div className="absolute bottom-0 h-1 w-full" style={{ backgroundColor: rarityColor }} />

            {/* Remove button - top right */}
            <Button
                className="absolute top-0.5 right-0.5 z-20 h-5 w-5 rounded-full bg-destructive/90 p-0 opacity-0 transition-opacity hover:bg-destructive group-hover:opacity-100"
                onClick={(e) => {
                    e.stopPropagation();
                    onRemove();
                }}
                size="icon"
                variant="ghost"
            >
                <X className="h-3 w-3 text-white" />
            </Button>

            {/* Operator name */}
            <div className="absolute inset-x-0 bottom-1.5 z-20 flex justify-center px-0.5 text-center font-medium text-[10px] text-white opacity-0 transition-opacity group-hover:opacity-100">
                <span className="truncate">{operator.name}</span>
            </div>

            {/* Drag indicator */}
            <div className="pointer-events-none absolute top-0.5 left-0.5 flex h-5 w-5 items-center justify-center rounded bg-background/60 opacity-0 transition-opacity group-hover:opacity-100">
                <GripVertical className="h-3 w-3 text-muted-foreground" />
            </div>
        </div>
    );
}

// Drag overlay component (shown while dragging)
interface DragOverlayCardProps {
    operator: OperatorFromList;
}

function DragOverlayCard({ operator }: DragOverlayCardProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";

    return (
        <div className="relative aspect-square w-16 overflow-hidden rounded-md border bg-card shadow-lg ring-2 ring-primary">
            <Image alt={operator.name} className="h-full w-full object-cover" fill src={`/api/cdn${operator.portrait}`} />
            <div className="absolute bottom-0 h-1 w-full" style={{ backgroundColor: rarityColor }} />
        </div>
    );
}

// Sortable tier row component - uses prefixed ID to distinguish from operator placements
interface SortableTierRowProps {
    tier: EditableTier;
    children: (dragHandleProps: React.HTMLAttributes<HTMLDivElement>) => React.ReactNode;
}

function SortableTierRow({ tier, children }: SortableTierRowProps) {
    const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
        id: `tier-row-${tier.id}`,
    });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isDragging ? 0.5 : 1,
    };

    return (
        <div ref={setNodeRef} style={style}>
            {children({ ...attributes, ...listeners })}
        </div>
    );
}

// Droppable tier zone for cross-tier dragging
interface DroppableTierZoneProps {
    tierId: string;
    isOver: boolean;
    children: React.ReactNode;
}

function DroppableTierZone({ tierId, isOver, children }: DroppableTierZoneProps) {
    const { setNodeRef } = useDroppable({
        id: `tier-drop-${tierId}`,
        data: { tierId },
    });

    return (
        <div className={cn("min-h-[60px] rounded-md p-4 transition-colors", isOver && "bg-primary/10 ring-2 ring-primary ring-inset")} ref={setNodeRef}>
            {children}
        </div>
    );
}

export function TierListEditor({ tierListData, operatorsData, allOperators, operatorsLoading = false, onBack, onSave }: TierListEditorProps) {
    const [tiers, setTiers] = useState<EditableTier[]>(tierListData.tiers.map((t) => ({ ...t })));
    const [tierListName, setTierListName] = useState(tierListData.tier_list.name);
    const [tierListDescription, setTierListDescription] = useState(tierListData.tier_list.description ?? "");
    const [isActive, setIsActive] = useState(tierListData.tier_list.is_active);
    const [saving, setSaving] = useState(false);
    const [addOperatorDialogOpen, setAddOperatorDialogOpen] = useState(false);
    const [selectedTierForAdd, setSelectedTierForAdd] = useState<string | null>(null);
    const [operatorSearch, setOperatorSearch] = useState("");
    const [expandedTiers, setExpandedTiers] = useState<Set<string>>(new Set(tiers.map((t) => t.id)));
    const [activeDragId, setActiveDragId] = useState<string | null>(null);
    const [activeDragType, setActiveDragType] = useState<"operator" | "tier" | null>(null);
    const [overTierId, setOverTierId] = useState<string | null>(null);

    // Drag and drop sensors
    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 8,
            },
        }),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        }),
    );

    // Get operators already in the tier list with their tier info
    const placedOperators = useMemo(() => {
        const map = new Map<string, { tierId: string; tierName: string }>();
        for (const tier of tiers) {
            for (const placement of tier.placements) {
                map.set(placement.operator_id, { tierId: tier.id, tierName: tier.name });
            }
        }
        return map;
    }, [tiers]);

    // Map placement IDs to their tier IDs for quick lookup during drag
    const placementToTierMap = useMemo(() => {
        const map = new Map<string, string>();
        for (const tier of tiers) {
            for (const placement of tier.placements) {
                map.set(placement.id, tier.id);
            }
        }
        return map;
    }, [tiers]);

    // Filter operators for dialog (show all, mark placed ones)
    const filteredOperators = useMemo(() => {
        return allOperators
            .filter((op) => operatorSearch === "" || op.name.toLowerCase().includes(operatorSearch.toLowerCase()))
            .sort((a, b) => {
                const rarityA = rarityToNumber(a.rarity);
                const rarityB = rarityToNumber(b.rarity);
                if (rarityA !== rarityB) return rarityB - rarityA;
                return a.name.localeCompare(b.name);
            });
    }, [allOperators, operatorSearch]);

    const toggleTierExpanded = (tierId: string) => {
        setExpandedTiers((prev) => {
            const next = new Set(prev);
            if (next.has(tierId)) {
                next.delete(tierId);
            } else {
                next.add(tierId);
            }
            return next;
        });
    };

    const handleAddTier = () => {
        const newTier: EditableTier = {
            id: `new-${Date.now()}`,
            tier_list_id: tierListData.tier_list.id,
            name: `New Tier`,
            display_order: tiers.length,
            color: PRESET_COLORS[tiers.length % PRESET_COLORS.length] ?? "#888888",
            description: null,
            placements: [],
            isNew: true,
            isModified: true,
        };
        setTiers([...tiers, newTier]);
        setExpandedTiers((prev) => new Set([...prev, newTier.id]));
    };

    const handleUpdateTier = (tierId: string, updates: Partial<Tier>) => {
        setTiers(
            tiers.map((tier) =>
                tier.id === tierId
                    ? {
                          ...tier,
                          ...updates,
                          isModified: true,
                      }
                    : tier,
            ),
        );
    };

    const handleDeleteTier = (tierId: string) => {
        setTiers(tiers.filter((tier) => tier.id !== tierId));
    };

    const handleAddOperatorToTier = (tierId: string, operator: OperatorFromList) => {
        setTiers(
            tiers.map((tier) => {
                if (tier.id !== tierId) return tier;
                const newPlacement: TierPlacement = {
                    id: `new-${Date.now()}-${operator.id}`,
                    tier_id: tierId,
                    operator_id: operator.id ?? "",
                    sub_order: tier.placements.length,
                    notes: null,
                    created_at: new Date().toISOString(),
                    updated_at: new Date().toISOString(),
                };
                return {
                    ...tier,
                    placements: [...tier.placements, newPlacement],
                    isModified: true,
                };
            }),
        );
    };

    const handleRemoveOperatorFromTier = (tierId: string, placementId: string) => {
        setTiers(
            tiers.map((tier) => {
                if (tier.id !== tierId) return tier;
                return {
                    ...tier,
                    placements: tier.placements.filter((p) => p.id !== placementId),
                    isModified: true,
                };
            }),
        );
    };

    // Unified drag handlers for single DndContext
    const handleDragStart = (event: DragStartEvent) => {
        const id = event.active.id as string;
        setActiveDragId(id);

        if (id.startsWith("tier-row-")) {
            setActiveDragType("tier");
        } else {
            setActiveDragType("operator");
        }
    };

    const handleDragOver = (event: DragOverEvent) => {
        const { over } = event;

        // Only track tier hover for operator drags
        if (activeDragType !== "operator") {
            setOverTierId(null);
            return;
        }

        if (!over) {
            setOverTierId(null);
            return;
        }

        const overId = over.id as string;

        // Check if hovering over a tier drop zone
        if (overId.startsWith("tier-drop-")) {
            const tierId = overId.replace("tier-drop-", "");
            setOverTierId(tierId);
        } else if (!overId.startsWith("tier-row-")) {
            // Hovering over another placement - find its tier
            const tierId = placementToTierMap.get(overId);
            setOverTierId(tierId ?? null);
        } else {
            setOverTierId(null);
        }
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        const dragType = activeDragType;

        setActiveDragId(null);
        setActiveDragType(null);
        setOverTierId(null);

        if (!over) return;

        const activeId = active.id as string;
        const overId = over.id as string;

        // Handle tier row reordering
        if (dragType === "tier") {
            if (activeId === overId) return;

            const actualActiveId = activeId.replace("tier-row-", "");
            const actualOverId = overId.replace("tier-row-", "");

            const oldIndex = tiers.findIndex((t) => t.id === actualActiveId);
            const newIndex = tiers.findIndex((t) => t.id === actualOverId);

            if (oldIndex === -1 || newIndex === -1) return;

            const newTiers = arrayMove(tiers, oldIndex, newIndex).map((tier, i) => ({
                ...tier,
                display_order: i,
                isModified: true,
            }));

            setTiers(newTiers);
            return;
        }

        // Handle operator placement reordering/moving
        const sourceTierId = placementToTierMap.get(activeId);
        if (!sourceTierId) return;

        // Determine target tier
        let targetTierId: string | null = null;
        let targetPlacementId: string | null = null;

        if (overId.startsWith("tier-drop-")) {
            targetTierId = overId.replace("tier-drop-", "");
        } else if (overId.startsWith("tier-row-")) {
            // Dropped on tier row - ignore
            return;
        } else {
            targetTierId = placementToTierMap.get(overId) ?? null;
            targetPlacementId = overId;
        }

        if (!targetTierId) return;

        // Same tier - reorder within tier
        if (sourceTierId === targetTierId) {
            if (activeId === overId) return;

            setTiers(
                tiers.map((tier) => {
                    if (tier.id !== sourceTierId) return tier;

                    const sortedPlacements = [...tier.placements].sort((a, b) => a.sub_order - b.sub_order);
                    const oldIndex = sortedPlacements.findIndex((p) => p.id === activeId);
                    const newIndex = targetPlacementId ? sortedPlacements.findIndex((p) => p.id === targetPlacementId) : sortedPlacements.length;

                    if (oldIndex === -1 || newIndex === -1) return tier;

                    const newPlacements = arrayMove(sortedPlacements, oldIndex, newIndex);

                    return {
                        ...tier,
                        placements: newPlacements.map((p, i) => ({ ...p, sub_order: i })),
                        isModified: true,
                    };
                }),
            );
        } else {
            // Cross-tier move
            setTiers(
                tiers.map((tier) => {
                    if (tier.id === sourceTierId) {
                        const newPlacements = tier.placements
                            .filter((p) => p.id !== activeId)
                            .sort((a, b) => a.sub_order - b.sub_order)
                            .map((p, i) => ({ ...p, sub_order: i }));
                        return { ...tier, placements: newPlacements, isModified: true };
                    }

                    if (tier.id === targetTierId) {
                        const sourceTier = tiers.find((t) => t.id === sourceTierId);
                        const movedPlacement = sourceTier?.placements.find((p) => p.id === activeId);
                        if (!movedPlacement) return tier;

                        const sortedPlacements = [...tier.placements].sort((a, b) => a.sub_order - b.sub_order);

                        let insertIndex = sortedPlacements.length;
                        if (targetPlacementId) {
                            const targetIndex = sortedPlacements.findIndex((p) => p.id === targetPlacementId);
                            if (targetIndex !== -1) insertIndex = targetIndex;
                        }

                        const updatedPlacement: TierPlacement = {
                            ...movedPlacement,
                            tier_id: targetTierId,
                            id: movedPlacement.id.startsWith("new-") ? movedPlacement.id : `new-moved-${Date.now()}-${movedPlacement.operator_id}`,
                        };

                        const newPlacements = [...sortedPlacements];
                        newPlacements.splice(insertIndex, 0, updatedPlacement);

                        return {
                            ...tier,
                            placements: newPlacements.map((p, i) => ({ ...p, sub_order: i })),
                            isModified: true,
                        };
                    }

                    return tier;
                }),
            );
        }
    };

    const handleSave = async () => {
        if (!onSave) return;
        setSaving(true);
        try {
            const updatedData: TierListResponse = {
                tier_list: {
                    ...tierListData.tier_list,
                    name: tierListName,
                    description: tierListDescription || null,
                    is_active: isActive,
                    updated_at: new Date().toISOString(),
                },
                tiers: tiers.map((tier, index) => ({
                    ...tier,
                    display_order: index,
                    // Sort placements by sub_order and re-index to ensure correct order is saved
                    placements: [...tier.placements]
                        .sort((a, b) => a.sub_order - b.sub_order)
                        .map((p, pIndex) => ({
                            ...p,
                            sub_order: pIndex,
                        })),
                })),
            };
            await onSave(updatedData);
        } finally {
            setSaving(false);
        }
    };

    const hasChanges = useMemo(() => {
        if (tierListName !== tierListData.tier_list.name) return true;
        if (tierListDescription !== (tierListData.tier_list.description ?? "")) return true;
        if (isActive !== tierListData.tier_list.is_active) return true;
        return tiers.some((t) => t.isModified || t.isNew);
    }, [tierListName, tierListDescription, isActive, tiers, tierListData]);

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-3">
                    <Button onClick={onBack} size="icon" variant="ghost">
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div>
                        <h2 className="font-semibold text-xl">{tierListName}</h2>
                        <p className="text-muted-foreground text-sm">Edit tier list configuration</p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    {hasChanges && <Badge variant="outline">Unsaved changes</Badge>}
                    <Button disabled={!hasChanges || saving} onClick={handleSave} variant="default">
                        <Save className="mr-2 h-4 w-4" />
                        {saving ? "Saving..." : "Save Changes"}
                    </Button>
                </div>
            </div>

            {/* Tier List Settings */}
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center gap-2">
                        <Settings className="h-5 w-5 text-muted-foreground" />
                        <CardTitle className="text-base">Tier List Settings</CardTitle>
                    </div>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="grid gap-4 sm:grid-cols-2">
                        <div className="space-y-2">
                            <Label htmlFor="name">Name</Label>
                            <Input id="name" onChange={(e) => setTierListName(e.target.value)} placeholder="Tier list name" value={tierListName} />
                        </div>
                        <div className="flex items-center justify-between space-x-2 rounded-lg border p-4">
                            <div className="space-y-0.5">
                                <Label>Active Status</Label>
                                <p className="text-muted-foreground text-sm">Make this tier list publicly visible</p>
                            </div>
                            <Switch checked={isActive} onCheckedChange={setIsActive} />
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label htmlFor="description">Description</Label>
                        <Textarea id="description" onChange={(e) => setTierListDescription(e.target.value)} placeholder="Optional description..." rows={2} value={tierListDescription} />
                    </div>
                </CardContent>
            </Card>

            {/* Tiers Management */}
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-base">Tiers ({tiers.length})</CardTitle>
                        <Button onClick={handleAddTier} size="sm" variant="outline">
                            <Plus className="mr-2 h-4 w-4" />
                            Add Tier
                        </Button>
                    </div>
                </CardHeader>
                <CardContent className="space-y-0 p-0">
                    {tiers.length > 0 ? (
                        /* Single DndContext for both tier rows and operators */
                        <DndContext collisionDetection={closestCenter} onDragEnd={handleDragEnd} onDragOver={handleDragOver} onDragStart={handleDragStart} sensors={sensors}>
                            {/* SortableContext for tier rows */}
                            <SortableContext items={tiers.map((t) => `tier-row-${t.id}`)} strategy={verticalListSortingStrategy}>
                                {/* SortableContext for all operator placements */}
                                <SortableContext items={tiers.flatMap((t) => t.placements.map((p) => p.id))} strategy={rectSortingStrategy}>
                                    <div className="divide-y">
                                        {tiers.map((tier) => {
                                            const tierColor = tier.color || DEFAULT_TIER_COLORS[tier.name] || "#888888";
                                            const isExpanded = expandedTiers.has(tier.id);
                                            const sourceTierId = activeDragId ? placementToTierMap.get(activeDragId) : null;
                                            const isDropTarget = overTierId === tier.id && sourceTierId !== tier.id;

                                            return (
                                                <SortableTierRow key={tier.id} tier={tier}>
                                                    {(dragHandleProps) => (
                                                        <>
                                                            {/* Tier Header */}
                                                            <div className="flex items-center gap-2 bg-muted/30 px-4 py-2">
                                                                {/* Drag handle */}
                                                                <div className="flex h-7 w-7 cursor-grab items-center justify-center rounded-md hover:bg-muted active:cursor-grabbing" {...dragHandleProps}>
                                                                    <GripVertical className="h-4 w-4 text-muted-foreground" />
                                                                </div>

                                                                {/* Color block with popover */}
                                                                <Popover>
                                                                    <PopoverTrigger asChild>
                                                                        <button className="h-8 w-12 cursor-pointer rounded transition-all hover:ring-2 hover:ring-primary hover:ring-offset-2" style={{ backgroundColor: tierColor }} type="button" />
                                                                    </PopoverTrigger>
                                                                    <PopoverContent className="w-auto p-3">
                                                                        <div className="grid grid-cols-6 gap-2">
                                                                            {PRESET_COLORS.map((color) => (
                                                                                <button className={cn("h-6 w-6 rounded border-2", tier.color === color ? "border-foreground" : "border-transparent")} key={color} onClick={() => handleUpdateTier(tier.id, { color })} style={{ backgroundColor: color }} type="button" />
                                                                            ))}
                                                                        </div>
                                                                        <Separator className="my-2" />
                                                                        <Input className="h-8" onChange={(e) => handleUpdateTier(tier.id, { color: e.target.value })} placeholder="#000000" type="text" value={tier.color ?? ""} />
                                                                    </PopoverContent>
                                                                </Popover>

                                                                <Input className="h-8 w-24 font-semibold" onChange={(e) => handleUpdateTier(tier.id, { name: e.target.value })} value={tier.name} />

                                                                <Input className="hidden h-8 flex-1 sm:block" onChange={(e) => handleUpdateTier(tier.id, { description: e.target.value || null })} placeholder="Description (optional)" value={tier.description ?? ""} />

                                                                <Badge variant="secondary">{tier.placements.length} ops</Badge>

                                                                <Button className="h-8 w-8" onClick={() => toggleTierExpanded(tier.id)} size="icon" variant="ghost">
                                                                    {isExpanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                                                                </Button>

                                                                <Button className="h-8 w-8 text-destructive hover:text-destructive" onClick={() => handleDeleteTier(tier.id)} size="icon" variant="ghost">
                                                                    <Trash2 className="h-4 w-4" />
                                                                </Button>
                                                            </div>

                                                            {/* Tier Content */}
                                                            <Disclosure open={isExpanded}>
                                                                <DisclosureTrigger>
                                                                    <span className="sr-only">Toggle tier content</span>
                                                                </DisclosureTrigger>
                                                                <DisclosureContent>
                                                                    <DroppableTierZone isOver={isDropTarget} tierId={tier.id}>
                                                                        {tier.placements.length > 0 ? (
                                                                            <div className="grid grid-cols-5 gap-2 sm:grid-cols-8 md:grid-cols-10 lg:grid-cols-12">
                                                                                {[...tier.placements]
                                                                                    .sort((a, b) => a.sub_order - b.sub_order)
                                                                                    .map((placement) => {
                                                                                        const operator = operatorsData[placement.operator_id];
                                                                                        if (!operator) return null;

                                                                                        return <SortableOperatorCard key={placement.id} onRemove={() => handleRemoveOperatorFromTier(tier.id, placement.id)} operator={operator} placement={placement} />;
                                                                                    })}

                                                                                {/* Add operator button */}
                                                                                <button
                                                                                    className="flex aspect-square items-center justify-center rounded-md border-2 border-muted-foreground/30 border-dashed transition-colors hover:border-primary hover:bg-primary/5"
                                                                                    onClick={() => {
                                                                                        setSelectedTierForAdd(tier.id);
                                                                                        setAddOperatorDialogOpen(true);
                                                                                    }}
                                                                                    type="button"
                                                                                >
                                                                                    <Plus className="h-6 w-6 text-muted-foreground" />
                                                                                </button>
                                                                            </div>
                                                                        ) : (
                                                                            <div className="flex flex-col items-center justify-center gap-2 py-4">
                                                                                <p className="text-muted-foreground text-sm">No operators in this tier</p>
                                                                                <Button
                                                                                    onClick={() => {
                                                                                        setSelectedTierForAdd(tier.id);
                                                                                        setAddOperatorDialogOpen(true);
                                                                                    }}
                                                                                    size="sm"
                                                                                    variant="outline"
                                                                                >
                                                                                    <Plus className="mr-2 h-4 w-4" />
                                                                                    Add Operators
                                                                                </Button>
                                                                            </div>
                                                                        )}
                                                                    </DroppableTierZone>
                                                                </DisclosureContent>
                                                            </Disclosure>
                                                        </>
                                                    )}
                                                </SortableTierRow>
                                            );
                                        })}
                                    </div>
                                </SortableContext>
                            </SortableContext>

                            {/* Drag overlay */}
                            <DragOverlay>
                                {activeDragId && activeDragType === "tier"
                                    ? (() => {
                                          const tierId = activeDragId.replace("tier-row-", "");
                                          const tier = tiers.find((t) => t.id === tierId);
                                          return tier ? (
                                              <div className="rounded-md border bg-card shadow-lg">
                                                  <div className="flex items-center gap-2 bg-muted/30 px-4 py-2">
                                                      <GripVertical className="h-4 w-4 text-muted-foreground" />
                                                      <div className="h-8 w-12 rounded" style={{ backgroundColor: tier.color || "#888888" }} />
                                                      <span className="font-semibold">{tier.name}</span>
                                                  </div>
                                              </div>
                                          ) : null;
                                      })()
                                    : activeDragId && activeDragType === "operator"
                                      ? (() => {
                                            const tier = tiers.find((t) => t.placements.some((p) => p.id === activeDragId));
                                            const placement = tier?.placements.find((p) => p.id === activeDragId);
                                            const operator = placement ? operatorsData[placement.operator_id] : null;
                                            return operator ? <DragOverlayCard operator={operator} /> : null;
                                        })()
                                      : null}
                            </DragOverlay>
                        </DndContext>
                    ) : (
                        <div className="flex flex-col items-center justify-center gap-2 py-12">
                            <p className="text-muted-foreground">No tiers configured</p>
                            <Button onClick={handleAddTier} variant="outline">
                                <Plus className="mr-2 h-4 w-4" />
                                Add First Tier
                            </Button>
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Add Operator Dialog */}
            <Dialog onOpenChange={setAddOperatorDialogOpen} open={addOperatorDialogOpen}>
                <DialogContent className="max-w-4xl">
                    <DialogHeader>
                        <DialogTitle>Add Operators</DialogTitle>
                        <DialogDescription>Select operators to add to {tiers.find((t) => t.id === selectedTierForAdd)?.name ?? "tier"}</DialogDescription>
                    </DialogHeader>

                    <div className="space-y-4">
                        <div className="flex items-center gap-2">
                            <Input className="flex-1" onChange={(e) => setOperatorSearch(e.target.value)} placeholder="Search operators..." value={operatorSearch} />
                            <span className="whitespace-nowrap text-muted-foreground text-sm">
                                {filteredOperators.filter((op) => !placedOperators.has(op.id ?? "")).length} available / {placedOperators.size} placed
                            </span>
                        </div>

                        <ScrollArea className="h-[60vh] max-h-[600px] rounded-md border p-4">
                            {operatorsLoading ? (
                                <div className="flex h-full min-h-[200px] flex-col items-center justify-center gap-3 text-muted-foreground">
                                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                                    <p>Loading operators...</p>
                                </div>
                            ) : filteredOperators.length > 0 ? (
                                <div className="grid grid-cols-4 gap-3 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-8">
                                    {filteredOperators.map((operator) => {
                                        const rarityNum = rarityToNumber(operator.rarity);
                                        const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
                                        const placementInfo = placedOperators.get(operator.id ?? "");
                                        const isPlaced = !!placementInfo;

                                        return (
                                            <button
                                                className={cn("group relative aspect-square overflow-hidden rounded-md border bg-card transition-all", isPlaced ? "cursor-not-allowed opacity-40 grayscale" : "hover:ring-2 hover:ring-primary")}
                                                disabled={isPlaced}
                                                key={operator.id}
                                                onClick={() => {
                                                    if (selectedTierForAdd && !isPlaced) {
                                                        handleAddOperatorToTier(selectedTierForAdd, operator);
                                                    }
                                                }}
                                                type="button"
                                            >
                                                <Image alt={operator.name} className="h-full w-full object-cover" fill src={`/api/cdn${operator.portrait}`} />
                                                <div className="absolute bottom-0 h-1 w-full" style={{ backgroundColor: rarityColor }} />
                                                {isPlaced && (
                                                    <div className="absolute inset-0 flex flex-col items-center justify-center bg-background/70">
                                                        <Check className="h-5 w-5 text-green-500" />
                                                        <span className="mt-0.5 px-1 text-center font-medium text-[9px] text-muted-foreground">{placementInfo.tierName}</span>
                                                    </div>
                                                )}
                                                {!isPlaced && (
                                                    <div className="absolute inset-0 flex items-end bg-linear-to-t from-background/80 to-transparent p-1 opacity-0 transition-opacity group-hover:opacity-100">
                                                        <p className="w-full truncate text-center text-xs">{operator.name}</p>
                                                    </div>
                                                )}
                                            </button>
                                        );
                                    })}
                                </div>
                            ) : (
                                <div className="flex h-full min-h-[200px] items-center justify-center text-muted-foreground">No operators found matching your search</div>
                            )}
                        </ScrollArea>
                    </div>

                    <DialogFooter>
                        <Button onClick={() => setAddOperatorDialogOpen(false)} variant="outline">
                            Done
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
}
