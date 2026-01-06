"use client";

import { Search } from "lucide-react";
import { useTheme } from "next-themes";
import { useMemo, useState } from "react";
import { RARITY_COLORS, RARITY_COLORS_LIGHT } from "~/components/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import type { DpsOperatorListEntry } from "~/types/api/impl/dps-calculator";

interface OperatorSelectorProps {
    operators: DpsOperatorListEntry[];
    onSelectOperator: (operator: DpsOperatorListEntry) => void;
}

export function OperatorSelector({ operators, onSelectOperator }: OperatorSelectorProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const { resolvedTheme } = useTheme();

    const rarityColors = resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS;

    const filteredOperators = useMemo(() => {
        if (!searchQuery.trim()) return operators;

        const query = searchQuery.toLowerCase();
        return operators.filter((op) => op.name.toLowerCase().includes(query) || op.id.toLowerCase().includes(query) || op.profession.toLowerCase().includes(query));
    }, [operators, searchQuery]);

    return (
        <div className="space-y-4">
            {/* Search */}
            <div className="relative">
                <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                <Input className="pl-9" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search operators..." value={searchQuery} />
            </div>

            {/* Operator List */}
            <ScrollArea className="h-[300px] rounded-md border border-border">
                <div className="grid grid-cols-1 gap-2 p-3 sm:grid-cols-2 lg:grid-cols-3">
                    {filteredOperators.map((operator) => {
                        const rarityColor = rarityColors[operator.rarity] ?? "#ffffff";

                        return (
                            <Button className="h-auto justify-start gap-3 px-3 py-2 text-left" key={operator.id} onClick={() => onSelectOperator(operator)} variant="outline">
                                <div className="h-8 w-1 shrink-0 rounded-full" style={{ backgroundColor: rarityColor }} />
                                <div className="flex-1 overflow-hidden">
                                    <div className="truncate font-semibold text-sm">{operator.name}</div>
                                    <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                                        <span style={{ color: rarityColor }}>{"★".repeat(operator.rarity)}</span>
                                        <span>•</span>
                                        <span>{operator.profession}</span>
                                    </div>
                                </div>
                            </Button>
                        );
                    })}
                </div>

                {filteredOperators.length === 0 && <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm">No operators found</div>}
            </ScrollArea>
        </div>
    );
}
