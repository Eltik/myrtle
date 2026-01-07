"use client";

import { AlertTriangle, Shuffle, Target } from "lucide-react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import type { Challenge } from "./types";

interface ChallengeDisplayProps {
    challenge: Challenge;
    onRandomize: () => void;
}

export function ChallengeDisplay({ challenge, onRandomize }: ChallengeDisplayProps) {
    const Icon = challenge.type === "restriction" ? AlertTriangle : Target;
    const badgeVariant = challenge.type === "restriction" ? "destructive" : challenge.type === "modifier" ? "default" : "secondary";

    return (
        <div className="space-y-4 rounded-xl border border-border bg-card/30 p-4 backdrop-blur-sm sm:p-6">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/10">
                        <Icon className="h-4 w-4 text-primary" />
                    </div>
                    <h2 className="font-semibold text-foreground">Your Challenge</h2>
                </div>
                <Button className="gap-2 bg-transparent" onClick={onRandomize} size="sm" variant="outline">
                    <Shuffle className="h-3.5 w-3.5" />
                    Reroll
                </Button>
            </div>

            <div className="rounded-lg border border-border/50 bg-secondary/30 p-4">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                    <div className="space-y-2">
                        <div className="flex items-center gap-2">
                            <h3 className="font-bold text-foreground text-lg">{challenge.title}</h3>
                            <Badge className="capitalize" variant={badgeVariant}>
                                {challenge.type}
                            </Badge>
                        </div>
                        <p className="text-muted-foreground text-sm">{challenge.description}</p>
                    </div>
                </div>
            </div>
        </div>
    );
}
