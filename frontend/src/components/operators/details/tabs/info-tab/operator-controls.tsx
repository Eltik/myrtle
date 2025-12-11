"use client";

import { ChevronDown } from "lucide-react";
import { motion, AnimatePresence } from "motion/react";
import type { Operator } from "~/types/api";
import { Button } from "~/components/ui/shadcn/button";
import { Slider } from "~/components/ui/shadcn/slider";
import { Input } from "~/components/ui/shadcn/input";

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
                <h3 className="text-lg font-semibold">Operator Controls</h3>
                <Button variant="outline" size="sm" onClick={onToggleControls} className="gap-2 bg-transparent">
                    {showControls ? "Hide" : "Show"} Controls
                    <motion.div animate={{ rotate: showControls ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-4 w-4" />
                    </motion.div>
                </Button>
            </div>

            <AnimatePresence>
                {showControls && (
                    <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.3 }} className="overflow-hidden">
                        <div className="mt-4 space-y-6">
                            <p className="text-sm text-muted-foreground">Adjust these controls to see how the operator&apos;s stats change at different levels and trust values.</p>

                            {/* Level Control */}
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <label className="text-sm font-medium">Level</label>
                                    <span className="text-xs text-muted-foreground">
                                        {level} / {maxLevel}
                                    </span>
                                </div>
                                <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={handleLevelChange} />
                                <div className="flex items-center gap-2">
                                    <Input type="number" min={1} max={maxLevel} value={level} onChange={(e) => onLevelChange(Math.min(Math.max(1, Number.parseInt(e.target.value) || 1), maxLevel))} className="w-20" />
                                    <Button variant="outline" size="sm" onClick={() => onLevelChange(1)}>
                                        Min
                                    </Button>
                                    <Button variant="outline" size="sm" onClick={() => onLevelChange(maxLevel)}>
                                        Max
                                    </Button>
                                </div>
                            </div>

                            {/* Trust Control */}
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <label className="text-sm font-medium">Trust</label>
                                    <span className="text-xs text-muted-foreground">{favorPoint}%</span>
                                </div>
                                <Slider min={0} max={200} step={1} value={[favorPoint]} onValueChange={handleTrustChange} />
                            </div>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
