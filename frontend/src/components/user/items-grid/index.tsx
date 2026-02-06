"use client";

import { Filter, Loader2, Package, Search, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/motion-primitives/morphing-popover";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { useUserItems } from "~/hooks/use-user-items";
import { RARITY_LABELS } from "./impl/constants";
import { formatClassType } from "./impl/helpers";
import { ItemDetailCard } from "./impl/item-detail-card";
import { ItemIcon } from "./impl/item-icon";
import { SortIcon } from "./impl/sort-icon";
import type { ItemWithData } from "./impl/types";

const RARITY_TIERS = ["TIER_1", "TIER_2", "TIER_3", "TIER_4", "TIER_5", "TIER_6"] as const;

function getRarityValue(rarity: string | undefined): number {
    if (!rarity) return 0;
    const match = rarity.match(/TIER_(\d)/);
    return match?.[1] ? Number.parseInt(match[1], 10) : 0;
}

interface ItemsGridProps {
    userId: string;
}

export function ItemsGrid({ userId }: ItemsGridProps) {
    const { inventory, isLoading, error } = useUserItems(userId);
    const [searchTerm, setSearchTerm] = useState("");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [sortBy, setSortBy] = useState<"name" | "amount" | "rarity" | "category">("amount");
    const [hiddenRarities, setHiddenRarities] = useState<Set<string>>(new Set());
    const [hiddenCategories, setHiddenCategories] = useState<Set<string>>(new Set());
    const hasAnimated = useRef(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            hasAnimated.current = true;
        }, 600);
        return () => clearTimeout(timer);
    }, []);

    const categories = useMemo((): string[] => {
        if (!inventory) return [];
        const cats = new Set<string>();
        for (const item of Object.values(inventory)) {
            if (item.classifyType) cats.add(item.classifyType);
        }
        return Array.from(cats).sort();
    }, [inventory]);

    const items = useMemo((): ItemWithData[] => {
        if (!inventory) return [];

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
                if (hiddenRarities.size > 0 && hiddenRarities.has(item.rarity ?? "TIER_1")) return false;
                if (hiddenCategories.size > 0 && hiddenCategories.has(item.classifyType ?? "")) return false;
                if (!searchTerm) return true;
                const name = item.name ?? item.id;
                return name.toLowerCase().includes(searchTerm.toLowerCase()) || item.id.toLowerCase().includes(searchTerm.toLowerCase());
            })
            .sort((a, b) => {
                const aName = a.name ?? a.id;
                const bName = b.name ?? b.id;
                let comparison: number;
                if (sortBy === "name") {
                    comparison = aName.localeCompare(bName);
                } else if (sortBy === "rarity") {
                    comparison = getRarityValue(a.rarity) - getRarityValue(b.rarity);
                } else if (sortBy === "category") {
                    comparison = (a.classifyType ?? "").localeCompare(b.classifyType ?? "");
                } else {
                    comparison = a.displayAmount - b.displayAmount;
                }
                return sortOrder === "asc" ? comparison : -comparison;
            });
    }, [inventory, searchTerm, sortBy, sortOrder, hiddenRarities, hiddenCategories]);

    const toggleSort = (field: "name" | "amount" | "rarity" | "category") => {
        if (sortBy === field) {
            setSortOrder((prev) => (prev === "asc" ? "desc" : "asc"));
        } else {
            setSortBy(field);
            setSortOrder("desc");
        }
    };

    const toggleRarityFilter = (rarity: string) => {
        setHiddenRarities((prev) => {
            const next = new Set(prev);
            if (next.has(rarity)) {
                next.delete(rarity);
            } else {
                next.add(rarity);
            }
            return next;
        });
    };

    const toggleCategoryFilter = (category: string) => {
        setHiddenCategories((prev) => {
            const next = new Set(prev);
            if (next.has(category)) {
                next.delete(category);
            } else {
                next.add(category);
            }
            return next;
        });
    };

    const totalHidden = hiddenRarities.size + hiddenCategories.size;

    if (isLoading) {
        return (
            <div className="flex min-h-100 items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        );
    }

    if (error) {
        return (
            <div className="flex min-h-100 flex-col items-center justify-center text-center">
                <p className="text-destructive">Failed to load items</p>
                <p className="mt-1 text-muted-foreground/70 text-sm">{error}</p>
            </div>
        );
    }

    return (
        <div className="space-y-4">
            {/* Search and Stats Bar */}
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex items-center gap-2">
                    <div className="relative w-full sm:max-w-xs">
                        <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input className="h-9 bg-background/50 pl-9 text-sm" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search by name or ID..." value={searchTerm} />
                    </div>
                    <MorphingPopover>
                        <MorphingPopoverTrigger>
                            <Button className="h-9 shrink-0" size="sm" variant={totalHidden > 0 ? "secondary" : "outline"}>
                                <Filter className="mr-1.5 h-3.5 w-3.5" />
                                Filter
                                {totalHidden > 0 && (
                                    <Badge className="ml-1.5 h-5 min-w-5 px-1 text-xs" variant="secondary">
                                        {totalHidden}
                                    </Badge>
                                )}
                            </Button>
                        </MorphingPopoverTrigger>
                        <MorphingPopoverContent className="w-72 sm:w-96">
                            <ScrollArea className="max-h-[min(400px,60vh)]">
                                <div className="space-y-3 p-3">
                                    <div className="space-y-1.5">
                                        <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Rarity</span>
                                        <div className="flex flex-wrap gap-1.5">
                                            {RARITY_TIERS.map((tier) => (
                                                <button
                                                    className={`rounded-md border px-2.5 py-1 text-xs transition-colors ${hiddenRarities.has(tier) ? "border-border/50 bg-muted/30 text-muted-foreground/50 line-through" : "border-border bg-background text-foreground hover:bg-muted/50"}`}
                                                    key={tier}
                                                    onClick={() => toggleRarityFilter(tier)}
                                                    type="button"
                                                >
                                                    {RARITY_LABELS[tier] ?? tier}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                    {categories.length > 0 && (
                                        <div className="space-y-1.5">
                                            <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Category</span>
                                            <div className="flex flex-wrap gap-1.5">
                                                {categories.map((cat) => (
                                                    <button
                                                        className={`rounded-md border px-2.5 py-1 text-xs transition-colors ${hiddenCategories.has(cat) ? "border-border/50 bg-muted/30 text-muted-foreground/50 line-through" : "border-border bg-background text-foreground hover:bg-muted/50"}`}
                                                        key={cat}
                                                        onClick={() => toggleCategoryFilter(cat)}
                                                        type="button"
                                                    >
                                                        {formatClassType(cat)}
                                                    </button>
                                                ))}
                                            </div>
                                        </div>
                                    )}
                                    {totalHidden > 0 && (
                                        <button
                                            className="text-muted-foreground text-xs underline hover:text-foreground"
                                            onClick={() => {
                                                setHiddenRarities(new Set());
                                                setHiddenCategories(new Set());
                                            }}
                                            type="button"
                                        >
                                            Reset all filters
                                        </button>
                                    )}
                                </div>
                            </ScrollArea>
                        </MorphingPopoverContent>
                    </MorphingPopover>
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
                <ScrollArea className="h-130">
                    <div className="min-w-full">
                        {/* Table Header */}
                        <div className="sticky top-0 z-10 grid grid-cols-[48px_1fr_80px] gap-3 border-border/50 border-b bg-card/95 px-4 py-3 backdrop-blur-sm sm:grid-cols-[56px_1fr_100px_80px_80px] sm:gap-4">
                            <div className="p-2 font-medium text-muted-foreground text-xs uppercase tracking-wider">Icon</div>
                            <Button className="flex h-auto items-center justify-start gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("name")} variant="ghost">
                                Item
                                <SortIcon field="name" sortBy={sortBy} sortOrder={sortOrder} />
                            </Button>
                            <Button className="hidden h-auto items-center justify-end gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground sm:flex" onClick={() => toggleSort("category")} variant="ghost">
                                Category
                                <SortIcon field="category" sortBy={sortBy} sortOrder={sortOrder} />
                            </Button>
                            <Button className="hidden h-auto items-center justify-end gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground sm:flex" onClick={() => toggleSort("rarity")} variant="ghost">
                                Rarity
                                <SortIcon field="rarity" sortBy={sortBy} sortOrder={sortOrder} />
                            </Button>
                            <Button className="flex h-auto items-center justify-end gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("amount")} variant="ghost">
                                Qty
                                <SortIcon field="amount" sortBy={sortBy} sortOrder={sortOrder} />
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
                                            <MorphingDialogTrigger className="grid w-full cursor-pointer grid-cols-[48px_1fr_80px] items-center gap-3 rounded-none px-4 py-3 text-left transition-colors hover:bg-muted/30 sm:grid-cols-[56px_1fr_100px_80px_80px] sm:gap-4">
                                                {/* Icon Cell */}
                                                <div className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-lg bg-muted/50">
                                                    <ItemIcon alt={item.name ?? item.id} src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} />
                                                </div>

                                                {/* Name Cell */}
                                                <div className="min-w-0">
                                                    <p className="truncate font-medium leading-tight">{item.name ?? item.id}</p>
                                                    <p className="truncate text-muted-foreground/70 text-xs">{item.id}</p>
                                                </div>

                                                {/* Category Cell */}
                                                <div className="hidden text-right sm:block">
                                                    <span className="text-muted-foreground text-xs">{formatClassType(item.classifyType)}</span>
                                                </div>

                                                {/* Rarity Cell */}
                                                <div className="hidden text-right sm:block">
                                                    <span className="text-muted-foreground text-xs">{RARITY_LABELS[item.rarity ?? ""] ?? "—"}</span>
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
