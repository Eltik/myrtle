"use client";

import { Check, Download, Search, Users } from "lucide-react";
import Image from "next/image";
import { useMemo, useState } from "react";
import { CLASS_DISPLAY, RARITY_COLORS } from "~/components/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { RandomizerOperator } from "../index";
import { getRarityNumber } from "./utils";

interface RosterPanelProps {
    operators: RandomizerOperator[];
    roster: Set<string>;
    setRoster: (roster: Set<string>) => void;
    onImportProfile?: () => void;
    hasProfile?: boolean;
}

export function RosterPanel({ operators, roster, setRoster, onImportProfile, hasProfile }: RosterPanelProps) {
    const [searchQuery, setSearchQuery] = useState("");

    const filteredOperators = useMemo(() => {
        if (!searchQuery.trim()) return operators;
        const query = searchQuery.toLowerCase();
        return operators.filter((op) => op.name.toLowerCase().includes(query));
    }, [operators, searchQuery]);

    const selectedCount = roster.size;
    const totalCount = operators.length;

    const handleToggle = (operatorId: string) => {
        const newRoster = new Set(roster);
        if (newRoster.has(operatorId)) {
            newRoster.delete(operatorId);
        } else {
            newRoster.add(operatorId);
        }
        setRoster(newRoster);
    };

    const handleSelectAll = () => {
        setRoster(new Set(operators.map((op) => op.id).filter((id): id is string => id !== null)));
    };

    const handleDeselectAll = () => {
        setRoster(new Set());
    };

    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="space-y-4 p-6">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-md">
                            <Users className="h-4.5 w-4.5 text-primary" />
                        </div>
                        <div>
                            <h2 className="font-semibold text-foreground text-lg">Roster</h2>
                            <p className="text-muted-foreground text-xs">
                                {selectedCount} / {totalCount} operators
                            </p>
                        </div>
                    </div>
                    <div className="flex gap-2">
                        <Button onClick={handleSelectAll} size="sm" variant="ghost">
                            All
                        </Button>
                        <Button onClick={handleDeselectAll} size="sm" variant="ghost">
                            None
                        </Button>
                    </div>
                </div>

                {hasProfile && onImportProfile && (
                    <Button className="w-full gap-2 bg-transparent shadow-sm" onClick={onImportProfile} size="sm" variant="outline">
                        <Download className="h-4 w-4" />
                        Import from Profile
                    </Button>
                )}

                <div className="relative">
                    <Search className="absolute top-1/2 left-3.5 z-2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <input
                        className="h-11 w-full rounded-xl border border-border/40 bg-secondary/40 pr-4 pl-11 text-foreground text-sm shadow-sm backdrop-blur-sm transition-all placeholder:text-muted-foreground focus:border-primary/60 focus:outline-none focus:ring-2 focus:ring-primary/20"
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search operators..."
                        type="text"
                        value={searchQuery}
                    />
                </div>

                <div className="max-h-96 space-y-1.5 overflow-y-auto rounded-xl border border-border/30 bg-secondary/20 p-2 shadow-inner backdrop-blur-sm">
                    {filteredOperators.map((operator) => {
                        const operatorId = operator.id;
                        if (!operatorId) return null;
                        const isSelected = roster.has(operatorId);
                        const rarityNum = getRarityNumber(operator.rarity);
                        const rarityColor = RARITY_COLORS[rarityNum] ?? RARITY_COLORS[1];
                        const className = CLASS_DISPLAY[operator.profession] ?? operator.profession;

                        return (
                            <button className="flex w-full items-center gap-3 rounded-lg p-2.5 transition-all duration-200 hover:scale-[1.02] hover:bg-accent/60 hover:shadow-md" key={operatorId} onClick={() => handleToggle(operatorId)} type="button">
                                <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-lg border border-border/40 shadow-sm">
                                    <Image alt={operator.name} className="h-full w-full object-cover" height={48} src={`/api/cdn${operator.portrait || "/placeholder.svg"}`} width={48} />
                                </div>
                                <div className="min-w-0 flex-1 text-left">
                                    <div className="truncate font-semibold text-foreground text-sm">{operator.name}</div>
                                    <div className="flex items-center gap-2 text-muted-foreground text-xs">
                                        <span>{className}</span>
                                        <span className="font-bold" style={{ color: rarityColor }}>
                                            {"★".repeat(rarityNum)}
                                        </span>
                                    </div>
                                </div>
                                <div className={`flex h-6 w-6 shrink-0 items-center justify-center rounded-lg shadow-sm transition-all duration-200 ${isSelected ? "scale-110 bg-primary text-primary-foreground shadow-primary/30" : "border border-border/40 bg-secondary/50"}`}>
                                    {isSelected && <Check className="h-4 w-4" />}
                                </div>
                            </button>
                        );
                    })}
                </div>
            </CardContent>
        </Card>
    );
}
