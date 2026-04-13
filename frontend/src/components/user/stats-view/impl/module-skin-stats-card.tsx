"use client";

import { Layers, Palette } from "lucide-react";
import { motion } from "motion/react";
import { AnimatedNumber } from "~/components/ui/motion-primitives/animated-number";
import { Card, CardContent } from "~/components/ui/shadcn/card";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";

interface ModuleSkinStatsCardProps {
    modules: {
        unlocked: number;
        atMax: number;
        totalAvailable: number;
    };
    skins: {
        totalOwned: number;
        totalAvailable: number;
        percentage: number;
    };
    className?: string;
}

export function ModuleSkinStatsCard({ modules, skins, className }: ModuleSkinStatsCardProps) {
    const modulePercentage = modules.totalAvailable > 0 ? (modules.unlocked / modules.totalAvailable) * 100 : 0;

    return (
        <Card className={cn("flex h-full flex-col border-border/50 bg-linear-to-br from-card/80 to-card/40 backdrop-blur-sm", className)}>
            <CardContent className="flex flex-1 flex-col justify-center gap-5 pt-6">
                {/* Module Stats */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2 text-muted-foreground">
                        <Layers className="h-4 w-4" />
                        <span className="font-medium text-sm">Modules</span>
                    </div>

                    <div className="grid grid-cols-2 gap-3">
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.1, duration: 0.4 }}>
                                    <AnimatedNumber className="font-bold text-cyan-400 text-xl tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={modules.unlocked} />
                                    <span className="text-muted-foreground text-xs">Unlocked</span>
                                </motion.div>
                            </TooltipTrigger>
                            <TooltipContent sideOffset={5} variant="dark">
                                <p className="font-medium">Modules Unlocked</p>
                                <p className="text-muted-foreground">
                                    {modules.unlocked} of {modules.totalAvailable} available modules
                                </p>
                            </TooltipContent>
                        </Tooltip>

                        <Tooltip>
                            <TooltipTrigger asChild>
                                <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.2, duration: 0.4 }}>
                                    <AnimatedNumber className="font-bold text-cyan-400 text-xl tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={modules.atMax} />
                                    <span className="text-muted-foreground text-xs">Max Level</span>
                                </motion.div>
                            </TooltipTrigger>
                            <TooltipContent sideOffset={5} variant="dark">
                                <p className="font-medium">Max Level Modules</p>
                                <p className="text-muted-foreground">Modules upgraded to level 3</p>
                            </TooltipContent>
                        </Tooltip>
                    </div>

                    <motion.div animate={{ opacity: 1 }} initial={{ opacity: 0 }} transition={{ delay: 0.25 }}>
                        <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                            <motion.div animate={{ width: `${modulePercentage}%` }} className="h-full rounded-full bg-cyan-500" initial={{ width: 0 }} transition={{ delay: 0.3, duration: 0.6, ease: "easeOut" }} />
                        </div>
                        <p className="mt-1 text-center text-[0.625rem] text-muted-foreground/70">{modulePercentage.toFixed(1)}% unlocked</p>
                    </motion.div>
                </div>

                {/* Divider */}
                <div className="border-border/30 border-t" />

                {/* Skin Stats */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2 text-muted-foreground">
                        <Palette className="h-4 w-4" />
                        <span className="font-medium text-sm">Skins</span>
                    </div>

                    <Tooltip>
                        <TooltipTrigger asChild>
                            <motion.div animate={{ y: 0, opacity: 1 }} className="flex flex-col items-center gap-1 rounded-lg bg-muted/30 p-2.5" initial={{ y: 10, opacity: 0 }} transition={{ delay: 0.35, duration: 0.4 }}>
                                <div className="flex items-baseline gap-1">
                                    <AnimatedNumber className="font-bold text-pink-400 text-xl tabular-nums" springOptions={{ stiffness: 80, damping: 20 }} value={skins.totalOwned} />
                                    <span className="text-muted-foreground/60 text-sm">/ {skins.totalAvailable}</span>
                                </div>
                                <span className="text-muted-foreground text-xs">Skins Collected</span>
                            </motion.div>
                        </TooltipTrigger>
                        <TooltipContent sideOffset={5} variant="dark">
                            <p className="font-medium">Skins Collected</p>
                            <p className="text-muted-foreground">
                                {skins.totalOwned} of {skins.totalAvailable} non-default skins
                            </p>
                        </TooltipContent>
                    </Tooltip>

                    <motion.div animate={{ opacity: 1 }} initial={{ opacity: 0 }} transition={{ delay: 0.4 }}>
                        <div className="relative h-1.5 w-full overflow-hidden rounded-full bg-muted/50">
                            <motion.div animate={{ width: `${skins.percentage}%` }} className="h-full rounded-full bg-pink-500" initial={{ width: 0 }} transition={{ delay: 0.45, duration: 0.6, ease: "easeOut" }} />
                        </div>
                        <p className="mt-1 text-center text-[0.625rem] text-muted-foreground/70">{skins.percentage.toFixed(1)}% collected</p>
                    </motion.div>
                </div>
            </CardContent>
        </Card>
    );
}
