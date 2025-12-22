"use client";

import { AnimatePresence, motion } from "motion/react";
import { ArrowUpDown, Search } from "lucide-react";
import Image from "next/image";
import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/shadcn/table";
import type { InventoryItem, User } from "~/types/api/impl/user";

const MotionTableRow = motion.create(TableRow);

interface ItemsGridProps {
    data: User;
}

export function ItemsGrid({ data }: ItemsGridProps) {
    const [searchTerm, setSearchTerm] = useState("");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [sortBy, setSortBy] = useState<"name" | "amount">("amount");
    const hasAnimated = useRef(false);

    useEffect(() => {
        // Mark as animated after initial mount
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

    return (
        <div className="space-y-4">
            <div className="flex items-center gap-4">
                <div className="relative flex-1">
                    <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                    <Input className="pl-10" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search items..." value={searchTerm} />
                </div>
            </div>

            <Card>
                <ScrollArea className="h-[500px]">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="w-16">Icon</TableHead>
                                <TableHead>
                                    <Button className="flex items-center gap-1 p-0 hover:bg-transparent" onClick={() => toggleSort("name")} variant="ghost">
                                        Name
                                        <ArrowUpDown className="h-4 w-4" />
                                    </Button>
                                </TableHead>
                                <TableHead className="text-right">
                                    <Button className="flex items-center gap-1 p-0 hover:bg-transparent" onClick={() => toggleSort("amount")} variant="ghost">
                                        Amount
                                        <ArrowUpDown className="h-4 w-4" />
                                    </Button>
                                </TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            <AnimatePresence mode="sync">
                                {items.map((item, index) => (
                                    <MotionTableRow
                                        key={item.id}
                                        initial={hasAnimated.current ? false : { opacity: 0, y: 20 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, transition: { duration: 0.1 } }}
                                        transition={{
                                            duration: hasAnimated.current ? 0.15 : 0.3,
                                            delay: hasAnimated.current ? 0 : Math.min(index * 0.02, 0.4),
                                            ease: [0.25, 0.46, 0.45, 0.94],
                                        }}
                                        whileHover={{
                                            backgroundColor: "hsl(var(--muted) / 0.5)",
                                            transition: { duration: 0.15 },
                                        }}
                                        className="transition-colors"
                                    >
                                        <TableCell>
                                            <motion.div
                                                whileHover={{ scale: 1.15 }}
                                                transition={{ type: "spring", stiffness: 400, damping: 17 }}
                                            >
                                                <Image alt={item.name} className="h-8 w-8" height={32} src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} unoptimized width={32} />
                                            </motion.div>
                                        </TableCell>
                                        <TableCell className="font-medium">{item.name}</TableCell>
                                        <TableCell className="text-right">{item.amount}</TableCell>
                                    </MotionTableRow>
                                ))}
                            </AnimatePresence>
                            {items.length === 0 && (
                                <TableRow>
                                    <TableCell className="text-center text-muted-foreground" colSpan={3}>
                                        No items found.
                                    </TableCell>
                                </TableRow>
                            )}
                        </TableBody>
                    </Table>
                </ScrollArea>
            </Card>
        </div>
    );
}
