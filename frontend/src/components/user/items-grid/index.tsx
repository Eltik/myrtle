"use client";

import { Filter, Loader2, Package, Search, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { useUserItems } from "~/hooks/use-user-items";
import { RARITY_LABELS } from "./impl/constants";
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
    const [sortBy, setSortBy] = useState<"name" | "amount" | "rarity">("amount");
    const [hiddenRarities, setHiddenRarities] = useState<Set<string>>(new Set());
    const [showFilters, setShowFilters] = useState(false);
    const hasAnimated = useRef(false);

    useEffect(() => {
        const timer = setTimeout(() => {
            hasAnimated.current = true;
        }, 600);
        return () => clearTimeout(timer);
    }, []);

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
                } else {
                    comparison = a.displayAmount - b.displayAmount;
                }
                return sortOrder === "asc" ? comparison : -comparison;
            });
    }, [inventory, searchTerm, sortBy, sortOrder, hiddenRarities]);

    const toggleSort = (field: "name" | "amount" | "rarity") => {
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
                    <Button className="h-9 shrink-0" onClick={() => setShowFilters((prev) => !prev)} size="sm" variant={showFilters || hiddenRarities.size > 0 ? "secondary" : "outline"}>
                        <Filter className="mr-1.5 h-3.5 w-3.5" />
                        Filter
                        {hiddenRarities.size > 0 && (
                            <Badge className="ml-1.5 h-5 min-w-5 px-1 text-xs" variant="secondary">
                                {hiddenRarities.size}
                            </Badge>
                        )}
                    </Button>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground text-sm">
                    <Package className="h-4 w-4" />
                    <span>
                        {items.length.toLocaleString()} item{items.length !== 1 ? "s" : ""}
                    </span>
                </div>
            </div>

            {/* Rarity Filters */}
            <AnimatePresence>
                {showFilters && (
                    <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2 }}>
                        <div className="flex flex-wrap items-center gap-2 pt-1">
                            <span className="text-muted-foreground text-xs">Rarity:</span>
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
                            {hiddenRarities.size > 0 && (
                                <button className="text-muted-foreground text-xs underline hover:text-foreground" onClick={() => setHiddenRarities(new Set())} type="button">
                                    Reset
                                </button>
                            )}
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Items Table */}
            <Card className="overflow-hidden border-border/50 py-0">
                <ScrollArea className="h-130">
                    <div className="min-w-full">
                        {/* Table Header */}
                        <div className="sticky top-0 z-10 grid grid-cols-[56px_1fr_80px_80px] gap-4 border-border/50 border-b bg-card/95 px-4 py-3 backdrop-blur-sm">
                            <div className="p-2 font-medium text-muted-foreground text-xs uppercase tracking-wider">Icon</div>
                            <Button className="flex h-auto items-center justify-start gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("name")} variant="ghost">
                                Item
                                <SortIcon field="name" sortBy={sortBy} sortOrder={sortOrder} />
                            </Button>
                            <Button className="flex h-auto items-center justify-end gap-1.5 font-medium text-muted-foreground text-xs uppercase tracking-wider hover:bg-transparent hover:text-foreground" onClick={() => toggleSort("rarity")} variant="ghost">
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
                                            <MorphingDialogTrigger className="grid w-full cursor-pointer grid-cols-[56px_1fr_80px_80px] items-center gap-4 rounded-none px-4 py-3 text-left transition-colors hover:bg-muted/30">
                                                {/* Icon Cell */}
                                                <div className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-lg bg-muted/50">
                                                    <ItemIcon alt={item.name ?? item.id} src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} />
                                                </div>

                                                {/* Name Cell */}
                                                <div className="min-w-0">
                                                    <p className="truncate font-medium leading-tight">{item.name ?? item.id}</p>
                                                    <p className="truncate text-muted-foreground/70 text-xs">{item.id}</p>
                                                </div>

                                                {/* Rarity Cell */}
                                                <div className="text-right">
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
