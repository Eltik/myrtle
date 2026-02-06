"use client";

import { Users } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { cn } from "~/lib/utils";

interface CollectionOverviewCardProps {
    totalOwned: number;
    totalAvailable: number;
    collectionPercentage: number;
    className?: string;
}

export function CollectionOverviewCard({ totalOwned, totalAvailable, collectionPercentage, className }: CollectionOverviewCardProps) {
    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm", className)}>
            <CardHeader className="flex-none pb-2">
                <CardTitle className="flex items-center gap-2 font-medium text-muted-foreground text-sm">
                    <Users className="h-4 w-4" />
                    Operator Collection
                </CardTitle>
            </CardHeader>
            <CardContent className="flex flex-1 flex-col justify-between gap-4">
                <motion.div animate={{ scale: 1, opacity: 1 }} className="flex flex-col items-center gap-1" initial={{ scale: 0.9, opacity: 0 }} transition={{ type: "spring", stiffness: 150, damping: 20 }}>
                    <div className="flex items-baseline gap-1">
                        <AnimatedNumber className="font-bold text-4xl tracking-tight" springOptions={{ stiffness: 60, damping: 20 }} value={totalOwned} />
                        <span className="text-lg text-muted-foreground">/ {totalAvailable}</span>
                    </div>
                    <p className="text-center text-muted-foreground text-xs">operators collected</p>
                </motion.div>

                <motion.div animate={{ opacity: 1 }} className="space-y-2" initial={{ opacity: 0 }} transition={{ delay: 0.2 }}>
                    <div className="flex items-center justify-between text-xs">
                        <span className="text-muted-foreground">Completion</span>
                        <span className="text-emerald-400">
                            <AnimatedNumber className="font-medium text-emerald-400 tabular-nums" decimals={1} springOptions={{ stiffness: 100, damping: 20 }} value={collectionPercentage} />%
                        </span>
                    </div>
                    <div className="relative h-2.5 w-full overflow-hidden rounded-full bg-muted/50">
                        <motion.div animate={{ width: `${collectionPercentage}%` }} className="h-full rounded-full bg-emerald-500" initial={{ width: 0 }} transition={{ delay: 0.3, duration: 0.6, ease: "easeOut" }} />
                    </div>
                </motion.div>
            </CardContent>
        </Card>
    );
}
