"use client";

import { ChevronDown, Info } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import type { Operator } from "~/types/api";

interface OperatorControlsProps {
    operator: Operator;
    phaseIndex: number;
    level: number;
    favorPoint: number;
    potentialRank: number;
    currentModule: string;
    currentModuleLevel: number;
    showControls: boolean;
    onPhaseChange: (index: number) => void;
    onLevelChange: (level: number) => void;
    onFavorPointChange: (trust: number) => void;
    onPotentialChange: (rank: number) => void;
    onModuleChange: (moduleId: string) => void;
    onModuleLevelChange: (level: number) => void;
    onToggleControls: () => void;
}

export function OperatorControls({ operator, phaseIndex, level, favorPoint, potentialRank, currentModule, currentModuleLevel, showControls, onLevelChange, onFavorPointChange, onPotentialChange, onModuleChange, onModuleLevelChange, onToggleControls }: OperatorControlsProps) {
    const maxLevel = operator.phases[phaseIndex]?.MaxLevel ?? 1;
    const modules = operator.modules?.filter((m) => m.type !== "INITIAL") ?? [];
    const hasModules = phaseIndex === operator.phases.length - 1 && modules.length > 0;

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
                            <p className="text-muted-foreground text-sm">Adjust these controls to see how the operator&apos;s stats change at different levels, trust, potential, and modules.</p>

                            {/* Level Control */}
                            <div className="space-y-3">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-1">
                                        <label className="font-medium text-sm">Level</label>
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p>Adjust the operator&apos;s level to see stat changes</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    </div>
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

                            {/* Trust and Potential Row */}
                            <div className="grid gap-4 md:grid-cols-2">
                                {/* Trust Control */}
                                <div className="space-y-3">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-1">
                                            <label className="font-medium text-sm">Trust</label>
                                            <TooltipProvider>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Trust affects HP, ATK, and DEF bonuses (max 200%)</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        </div>
                                        <span className="text-muted-foreground text-xs">{favorPoint}%</span>
                                    </div>
                                    <Slider max={200} min={0} onValueChange={handleTrustChange} step={1} value={[favorPoint]} />
                                    <Input className="w-20" max={200} min={0} onChange={(e) => onFavorPointChange(Math.min(Math.max(0, Number.parseInt(e.target.value, 10) || 0), 200))} type="number" value={favorPoint} />
                                </div>

                                {/* Potential Control */}
                                <div className="space-y-3">
                                    <div className="flex items-center gap-1">
                                        <label className="font-medium text-sm">Potential</label>
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p>Potential rank provides stat bonuses</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    </div>
                                    <Select onValueChange={(value) => onPotentialChange(Number.parseInt(value, 10))} value={String(potentialRank)}>
                                        <SelectTrigger>
                                            <SelectValue placeholder="Select Potential" />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="0">No Potential</SelectItem>
                                            {operator.potentialRanks.map((rank, index) => (
                                                <SelectItem key={index} value={String(index + 1)}>
                                                    <div className="flex items-center gap-2">
                                                        <Image alt={`Potential ${index + 1}`} className="h-5 w-5" height={20} src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${index + 2}.png`} width={20} />
                                                        <span>
                                                            Pot {index + 1} - {rank.Description}
                                                        </span>
                                                    </div>
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>

                            {/* Module Selection (only at E2) */}
                            {hasModules && (
                                <div className="space-y-3 rounded-md border border-border bg-muted/30 p-3">
                                    <div className="flex items-center gap-1">
                                        <label className="font-medium text-sm">Module</label>
                                        <TooltipProvider>
                                            <Tooltip>
                                                <TooltipTrigger asChild>
                                                    <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p>Modules provide stat bonuses and talent upgrades at E2</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        </TooltipProvider>
                                    </div>
                                    <div className="grid gap-3 md:grid-cols-2">
                                        <Select onValueChange={onModuleChange} value={currentModule || "defaultModuleValue"}>
                                            <SelectTrigger>
                                                <SelectValue placeholder="Select Module" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="defaultModuleValue">No Module</SelectItem>
                                                {modules.map((module) => (
                                                    <SelectItem key={module.uniEquipId} value={module.uniEquipId ?? "defaultModuleValue"}>
                                                        {module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>

                                        {currentModule && (
                                            <Select onValueChange={(value) => onModuleLevelChange(Number.parseInt(value, 10))} value={String(currentModuleLevel)}>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Module Level" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {[1, 2, 3].map((lvl) => (
                                                        <SelectItem key={lvl} value={String(lvl)}>
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
                )}
            </AnimatePresence>
        </div>
    );
}
