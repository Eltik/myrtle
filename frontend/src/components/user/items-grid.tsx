"use client";

import { ArrowDown, ArrowUp, ArrowUpDown, Box, Package, Search } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import type { InventoryItem, User } from "~/types/api/impl/user";

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

    const items = useMemo(() => {
        const inventory = data.inventory as Record<string, InventoryItem>;

        return Object.entries(inventory)
            .map(([id, item]) => ({
                id,
                iconId: item.iconId ?? id,
                name: item.name ?? id,
                amount: (item.amount as unknown as { amount: number }).amount,
                image: item.image,
            }))
            .filter((item) => item.amount > 0)
            .filter((item) => {
                if (!searchTerm) return true;
                return item.name.toLowerCase().includes(searchTerm.toLowerCase()) || item.id.toLowerCase().includes(searchTerm.toLowerCase());
            })
            .sort((a, b) => {
                const comparison = sortBy === "name" ? a.name.localeCompare(b.name) : a.amount - b.amount;
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
                        <motion.div
                            key="inactive"
                            initial={{ opacity: 0, scale: 0.8 }}
                            animate={{ opacity: 0.5, scale: 1 }}
                            exit={{ opacity: 0, scale: 0.8 }}
                            transition={{ duration: 0.15 }}
                            className="absolute inset-0"
                        >
                            <ArrowUpDown className="h-3.5 w-3.5" />
                        </motion.div>
                    ) : (
                        <motion.div
                            key={isAscending ? "asc" : "desc"}
                            initial={{ opacity: 0, rotate: isAscending ? 180 : -180 }}
                            animate={{ opacity: 1, rotate: 0 }}
                            exit={{ opacity: 0, rotate: isAscending ? -180 : 180 }}
                            transition={{ duration: 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}
                            className="absolute inset-0"
                        >
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
                                        className="grid grid-cols-[56px_1fr_100px] items-center gap-4 px-4 py-3 transition-colors"
                                        exit={{ opacity: 0, transition: { duration: 0.1 } }}
                                        initial={hasAnimated.current ? false : { opacity: 0, y: 12 }}
                                        key={item.id}
                                        transition={{
                                            duration: hasAnimated.current ? 0.15 : 0.25,
                                            delay: hasAnimated.current ? 0 : Math.min(index * 0.015, 0.3),
                                            ease: [0.25, 0.46, 0.45, 0.94],
                                        }}
                                        whileHover={{
                                            backgroundColor: "hsl(var(--muted) / 0.3)",
                                        }}
                                    >
                                        {/* Icon Cell */}
                                        <motion.div className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-lg bg-muted/50" transition={{ type: "spring", stiffness: 400, damping: 17 }} whileHover={{ scale: 1.1 }}>
                                            <ItemIcon alt={item.name} src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} />
                                        </motion.div>

                                        {/* Name Cell */}
                                        <div className="min-w-0">
                                            <p className="truncate font-medium leading-tight">{item.name}</p>
                                            <p className="truncate text-muted-foreground/70 text-xs">{item.id}</p>
                                        </div>

                                        {/* Amount Cell */}
                                        <div className="text-right">
                                            <span className="font-mono font-semibold text-sm tabular-nums">{item.amount.toLocaleString()}</span>
                                        </div>
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
