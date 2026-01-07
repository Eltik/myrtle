"use client";

import { Shuffle, Users } from "lucide-react";
import Image from "next/image";
import { CLASS_DISPLAY, RARITY_COLORS } from "~/components/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
import type { RandomizerOperator } from "../index";
import { getRarityNumber } from "./utils";

interface SquadDisplayProps {
    operators: RandomizerOperator[];
    squadSize: number;
    onRandomize: () => void;
}

export function SquadDisplay({ operators, squadSize, onRandomize }: SquadDisplayProps) {
    return (
        <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                        <Users className="h-4 w-4 text-primary" />
                    </div>
                    <h2 className="font-semibold text-foreground">Your Squad</h2>
                    <span className="rounded-full bg-muted px-2 py-0.5 font-medium text-muted-foreground text-xs">
                        {operators.length} / {squadSize}
                    </span>
                </div>
                <Button className="gap-2 bg-transparent" onClick={onRandomize} size="sm" variant="outline">
                    <Shuffle className="h-3.5 w-3.5" />
                    Reroll
                </Button>
            </div>

            <div className="grid grid-cols-3 gap-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8">
                {operators.map((operator, index) => {
                    const rarityNum = getRarityNumber(operator.rarity);
                    const rarityColor = RARITY_COLORS[rarityNum] ?? RARITY_COLORS[1];
                    const className = CLASS_DISPLAY[operator.profession] ?? operator.profession;

                    return (
                        <div className="group relative aspect-square overflow-hidden rounded-lg border border-border bg-secondary/50 transition-all hover:border-primary/50 hover:shadow-lg" key={`${operator.id}-${index}`}>
                            <Image alt={operator.name} className="h-full w-full object-cover transition-transform group-hover:scale-110" height={120} src={`/api/cdn${operator.portrait || "/placeholder.svg"}`} width={120} />
                            <div className="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/80 to-transparent p-2">
                                <div className="truncate font-medium text-white text-xs">{operator.name}</div>
                                <div className="flex items-center justify-between text-[10px]">
                                    <span className="text-white/80">{className}</span>
                                    <span style={{ color: rarityColor }}>{"★".repeat(rarityNum)}</span>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
