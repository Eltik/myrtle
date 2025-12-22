"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import { ArrowUpDown, Search } from "lucide-react";
import { Input } from "~/components/ui/shadcn/input";
import { Button } from "~/components/ui/shadcn/button";
import { Card } from "~/components/ui/shadcn/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/shadcn/table";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import type { User } from "~/types/api/impl/user";

interface ItemData {
    id: string;
    iconId?: string;
    name?: string;
    amount: number;
    image?: string;
}

interface ItemsGridProps {
    data: User;
}

export function ItemsGrid({ data }: ItemsGridProps) {
    const [searchTerm, setSearchTerm] = useState("");
    const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
    const [sortBy, setSortBy] = useState<"name" | "amount">("amount");

    const items = useMemo(() => {
        const inventory = data.inventory as Record<string, ItemData>;

        return Object.entries(inventory)
            .map(([id, item]) => ({
                id,
                iconId: item.iconId ?? id,
                name: item.name ?? id,
                amount: item.amount ?? (typeof item === "number" ? item : 0),
                image: item.image,
            }))
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
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input placeholder="Search items..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="pl-10" />
                </div>
            </div>

            <Card>
                <ScrollArea className="h-[500px]">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="w-16">Icon</TableHead>
                                <TableHead>
                                    <Button variant="ghost" onClick={() => toggleSort("name")} className="flex items-center gap-1 p-0 hover:bg-transparent">
                                        Name
                                        <ArrowUpDown className="h-4 w-4" />
                                    </Button>
                                </TableHead>
                                <TableHead className="text-right">
                                    <Button variant="ghost" onClick={() => toggleSort("amount")} className="flex items-center gap-1 p-0 hover:bg-transparent">
                                        Amount
                                        <ArrowUpDown className="h-4 w-4" />
                                    </Button>
                                </TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {items.map((item) => (
                                <TableRow key={item.id}>
                                    <TableCell>
                                        <Image src={item.image ? `/api/cdn${item.image}` : `/api/cdn/upk/spritepack/ui_item_icons_h1_0/${item.iconId}.png`} width={32} height={32} alt={item.name} className="h-8 w-8" unoptimized />
                                    </TableCell>
                                    <TableCell className="font-medium">{item.name}</TableCell>
                                    <TableCell className="text-right">{item.amount.toLocaleString()}</TableCell>
                                </TableRow>
                            ))}
                            {items.length === 0 && (
                                <TableRow>
                                    <TableCell colSpan={3} className="text-center text-muted-foreground">
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
