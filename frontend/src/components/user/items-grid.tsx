"use client";

import { ArrowDown, ArrowUp, ArrowUpDown, Box, ChevronDown, ChevronUp, Hammer, Layers, MapPin, Package, Search, Sparkles, Tag, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import type { InventoryItem, User } from "~/types/api/impl/user";

// Muted rarity color mappings - more monochrome with subtle tints
const RARITY_COLORS: Record<string, { bg: string; text: string; border: string }> = {
    TIER_1: { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" },
    TIER_2: { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" },
    TIER_3: { bg: "bg-neutral-100/80 dark:bg-neutral-800/80", text: "text-neutral-700 dark:text-neutral-200", border: "border-neutral-300 dark:border-neutral-600" },
    TIER_4: { bg: "bg-neutral-200/60 dark:bg-neutral-700/60", text: "text-neutral-700 dark:text-neutral-200", border: "border-neutral-300 dark:border-neutral-600" },
    TIER_5: { bg: "bg-neutral-200/80 dark:bg-neutral-700/80", text: "text-neutral-800 dark:text-neutral-100", border: "border-neutral-400 dark:border-neutral-500" },
    TIER_6: { bg: "bg-neutral-300/60 dark:bg-neutral-600/60", text: "text-neutral-900 dark:text-neutral-50", border: "border-neutral-400 dark:border-neutral-500" },
};

const RARITY_LABELS: Record<string, string> = {
    TIER_1: "Tier 1",
    TIER_2: "Tier 2",
    TIER_3: "Tier 3",
    TIER_4: "Tier 4",
    TIER_5: "Tier 5",
    TIER_6: "Tier 6",
};

const OCC_PER_LABELS: Record<string, string> = {
    USUAL: "Usual",
    ALMOST: "Almost",
    ALWAYS: "Always",
    SOMETIMES: "Sometimes",
    OFTEN: "Often"
};

// Format item class type for display
function formatClassType(classType: string | undefined): string {
    if (!classType) return "Unknown";
    return classType.charAt(0).toUpperCase() + classType.slice(1).toLowerCase();
}

// Format item type for display
function formatItemType(itemType: string | undefined): string {
    if (!itemType) return "Unknown";
    return itemType
        .split("_")
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
        .join(" ");
}

// Extended item type with all data for dialog
interface ItemWithData extends InventoryItem {
    id: string;
    displayAmount: number;
}

function ItemIcon({ src, alt }: { src: string; alt: string }) {
    const [hasError, setHasError] = useState(false);

    const handleError = useCallback(() => {
        setHasError(true);
    }, []);

    if (hasError) {
        return (
            <div className="flex h-9 w-9 items-center justify-center text-muted-foreground/50">
                <Box className="h-5 w-5" />
            </div>
        );
    }

    return <Image alt={alt} className="h-9 w-9 object-contain" height={36} onError={handleError} src={src} unoptimized width={36} />;
}

// Large icon for dialog
function ItemIconLarge({ src, alt }: { src: string; alt: string }) {
    const [hasError, setHasError] = useState(false);

    const handleError = useCallback(() => {
        setHasError(true);
    }, []);

    if (hasError) {
        return (
            <div className="flex h-20 w-20 items-center justify-center text-muted-foreground/50">
                <Box className="h-10 w-10" />
            </div>
        );
    }

    return <Image alt={alt} className="h-20 w-20 object-contain" height={80} onError={handleError} src={src} unoptimized width={80} />;
}

// Item detail card shown in the morphing dialog
function ItemDetailCard({ item }: { item: ItemWithData }) {
    const [showDetails, setShowDetails] = useState(false);

    const defaultColors = { bg: "bg-neutral-100 dark:bg-neutral-800", text: "text-neutral-600 dark:text-neutral-300", border: "border-neutral-200 dark:border-neutral-700" };
    const rarityColors = RARITY_COLORS[item.rarity ?? "TIER_1"] ?? defaultColors;
    const rarityLabel = RARITY_LABELS[item.rarity ?? "TIER_1"] ?? "Unknown";
    const imageUrl = item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId ?? item.id}.png`;

    return (
        <div className="w-full max-w-lg">
            {/* Header with rarity-colored background */}
            <div className={`rounded-t-xl p-4 ${rarityColors.bg}`}>
                <div className="flex items-start gap-4">
                    {/* Item Icon */}
                    <div className="shrink-0 rounded-xl bg-background/80 p-2 shadow-sm">
                        <ItemIconLarge alt={item.name ?? item.id} src={imageUrl} />
                    </div>

                    {/* Item Info */}
                    <div className="min-w-0 flex-1">
                        <h2 className={`font-bold text-xl ${rarityColors.text}`}>{item.name ?? item.id}</h2>
                        <p className="mt-0.5 text-muted-foreground text-sm">{item.id}</p>
                        <div className="mt-2 flex items-center gap-2">
                            <Badge className="font-mono" variant="secondary">
                                <Layers className="mr-1 h-3 w-3" />
                                {item.displayAmount.toLocaleString()}
                            </Badge>
                            <Badge className={rarityColors.border} variant="outline">
                                {rarityLabel}
                            </Badge>
                        </div>
                    </div>
                </div>
            </div>

            {/* Content */}
            <div className="space-y-4 rounded-b-xl border border-t-0 bg-background p-4">
                {/* Description */}
                {item.description && (
                    <div>
                        <p className="text-muted-foreground text-sm leading-relaxed">{item.description}</p>
                    </div>
                )}

                {/* Type Badges */}
                <div className="flex flex-wrap gap-2">
                    {item.classifyType && (
                        <Badge variant="secondary">
                            <Tag className="mr-1 h-3 w-3" />
                            {formatClassType(item.classifyType)}
                        </Badge>
                    )}
                    {item.itemType && (
                        <Badge variant="secondary">
                            <Tag className="mr-1 h-3 w-3" />
                            {formatItemType(item.itemType)}
                        </Badge>
                    )}
                </div>

                <Separator />

                {/* Usage */}
                {item.usage && (
                    <div>
                        <h3 className="mb-2 font-semibold text-sm">Usage</h3>
                        <p className="text-muted-foreground text-sm leading-relaxed">{item.usage}</p>
                    </div>
                )}

                {/* Building Production */}
                {item.buildingProductList && item.buildingProductList.length > 0 && (
                    <div>
                        <h3 className="mb-2 font-semibold text-sm">Building Production</h3>
                        <div className="space-y-1.5">
                            {item.buildingProductList.map((product) => (
                                <div className="flex items-center gap-2 rounded-md bg-muted/50 px-3 py-1.5 text-sm" key={product.formulaId}>
                                    <Hammer className="h-4 w-4 text-muted-foreground" />
                                    <span className="font-medium">{product.roomType}</span>
                                    <span className="text-muted-foreground">•</span>
                                    <span className="text-muted-foreground">{product.formulaId}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Expandable Details Button */}
                {(item.obtainApproach || (item.stageDropList && item.stageDropList.length > 0)) && (
                    <>
                        <Button className="w-full" onClick={() => setShowDetails(!showDetails)} size="sm" variant="outline">
                            {showDetails ? (
                                <>
                                    <ChevronUp className="mr-2 h-4 w-4" />
                                    Hide Details
                                </>
                            ) : (
                                <>
                                    <ChevronDown className="mr-2 h-4 w-4" />
                                    Show Details
                                </>
                            )}
                        </Button>

                        {/* Expandable Content */}
                        <AnimatePresence>
                            {showDetails && (
                                <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }}>
                                    <div className="space-y-4 pt-2">
                                        {/* How to Obtain */}
                                        {item.obtainApproach && (
                                            <div>
                                                <h3 className="mb-2 font-semibold text-sm">How to Obtain</h3>
                                                <p className="text-muted-foreground text-sm">{item.obtainApproach}</p>
                                            </div>
                                        )}

                                        {/* Drop Stages */}
                                        {item.stageDropList && item.stageDropList.length > 0 && (
                                            <div>
                                                <h3 className="mb-2 font-semibold text-sm">Drop Stages</h3>
                                                <ScrollArea className="max-h-32">
                                                    <div className="space-y-1.5">
                                                        {item.stageDropList.map((drop) => (
                                                            <div className="flex items-center justify-between rounded-md bg-muted/50 px-3 py-1.5 text-sm" key={drop.stageId}>
                                                                <span className="flex items-center gap-2">
                                                                    <MapPin className="h-4 w-4 text-muted-foreground" />
                                                                    {drop.stageId}
                                                                </span>
                                                                <Badge className="text-xs" variant="outline">
                                                                    {OCC_PER_LABELS[drop.occPer] ?? drop.occPer}
                                                                </Badge>
                                                            </div>
                                                        ))}
                                                    </div>
                                                </ScrollArea>
                                            </div>
                                        )}
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </>
                )}
            </div>
        </div>
    );
}

interface ItemsGridProps {
    data: User;
}

export function ItemsGrid({ data }: ItemsGridProps) {
    const [searchTerm, setSearchTerm] = useState("");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [sortBy, setSortBy] = useState<"name" | "amount">("amount");
    const hasAnimated = useRef(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            hasAnimated.current = true;
        }, 600);
        return () => clearTimeout(timer);
    }, []);

    const items = useMemo((): ItemWithData[] => {
        const inventory = data.inventory as Record<string, InventoryItem>;

        return Object.entries(inventory)
            .map(([id, item]): ItemWithData => {
                const displayAmount = (item.amount as unknown as { amount: number }).amount;
                return {
                    ...item,
                    id,
                    displayAmount,
                    iconId: item.iconId ?? id,
                    name: item.name ?? id,
                };
            })
            .filter((item) => item.displayAmount > 0)
            .filter((item) => {
                if (!searchTerm) return true;
                const name = item.name ?? item.id;
                return name.toLowerCase().includes(searchTerm.toLowerCase()) || item.id.toLowerCase().includes(searchTerm.toLowerCase());
            })
            .sort((a, b) => {
                const aName = a.name ?? a.id;
                const bName = b.name ?? b.id;
                const comparison = sortBy === "name" ? aName.localeCompare(bName) : a.displayAmount - b.displayAmount;
                return sortOrder === "asc" ? comparison : -comparison;
            });
    }, [data.inventory, searchTerm, sortBy, sortOrder]);

    const toggleSort = (field: "name" | "amount") => {
        if (sortBy === field) {
            setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
        } else {
            setSortBy(field);
            setSortOrder("desc");
        }
    };

    const SortIcon = ({ field }: { field: "name" | "amount" }) => {
        const isActive = sortBy === field;
        const isAscending = sortOrder === "asc";

        return (
            <div className="relative h-3.5 w-3.5">
                <AnimatePresence mode="wait">
                    {!isActive ? (
                        <motion.div animate={{ opacity: 0.5, scale: 1 }} className="absolute inset-0" exit={{ opacity: 0, scale: 0.8 }} initial={{ opacity: 0, scale: 0.8 }} key="inactive" transition={{ duration: 0.15 }}>
                            <ArrowUpDown className="h-3.5 w-3.5" />
                        </motion.div>
                    ) : (
                        <motion.div animate={{ opacity: 1, rotate: 0 }} className="absolute inset-0" exit={{ opacity: 0, rotate: isAscending ? -180 : 180 }} initial={{ opacity: 0, rotate: isAscending ? 180 : -180 }} key={isAscending ? "asc" : "desc"} transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}>
                            {isAscending ? <ArrowUp className="h-3.5 w-3.5" /> : <ArrowDown className="h-3.5 w-3.5" />}
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        );
    };

    return (
        <div className="space-y-4">
            {/* Search and Stats Bar */}
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="relative w-full sm:max-w-xs">
                    <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                    <Input className="h-9 bg-background/50 pl-9 text-sm" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search by name or ID..." value={searchTerm} />
                </div>
                <div className="flex items-center gap-2 text-muted-foreground text-sm">
                    <Package className="h-4 w-4" />
                    <span>
                        {items.length.toLocaleString()} item{items.length !== 1 ? "s" : ""}
                    </span>
                </div>
            </div>

            {/* Items Table */}
            <Card className="overflow-hidden border-border/50 py-0">
                <ScrollArea className="h-[520px]">
                    <div className="min-w-full">
                        {/* Table Header */}
                        <div className="sticky top-0 z-10 grid grid-cols-[56px_1fr_100px] gap-4 border-border/50 border-b bg-card/95 px-4 py-3 backdrop-blur-sm">
                            <div className="p-2 font-medium text-muted-foreground text-xs uppercase tracking-wider">Icon</div>
                            <Button className="flex h-auto items-center justify-start gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("name")} variant="ghost">
                                Item
                                <SortIcon field="name" />
                            </Button>
                            <Button className="flex h-auto items-center justify-end gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("amount")} variant="ghost">
                                Qty
                                <SortIcon field="amount" />
                            </Button>
                        </div>

                        {/* Table Body */}
                        <div className="divide-y divide-border/30">
                            <AnimatePresence mode="sync">
                                {items.map((item, index) => (
                                    <motion.div
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, transition: { duration: 0.1 } }}
                                        initial={hasAnimated.current ? false : { opacity: 0, y: 12 }}
                                        key={item.id}
                                        transition={{
                                            duration: hasAnimated.current ? 0.15 : 0.25,
                                            delay: hasAnimated.current ? 0 : Math.min(index * 0.015, 0.3),
                                            ease: [0.25, 0.46, 0.45, 0.94],
                                        }}
                                    >
                                        <MorphingDialog transition={{ type: "spring", bounce: 0.05, duration: 0.25 }}>
                                            <MorphingDialogTrigger className="grid w-full cursor-pointer grid-cols-[56px_1fr_100px] items-center gap-4 rounded-none px-4 py-3 text-left transition-colors hover:bg-muted/30">
                                                {/* Icon Cell */}
                                                <div className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-lg bg-muted/50">
                                                    <ItemIcon alt={item.name ?? item.id} src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} />
                                                </div>

                                                {/* Name Cell */}
                                                <div className="min-w-0">
                                                    <p className="truncate font-medium leading-tight">{item.name ?? item.id}</p>
                                                    <p className="truncate text-muted-foreground/70 text-xs">{item.id}</p>
                                                </div>

                                                {/* Amount Cell */}
                                                <div className="text-right">
                                                    <span className="font-mono font-semibold text-sm tabular-nums">{item.displayAmount.toLocaleString()}</span>
                                                </div>
                                            </MorphingDialogTrigger>

                                            {/* Item Detail Dialog */}
                                            <MorphingDialogContainer>
                                                <MorphingDialogContent className="relative rounded-xl border bg-card shadow-lg">
                                                    <MorphingDialogClose
                                                        className="absolute top-3 right-3 z-10 rounded-full bg-background/80 p-1.5 backdrop-blur-sm transition-colors hover:bg-background"
                                                        variants={{
                                                            initial: { opacity: 0, scale: 0.8 },
                                                            animate: { opacity: 1, scale: 1 },
                                                            exit: { opacity: 0, scale: 0.8 },
                                                        }}
                                                    >
                                                        <X className="h-4 w-4" />
                                                    </MorphingDialogClose>
                                                    <ItemDetailCard item={item} />
                                                </MorphingDialogContent>
                                            </MorphingDialogContainer>
                                        </MorphingDialog>
                                    </motion.div>
                                ))}
                            </AnimatePresence>

                            {/* Empty State */}
                            {items.length === 0 && (
                                <div className="flex flex-col items-center justify-center gap-2 py-16 text-center">
                                    <Package className="h-10 w-10 text-muted-foreground/50" />
                                    <p className="text-muted-foreground text-sm">No items found</p>
                                    {searchTerm && <p className="text-muted-foreground/70 text-xs">Try adjusting your search</p>}
                                </div>
                            )}
                        </div>
                    </div>
                </ScrollArea>
            </Card>
        </div>
    );
}
