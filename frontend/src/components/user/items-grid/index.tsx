"use client";

import { Package, Search, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import type { InventoryItem, User } from "~/types/api/impl/user";
import { ItemDetailCard } from "./impl/item-detail-card";
import { ItemIcon } from "./impl/item-icon";
import { SortIcon } from "./impl/sort-icon";
import type { ItemWithData } from "./impl/types";

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
                                <SortIcon field="name" sortBy={sortBy} sortOrder={sortOrder} />
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
