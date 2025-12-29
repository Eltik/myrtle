"use client";

import { DndContext, DragOverlay, KeyboardSensor, PointerSensor, closestCenter, useSensor, useSensors, type DragEndEvent, type DragStartEvent } from "@dnd-kit/core";
import { SortableContext, arrayMove, rectSortingStrategy, sortableKeyboardCoordinates, useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { ArrowLeft, Check, ChevronDown, ChevronUp, GripVertical, Palette, Plus, Save, Settings, Trash2, X } from "lucide-react";
import Image from "next/image";
import { useMemo, useState } from "react";
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
import { cn, rarityToNumber } from "~/lib/utils";

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
            <div
                className="absolute inset-0 z-10 cursor-grab active:cursor-grabbing"
                {...attributes}
                {...listeners}
            />

            <Image alt={operator.name} className="h-full w-full object-cover" fill src={`/api/cdn${operator.portrait}`} />

            {/* Vignette overlay for name visibility */}
            <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-black/70 to-transparent opacity-0 transition-opacity group-hover:opacity-100" />

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
    const [activeDragTierId, setActiveDragTierId] = useState<string | null>(null);

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

    // Get operators already in the tier list
    const placedOperatorIds = useMemo(() => {
        const ids = new Set<string>();
        for (const tier of tiers) {
            for (const placement of tier.placements) {
                ids.add(placement.operator_id);
            }
        }
        return ids;
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

    const handleMoveTierUp = (index: number) => {
        if (index === 0) return;
        const newTiers = [...tiers];
        const currentTier = newTiers[index];
        const prevTier = newTiers[index - 1];
        if (currentTier && prevTier) {
            newTiers[index - 1] = currentTier;
            newTiers[index] = prevTier;
        }
        // Update display orders
        newTiers.forEach((tier, i) => {
            tier.display_order = i;
            tier.isModified = true;
        });
        setTiers(newTiers);
    };

    const handleMoveTierDown = (index: number) => {
        if (index === tiers.length - 1) return;
        const newTiers = [...tiers];
        const currentTier = newTiers[index];
        const nextTier = newTiers[index + 1];
        if (currentTier && nextTier) {
            newTiers[index] = nextTier;
            newTiers[index + 1] = currentTier;
        }
        // Update display orders
        newTiers.forEach((tier, i) => {
            tier.display_order = i;
            tier.isModified = true;
        });
        setTiers(newTiers);
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

    const handleDragStart = (event: DragStartEvent, tierId: string) => {
        setActiveDragId(event.active.id as string);
        setActiveDragTierId(tierId);
    };

    const handleDragEnd = (event: DragEndEvent, tierId: string) => {
        const { active, over } = event;

        setActiveDragId(null);
        setActiveDragTierId(null);

        if (!over || active.id === over.id) return;

        setTiers(
            tiers.map((tier) => {
                if (tier.id !== tierId) return tier;

                const sortedPlacements = [...tier.placements].sort((a, b) => a.sub_order - b.sub_order);
                const oldIndex = sortedPlacements.findIndex((p) => p.id === active.id);
                const newIndex = sortedPlacements.findIndex((p) => p.id === over.id);

                if (oldIndex === -1 || newIndex === -1) return tier;

                const newPlacements = arrayMove(sortedPlacements, oldIndex, newIndex);

                return {
                    ...tier,
                    placements: newPlacements.map((p, i) => ({ ...p, sub_order: i })),
                    isModified: true,
                };
            }),
        );
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
                    placements: tier.placements.map((p, pIndex) => ({
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
            <div className="space-y-4">
                <div className="flex items-center justify-between">
                    <h3 className="font-semibold text-lg">Tiers ({tiers.length})</h3>
                    <Button onClick={handleAddTier} size="sm" variant="outline">
                        <Plus className="mr-2 h-4 w-4" />
                        Add Tier
                    </Button>
                </div>

                <div className="space-y-3">
                    {tiers.map((tier, index) => {
                        const tierColor = tier.color || DEFAULT_TIER_COLORS[tier.name] || "#888888";
                        const isExpanded = expandedTiers.has(tier.id);

                        return (
                            <Card className="overflow-hidden" key={tier.id}>
                                {/* Tier Header */}
                                <div className="flex items-center gap-2 border-b bg-muted/30 px-4 py-2">
                                    <div className="flex items-center gap-1">
                                        <Button className="h-7 w-7 cursor-grab" size="icon" variant="ghost">
                                            <GripVertical className="h-4 w-4 text-muted-foreground" />
                                        </Button>
                                        <Button className="h-7 w-7" disabled={index === 0} onClick={() => handleMoveTierUp(index)} size="icon" variant="ghost">
                                            <ChevronUp className="h-4 w-4" />
                                        </Button>
                                        <Button className="h-7 w-7" disabled={index === tiers.length - 1} onClick={() => handleMoveTierDown(index)} size="icon" variant="ghost">
                                            <ChevronDown className="h-4 w-4" />
                                        </Button>
                                    </div>

                                    <div className="h-8 w-12 rounded" style={{ backgroundColor: tierColor }} />

                                    <Input className="h-8 w-24 font-semibold" onChange={(e) => handleUpdateTier(tier.id, { name: e.target.value })} value={tier.name} />

                                    <Input className="hidden h-8 flex-1 sm:block" onChange={(e) => handleUpdateTier(tier.id, { description: e.target.value || null })} placeholder="Description (optional)" value={tier.description ?? ""} />

                                    {/* Color Picker */}
                                    <Popover>
                                        <PopoverTrigger asChild>
                                            <Button className="h-8 w-8" size="icon" variant="outline">
                                                <Palette className="h-4 w-4" />
                                            </Button>
                                        </PopoverTrigger>
                                        <PopoverContent className="w-auto p-3">
                                            <div className="grid grid-cols-6 gap-2">
                                                {PRESET_COLORS.map((color) => (
                                                    <button
                                                        className={cn("h-6 w-6 rounded border-2", tier.color === color ? "border-foreground" : "border-transparent")}
                                                        key={color}
                                                        onClick={() => handleUpdateTier(tier.id, { color })}
                                                        style={{ backgroundColor: color }}
                                                        type="button"
                                                    />
                                                ))}
                                            </div>
                                            <Separator className="my-2" />
                                            <Input className="h-8" onChange={(e) => handleUpdateTier(tier.id, { color: e.target.value })} placeholder="#000000" type="text" value={tier.color ?? ""} />
                                        </PopoverContent>
                                    </Popover>

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
                                        <CardContent className="p-4">
                                            {tier.placements.length > 0 ? (
                                                <DndContext
                                                    collisionDetection={closestCenter}
                                                    onDragEnd={(event) => handleDragEnd(event, tier.id)}
                                                    onDragStart={(event) => handleDragStart(event, tier.id)}
                                                    sensors={sensors}
                                                >
                                                    <SortableContext
                                                        items={tier.placements.sort((a, b) => a.sub_order - b.sub_order).map((p) => p.id)}
                                                        strategy={rectSortingStrategy}
                                                    >
                                                        <div className="grid grid-cols-5 gap-2 sm:grid-cols-8 md:grid-cols-10 lg:grid-cols-12">
                                                            {tier.placements
                                                                .sort((a, b) => a.sub_order - b.sub_order)
                                                                .map((placement) => {
                                                                    const operator = operatorsData[placement.operator_id];
                                                                    if (!operator) return null;

                                                                    return (
                                                                        <SortableOperatorCard
                                                                            key={placement.id}
                                                                            onRemove={() => handleRemoveOperatorFromTier(tier.id, placement.id)}
                                                                            operator={operator}
                                                                            placement={placement}
                                                                        />
                                                                    );
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
                                                    </SortableContext>

                                                    {/* Drag overlay for visual feedback */}
                                                    <DragOverlay>
                                                        {activeDragId && activeDragTierId === tier.id ? (
                                                            (() => {
                                                                const placement = tier.placements.find((p) => p.id === activeDragId);
                                                                const operator = placement ? operatorsData[placement.operator_id] : null;
                                                                return operator ? <DragOverlayCard operator={operator} /> : null;
                                                            })()
                                                        ) : null}
                                                    </DragOverlay>
                                                </DndContext>
                                            ) : (
                                                <div className="flex flex-col items-center justify-center gap-2 py-8">
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
                                        </CardContent>
                                    </DisclosureContent>
                                </Disclosure>
                            </Card>
                        );
                    })}
                </div>

                {tiers.length === 0 && (
                    <Card className="py-12">
                        <div className="flex flex-col items-center justify-center gap-2">
                            <p className="text-muted-foreground">No tiers configured</p>
                            <Button onClick={handleAddTier} variant="outline">
                                <Plus className="mr-2 h-4 w-4" />
                                Add First Tier
                            </Button>
                        </div>
                    </Card>
                )}
            </div>

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
                                {filteredOperators.length} / {allOperators.length} operators
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
                                        const isPlaced = placedOperatorIds.has(operator.id ?? "");

                                        return (
                                            <button
                                                className={cn(
                                                    "group relative aspect-square overflow-hidden rounded-md border bg-card transition-all hover:ring-2 hover:ring-primary",
                                                    isPlaced && "opacity-50 ring-1 ring-green-500/50",
                                                )}
                                                key={operator.id}
                                                onClick={() => {
                                                    if (selectedTierForAdd) {
                                                        handleAddOperatorToTier(selectedTierForAdd, operator);
                                                    }
                                                }}
                                                type="button"
                                            >
                                                <Image alt={operator.name} className="h-full w-full object-cover" fill src={`/api/cdn${operator.portrait}`} />
                                                <div className="absolute bottom-0 h-1 w-full" style={{ backgroundColor: rarityColor }} />
                                                {isPlaced && (
                                                    <div className="absolute top-0.5 right-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-green-500">
                                                        <Check className="h-3 w-3 text-white" />
                                                    </div>
                                                )}
                                                <div className="absolute inset-0 flex items-end bg-gradient-to-t from-background/80 to-transparent p-1 opacity-0 transition-opacity group-hover:opacity-100">
                                                    <p className="w-full truncate text-center text-xs">{operator.name}</p>
                                                </div>
                                            </button>
                                        );
                                    })}
                                </div>
                            ) : (
                                <div className="flex h-full min-h-[200px] items-center justify-center text-muted-foreground">
                                    No operators found matching your search
                                </div>
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
