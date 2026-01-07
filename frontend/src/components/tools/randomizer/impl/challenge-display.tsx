"use client";

import { AlertTriangle, Shuffle, Target } from "lucide-react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import type { Challenge } from "./types";

interface ChallengeDisplayProps {
    challenge: Challenge;
    onRandomize: () => void;
}

export function ChallengeDisplay({ challenge, onRandomize }: ChallengeDisplayProps) {
    const Icon = challenge.type === "restriction" ? AlertTriangle : Target;
    const badgeVariant = challenge.type === "restriction" ? "destructive" : challenge.type === "modifier" ? "default" : "secondary";

    return (
        <Card className="overflow-hidden border-border/50 bg-linear-to-br from-card/40 to-card/20 py-0 shadow-lg backdrop-blur-md">
            <CardContent className="p-4">
                <div className="space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-linear-to-br from-primary/20 to-primary/5 shadow-sm">
                                <Icon className="h-4 w-4 text-primary" />
                            </div>
                            <div>
                                <h2 className="font-semibold text-foreground text-sm">Your Challenge</h2>
                                <p className="text-[10px] text-muted-foreground">Additional modifier for extra difficulty</p>
                            </div>
                        </div>
                        <Button className="h-7 gap-1.5 bg-transparent px-2 text-xs shadow-sm transition-all hover:scale-105" onClick={onRandomize} variant="outline">
                            <Shuffle className="h-3 w-3" />
                            Reroll
                        </Button>
                    </div>

                    <div className="rounded-lg border border-border/30 bg-linear-to-br from-secondary/60 to-secondary/30 p-3 backdrop-blur-sm">
                        <div className="flex flex-col gap-1.5 sm:flex-row sm:items-center sm:justify-between">
                            <div className="space-y-1">
                                <div className="flex items-center gap-2">
                                    <h3 className="font-bold text-foreground text-sm">{challenge.title}</h3>
                                    <Badge className="h-5 px-1.5 text-[10px] capitalize shadow-sm" variant={badgeVariant}>
                                        {challenge.type}
                                    </Badge>
                                </div>
                                <p className="max-w-xl text-muted-foreground text-xs">{challenge.description}</p>
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
