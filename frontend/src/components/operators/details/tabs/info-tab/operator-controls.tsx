"use client";

import { ChevronDown } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Slider } from "~/components/ui/shadcn/slider";
import type { Operator } from "~/types/api";

interface OperatorControlsProps {
    operator: Operator;
    phaseIndex: number;
    level: number;
    favorPoint: number;
    showControls: boolean;
    onPhaseChange: (index: number) => void;
    onLevelChange: (level: number) => void;
    onFavorPointChange: (trust: number) => void;
    onToggleControls: () => void;
}

export function OperatorControls({ operator, phaseIndex, level, favorPoint, showControls, onLevelChange, onFavorPointChange, onToggleControls }: OperatorControlsProps) {
    const maxLevel = operator.phases[phaseIndex]?.MaxLevel ?? 1;

    const handleLevelChange = (value: number[]) => {
        onLevelChange(value[0] ?? 1);
    };

    const handleTrustChange = (value: number[]) => {
        onFavorPointChange(value[0] ?? 0);
    };

    return (
        <div className="rounded-lg border border-border bg-card/50 p-4">
            <div className="flex items-center justify-between">
                <h3 className="font-semibold text-lg">Operator Controls</h3>
                <Button className="gap-2 bg-transparent" onClick={onToggleControls} size="sm" variant="outline">
                    {showControls ? "Hide" : "Show"} Controls
                    <motion.div animate={{ rotate: showControls ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-4 w-4" />
                    </motion.div>
                </Button>
            </div>

            <AnimatePresence>
                {showControls && (
                    <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.3 }}>
                        <div className="mt-4 space-y-6">
                            <p className="text-muted-foreground text-sm">Adjust these controls to see how the operator&apos;s stats change at different levels and trust values.</p>

                            {/* Level Control */}
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <label className="font-medium text-sm">Level</label>
                                    <span className="text-muted-foreground text-xs">
                                        {level} / {maxLevel}
                                    </span>
                                </div>
                                <Slider max={maxLevel} min={1} onValueChange={handleLevelChange} step={1} value={[level]} />
                                <div className="flex items-center gap-2">
                                    <Input className="w-20" max={maxLevel} min={1} onChange={(e) => onLevelChange(Math.min(Math.max(1, Number.parseInt(e.target.value, 10) || 1), maxLevel))} type="number" value={level} />
                                    <Button onClick={() => onLevelChange(1)} size="sm" variant="outline">
                                        Min
                                    </Button>
                                    <Button onClick={() => onLevelChange(maxLevel)} size="sm" variant="outline">
                                        Max
                                    </Button>
                                </div>
                            </div>

                            {/* Trust Control */}
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <label className="font-medium text-sm">Trust</label>
                                    <span className="text-muted-foreground text-xs">{favorPoint}%</span>
                                </div>
                                <Slider max={200} min={0} onValueChange={handleTrustChange} step={1} value={[favorPoint]} />
                            </div>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
