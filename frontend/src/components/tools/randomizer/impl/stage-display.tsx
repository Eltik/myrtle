"use client";

import { MapPin, Shuffle, Zap } from "lucide-react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import type { Stage } from "~/types/api/impl/stage";

interface StageDisplayProps {
    stage: Stage;
    getZoneName: (zoneId: string) => string;
    onRandomize: () => void;
}

export function StageDisplay({ stage, getZoneName, onRandomize }: StageDisplayProps) {
    const zoneName = getZoneName(stage.zoneId);

    return (
        <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                        <MapPin className="h-4 w-4 text-primary" />
                    </div>
                    <h2 className="font-semibold text-foreground">Your Stage</h2>
                </div>
                <Button className="gap-2 bg-transparent" onClick={onRandomize} size="sm" variant="outline">
                    <Shuffle className="h-3.5 w-3.5" />
                    Reroll
                </Button>
            </div>

            <div className="rounded-lg border border-border/50 bg-secondary/30 p-4">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                    <div className="space-y-2">
                        <div className="flex flex-wrap items-center gap-2">
                            <h3 className="font-bold text-foreground text-xl">{stage.code}</h3>
                            {stage.name && <span className="text-muted-foreground">— {stage.name}</span>}
                        </div>
                        <div className="flex flex-wrap items-center gap-2 text-muted-foreground text-sm">
                            <span>{zoneName}</span>
                            <span className="text-border">•</span>
                            <span className="capitalize">{stage.difficulty.toLowerCase().replace("_", " ")}</span>
                        </div>
                        {stage.description && <p className="text-muted-foreground text-sm">{stage.description}</p>}
                    </div>

                    <div className="flex flex-wrap gap-2">
                        {stage.bossMark && (
                            <Badge className="gap-1" variant="destructive">
                                <Zap className="h-3 w-3" />
                                Boss
                            </Badge>
                        )}
                        <Badge className="bg-background/50" variant="outline">
                            {stage.apCost} <span className="ml-1 text-muted-foreground text-xs">AP</span>
                        </Badge>
                        {stage.dangerLevel && (
                            <Badge className="bg-background/50" variant="outline">
                                Lvl {stage.dangerLevel}
                            </Badge>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
