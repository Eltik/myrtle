"use client";

import type React from "react";
import { ChevronDown, Info } from "lucide-react";
import { motion } from "framer-motion";
import type { Operator } from "~/types/api";
import { Button } from "~/components/ui/shadcn/button";
import { Slider } from "~/components/ui/shadcn/slider";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";

interface OperatorControlsProps {
    operator: Operator;
    showControls: boolean;
    setShowControls: (show: boolean) => void;
    phaseIndex: number;
    level: number;
    setLevel: (level: number) => void;
    favorPoint: number;
    setFavorPoint: (favor: number) => void;
    potentialRank: number;
    setPotentialRank: (potential: number) => void;
    currentModule: string;
    setCurrentModule: (module: string) => void;
    currentModuleLevel: number;
    setCurrentModuleLevel: (level: number) => void;
}

export function OperatorControls({ operator, showControls, setShowControls, phaseIndex, level, setLevel, favorPoint, setFavorPoint, potentialRank, setPotentialRank, currentModule, setCurrentModule, currentModuleLevel, setCurrentModuleLevel }: OperatorControlsProps) {
    const maxLevel = operator.phases[phaseIndex]?.MaxLevel ?? 1;
    const hasModules = operator.modules && operator.modules.length > 0 && phaseIndex === 2;

    const handleLevelChange = (value: number[]) => {
        setLevel(value[0] ?? 1);
    };

    const handleLevelInput = (e: React.ChangeEvent<HTMLInputElement>) => {
        const newLevel = Number.parseInt(e.target.value, 10);
        if (!isNaN(newLevel)) {
            setLevel(Math.max(1, Math.min(newLevel, maxLevel)));
        }
    };

    const handleFavorChange = (value: number[]) => {
        setFavorPoint(value[0] ?? 0);
    };

    return (
        <div className="mb-6 rounded-lg border bg-muted/30 p-4">
            <div className="mb-3 flex items-center justify-between">
                <h3 className="font-medium">Operator Controls</h3>
                <Button variant="ghost" size="sm" onClick={() => setShowControls(!showControls)} className="gap-2">
                    {showControls ? "Hide" : "Show"} Controls
                    <motion.div animate={{ rotate: showControls ? 180 : 0 }} transition={{ duration: 0.2 }}>
                        <ChevronDown className="h-4 w-4" />
                    </motion.div>
                </Button>
            </div>

            <motion.div
                initial={false}
                animate={{
                    height: showControls ? "auto" : 0,
                    opacity: showControls ? 1 : 0,
                }}
                transition={{ duration: 0.3 }}
                className="overflow-hidden"
            >
                <p className="mb-4 text-sm text-muted-foreground">Adjust these controls to see how the operator&apos;s stats change at different levels, with different modules, and at various trust levels.</p>

                <div className="space-y-6">
                    {/* Level Control */}
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-medium">Level</span>
                                <TooltipProvider>
                                    <Tooltip>
                                        <TooltipTrigger>
                                            <Info className="h-3 w-3 text-muted-foreground" />
                                        </TooltipTrigger>
                                        <TooltipContent>
                                            <p>Adjust to see stat changes at different levels</p>
                                        </TooltipContent>
                                    </Tooltip>
                                </TooltipProvider>
                            </div>
                            <span className="text-xs text-muted-foreground">
                                {level} / {maxLevel}
                            </span>
                        </div>
                        <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={handleLevelChange} />
                        <div className="flex items-center gap-3">
                            <Input type="number" min={1} max={maxLevel} value={level} onChange={handleLevelInput} className="w-20" />
                            <div className="flex gap-2">
                                <Button variant="outline" size="sm" onClick={() => setLevel(1)}>
                                    Min
                                </Button>
                                <Button variant="outline" size="sm" onClick={() => setLevel(maxLevel)}>
                                    Max
                                </Button>
                            </div>
                        </div>
                    </div>

                    {/* Trust Control */}
                    <div className="space-y-3">
                        <div className="flex items-center justify-between">
                            <span className="text-sm font-medium">Trust</span>
                            <span className="text-xs text-muted-foreground">{favorPoint}%</span>
                        </div>
                        <Slider min={0} max={200} step={1} value={[favorPoint]} onValueChange={handleFavorChange} />
                    </div>

                    {/* Potential Control */}
                    <div className="space-y-2">
                        <span className="text-sm font-medium">Potential</span>
                        <Select value={potentialRank.toString()} onValueChange={(value) => setPotentialRank(Number.parseInt(value, 10))}>
                            <SelectTrigger className="w-full">
                                <SelectValue placeholder="Select potential" />
                            </SelectTrigger>
                            <SelectContent>
                                {[0, 1, 2, 3, 4, 5].map((pot) => (
                                    <SelectItem key={pot} value={pot.toString()}>
                                        Potential {pot + 1}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    {/* Module Control */}
                    {hasModules && (
                        <div className="space-y-3">
                            <span className="text-sm font-medium">Module</span>
                            <div className="flex gap-3">
                                <Select value={currentModule || "None"} onValueChange={setCurrentModule}>
                                    <SelectTrigger className="flex-1">
                                        <SelectValue placeholder="Select module" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="None">None</SelectItem>
                                        {operator.modules.map((mod) => (
                                            <SelectItem key={mod.uniEquipId} value={mod.uniEquipId ?? "None"}>
                                                {mod.uniEquipName ?? mod.uniEquipId}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                {currentModule && (
                                    <Select value={currentModuleLevel.toString()} onValueChange={(value) => setCurrentModuleLevel(Number.parseInt(value, 10))}>
                                        <SelectTrigger className="w-24">
                                            <SelectValue placeholder="Level" />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {[1, 2, 3].map((lvl) => (
                                                <SelectItem key={lvl} value={lvl.toString()}>
                                                    Stage {lvl}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                )}
                            </div>
                        </div>
                    )}
                </div>
            </motion.div>
        </div>
    );
}
