"use client";

import { Check, Download, Search, Users } from "lucide-react";
import Image from "next/image";
import { useMemo, useState } from "react";
import { CLASS_DISPLAY, RARITY_COLORS } from "~/components/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
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
        <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                        <Users className="h-4 w-4 text-primary" />
                    </div>
                    <h2 className="font-semibold text-foreground">Roster</h2>
                    <span className="rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground text-xs">
                        {selectedCount} / {totalCount}
                    </span>
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
                <Button className="w-full gap-2 bg-transparent" onClick={onImportProfile} size="sm" variant="outline">
                    <Download className="h-4 w-4" />
                    Import from Profile
                </Button>
            )}

            <div className="relative">
                <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <input
                    className="h-10 w-full rounded-lg border border-border bg-secondary/50 pr-4 pl-10 text-foreground text-sm transition-colors placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                    onChange={(e) => setSearchQuery(e.target.value)}
                    placeholder="Search roster..."
                    type="text"
                    value={searchQuery}
                />
            </div>

            <div className="max-h-96 space-y-1 overflow-y-auto rounded-lg border border-border/50 bg-secondary/30 p-2">
                {filteredOperators.map((operator) => {
                    const operatorId = operator.id;
                    if (!operatorId) return null;
                    const isSelected = roster.has(operatorId);
                    const rarityNum = getRarityNumber(operator.rarity);
                    const rarityColor = RARITY_COLORS[rarityNum] ?? RARITY_COLORS[1];
                    const className = CLASS_DISPLAY[operator.profession] ?? operator.profession;

                    return (
                        <button className="flex w-full items-center gap-3 rounded-md p-2 transition-colors hover:bg-accent/50" key={operatorId} onClick={() => handleToggle(operatorId)} type="button">
                            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border">
                                <Image alt={operator.name} className="h-full w-full object-cover" height={48} src={`/api/cdn${operator.portrait || "/placeholder.svg"}`} width={48} />
                            </div>
                            <div className="min-w-0 flex-1 text-left">
                                <div className="truncate font-medium text-foreground text-sm">{operator.name}</div>
                                <div className="flex items-center gap-2 text-muted-foreground text-xs">
                                    <span>{className}</span>
                                    <span style={{ color: rarityColor }}>{"★".repeat(rarityNum)}</span>
                                </div>
                            </div>
                            <div className={`flex h-5 w-5 shrink-0 items-center justify-center rounded transition-colors ${isSelected ? "bg-primary text-primary-foreground" : "border border-border bg-secondary"}`}>{isSelected && <Check className="h-3.5 w-3.5" />}</div>
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
