"use client";

import { Shuffle, Users } from "lucide-react";
import Image from "next/image";
import { CLASS_DISPLAY, CLASS_ICON, RARITY_BLUR_COLORS, RARITY_COLORS } from "~/components/collection/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { RandomizerOperator } from "../index";
import { getRarityNumber } from "./utils";

interface SquadDisplayProps {
    operators: RandomizerOperator[];
    squadSize: number;
    onRandomize: () => void;
}

export function SquadDisplay({ operators, squadSize, onRandomize }: SquadDisplayProps) {
    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="p-4">
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-sm">
                                <Users className="h-4 w-4 text-primary" />
                            </div>
                            <div>
                                <h2 className="font-semibold text-foreground text-sm">Your Squad</h2>
                                <p className="text-[0.625rem] text-muted-foreground">
                                    {operators.length} / {squadSize} operators
                                </p>
                            </div>
                        </div>
                        <Button className="h-7 gap-1.5 bg-transparent px-2 text-xs shadow-sm transition-all hover:scale-105" onClick={onRandomize} variant="outline">
                            <Shuffle className="h-3 w-3" />
                            Reroll
                        </Button>
                    </div>

                    <div className="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-6 xl:grid-cols-8">
                        {operators.map((operator, index) => {
                            const rarityNum = getRarityNumber(operator.rarity);
                            const rarityColor = RARITY_COLORS[rarityNum] ?? RARITY_COLORS[1];
                            const rarityBlurColor = RARITY_BLUR_COLORS[rarityNum] ?? "#aaaaaa";
                            const iconName = CLASS_ICON[operator.profession] ?? operator.profession.toLowerCase();

                            return (
                                <div className="group relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card transition-all duration-200 hover:rounded-lg" key={`${operator.id}-${index}`}>
                                    {/* Portrait */}
                                    <div className="absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-105">
                                        <Image alt={operator.name} className="h-full w-full object-contain" height={120} src={`/api/cdn${operator.portrait || "/placeholder.svg"}`} width={80} />
                                    </div>

                                    {/* Bottom info bar */}
                                    <div className="absolute inset-x-0 bottom-0 z-10">
                                        <div className="relative">
                                            <div className="h-8 w-full bg-background/80 backdrop-blur-sm" />
                                            <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[85%] font-bold text-[0.625rem] uppercase opacity-60 transition-opacity group-hover:opacity-100">{operator.name}</h2>
                                            {/* Class icon */}
                                            <div className="absolute right-1 bottom-1 flex scale-75 items-center opacity-0 transition-all duration-200 group-hover:scale-100 group-hover:opacity-100">
                                                <div className="h-4 w-4">
                                                    <Image alt={CLASS_DISPLAY[operator.profession] ?? operator.profession} className="icon-theme-aware" height={16} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${iconName}.png`} width={16} />
                                                </div>
                                            </div>
                                            {/* Rarity color bar */}
                                            <div className="absolute bottom-0 h-0.5 w-full" style={{ backgroundColor: rarityColor }} />
                                            <div className="absolute -bottom-0.5 h-1 w-full blur-sm" style={{ backgroundColor: rarityBlurColor }} />
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
