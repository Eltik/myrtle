"use client";

import { MapPin, Shuffle, Zap } from "lucide-react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { Stage } from "~/types/api/impl/stage";

interface StageDisplayProps {
    stage: Stage;
    getZoneName: (zoneId: string) => string;
    onRandomize: () => void;
}

export function StageDisplay({ stage, getZoneName, onRandomize }: StageDisplayProps) {
    const zoneName = getZoneName(stage.zoneId);

    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="p-4">
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-sm">
                                <MapPin className="h-4 w-4 text-primary" />
                            </div>
                            <div>
                                <h2 className="font-semibold text-foreground text-sm">Your Stage</h2>
                                <p className="text-2.5 text-muted-foreground">Complete this stage with your squad</p>
                            </div>
                        </div>
                        <Button className="h-7 gap-1.5 bg-transparent px-2 text-xs shadow-sm transition-all hover:scale-105" onClick={onRandomize} variant="outline">
                            <Shuffle className="h-3 w-3" />
                            Reroll
                        </Button>
                    </div>

                    <div className="rounded-lg border border-border/30 bg-linear-to-br from-secondary/60 to-secondary/30 p-3 backdrop-blur-sm">
                        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                            <div className="space-y-1">
                                <div className="flex flex-wrap items-center gap-1.5">
                                    <h3 className="font-bold text-foreground text-lg">
                                        {stage.code} {stage.difficulty.toLowerCase().replace("_", " ").trim() === "four star" ? "CM" : ""}
                                    </h3>
                                    {stage.name && <span className="text-muted-foreground text-sm">— {stage.name}</span>}
                                </div>
                                <div className="flex flex-wrap items-center gap-1.5 text-muted-foreground text-xs">
                                    <span className="font-medium">{zoneName}</span>
                                    <span className="text-border">•</span>
                                    <span className="capitalize">{stage.difficulty.toLowerCase().replace("_", " ")}</span>
                                </div>
                            </div>

                            <div className="flex flex-wrap gap-1.5">
                                {stage.bossMark && (
                                    <Badge className="h-6 gap-1 px-2 text-2.5 shadow-sm" variant="destructive">
                                        <Zap className="h-3 w-3" />
                                        Boss
                                    </Badge>
                                )}
                                <Badge className="h-6 bg-background/60 px-2 font-semibold text-2.5 shadow-sm backdrop-blur-sm" variant="outline">
                                    {stage.apCost} AP
                                </Badge>
                                {stage.dangerLevel && (
                                    <Badge className="h-6 bg-background/60 px-2 font-semibold text-2.5 shadow-sm backdrop-blur-sm" variant="outline">
                                        Lvl {stage.dangerLevel}
                                    </Badge>
                                )}
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
