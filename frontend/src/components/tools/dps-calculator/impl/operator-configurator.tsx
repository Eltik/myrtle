"use client";

import { ChevronDown, Heart, Settings2, Trash2 } from "lucide-react";
import Image from "next/image";
import { useTheme } from "next-themes";
import { useState } from "react";
import { RARITY_COLORS, RARITY_COLORS_LIGHT } from "~/components/operators/list/constants";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { Input } from "~/components/ui/shadcn/input";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Slider } from "~/components/ui/shadcn/slider";
import { Switch } from "~/components/ui/shadcn/switch";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { DpsBuffs, DpsConditionals, DpsShred } from "~/types/api/impl/dps-calculator";
import type { OperatorConfiguration } from "./types";

interface OperatorConfiguratorProps {
    operator: OperatorConfiguration;
    onUpdate: (id: string, updates: Partial<OperatorConfiguration>) => void;
    onRemove: (id: string) => void;
}

export function OperatorConfigurator({ operator, onUpdate, onRemove }: OperatorConfiguratorProps) {
    const [isOpen, setIsOpen] = useState(true);
    const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);
    const { resolvedTheme } = useTheme();

    const rarityColors = resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS;
    const rarityColor = rarityColors[operator.rarity] ?? "#ffffff";

    const updateParams = (key: keyof typeof operator.params, value: number) => {
        onUpdate(operator.id, {
            params: { ...operator.params, [key]: value },
        });
    };

    const updateAllCond = (value: boolean) => {
        onUpdate(operator.id, {
            params: { ...operator.params, allCond: value },
        });
    };

    const updateConditional = (key: keyof DpsConditionals, value: boolean) => {
        onUpdate(operator.id, {
            params: {
                ...operator.params,
                conditionals: {
                    ...operator.params.conditionals,
                    [key]: value,
                },
            },
        });
    };

    const updateBuff = (key: keyof DpsBuffs, value: number) => {
        onUpdate(operator.id, {
            params: {
                ...operator.params,
                buffs: {
                    ...operator.params.buffs,
                    [key]: value,
                },
            },
        });
    };

    const updateShred = (key: keyof DpsShred, value: number) => {
        onUpdate(operator.id, {
            params: {
                ...operator.params,
                shred: {
                    ...operator.params.shred,
                    [key]: value,
                },
            },
        });
    };

    // Helper to get max level for current promotion
    const getMaxLevel = () => {
        const promotion = operator.params.promotion ?? operator.maxPromotion;
        if (operator.phaseLevels?.[promotion]) {
            return operator.phaseLevels[promotion];
        }
        // Fallback based on rarity (standard max levels)
        const fallbackLevels: Record<number, number[]> = {
            6: [50, 80, 90],
            5: [50, 70, 80],
            4: [45, 60, 70],
            3: [40, 55],
            2: [30],
            1: [30],
        };
        return fallbackLevels[operator.rarity]?.[promotion] ?? 1;
    };

    return (
        <div className="overflow-hidden rounded-lg border border-border bg-card">
            <Disclosure onOpenChange={setIsOpen} open={isOpen}>
                <DisclosureTrigger>
                    <div className="flex w-full items-center justify-between p-3 text-left hover:bg-muted/30 data-[state=open]:bg-muted/30">
                        <div className="flex items-center gap-3">
                            <div className="h-8 w-1 shrink-0 rounded-full" style={{ backgroundColor: operator.color }} />
                            <div>
                                <div className="font-semibold text-sm">{operator.operatorName}</div>
                                <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                                    <span style={{ color: rarityColor }}>{"★".repeat(operator.rarity)}</span>
                                    {operator.availableSkills.length > 0 && (
                                        <>
                                            <span>•</span>
                                            <span>
                                                S{operator.params.skillIndex ?? operator.availableSkills[0] ?? 1}
                                                {operator.maxPromotion >= 2 && ((operator.params.masteryLevel ?? 3) === 0 ? " Lv7" : ` M${operator.params.masteryLevel ?? 3}`)}
                                            </span>
                                        </>
                                    )}
                                </div>
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <Button
                                className="h-8 w-8"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onRemove(operator.id);
                                }}
                                size="icon"
                                variant="ghost"
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                            <ChevronDown className="h-4 w-4 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180" />
                        </div>
                    </div>
                </DisclosureTrigger>

                <DisclosureContent>
                    <div className="border-border/50 border-t p-4">
                        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                            {/* Skill Selection */}
                            {operator.availableSkills.length > 0 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Skill</Label>
                                    <Select onValueChange={(value) => updateParams("skillIndex", Number(value))} value={String(operator.params.skillIndex ?? operator.availableSkills[0] ?? 1)}>
                                        <SelectTrigger>
                                            <SelectValue>
                                                {(() => {
                                                    const skillIdx = operator.params.skillIndex ?? operator.availableSkills[0] ?? 1;
                                                    const skillData = operator.skillData?.find((s) => s.index === skillIdx);
                                                    return <span className="truncate">{skillData?.name ?? `Skill ${skillIdx}`}</span>;
                                                })()}
                                            </SelectValue>
                                        </SelectTrigger>
                                        <SelectContent>
                                            {operator.availableSkills.map((skillIdx) => {
                                                const skillData = operator.skillData?.find((s) => s.index === skillIdx);
                                                return (
                                                    <SelectItem key={`skill-${skillIdx}`} value={String(skillIdx)}>
                                                        {skillData?.name ?? `Skill ${skillIdx}`}
                                                    </SelectItem>
                                                );
                                            })}
                                        </SelectContent>
                                    </Select>
                                </div>
                            )}

                            {/* Mastery Level - image button grid */}
                            {operator.availableSkills.length > 0 && operator.maxPromotion >= 2 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Mastery</Label>
                                    <div className="flex items-center gap-1.5">
                                        {/* Level 7 (no mastery) */}
                                        <button
                                            className={cn("flex h-8 w-8 items-center justify-center rounded-md border font-medium text-xs transition-all", (operator.params.masteryLevel ?? 3) === 0 ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                            onClick={() => updateParams("masteryLevel", 0)}
                                            type="button"
                                        >
                                            Lv7
                                        </button>
                                        {/* M1, M2, M3 */}
                                        {[1, 2, 3].map((level) => (
                                            <button
                                                className={cn("flex h-8 w-8 items-center justify-center rounded-md border transition-all", (operator.params.masteryLevel ?? 3) === level ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                                key={`mastery-${level}`}
                                                onClick={() => updateParams("masteryLevel", level)}
                                                type="button"
                                            >
                                                <Image alt={`M${level}`} height={20} src={`/api/cdn/upk/arts/specialized_hub/specialized_${level}.png`} unoptimized width={20} />
                                            </button>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Module - selection with level control */}
                            {operator.availableModules.length > 0 && (
                                <div className="space-y-2">
                                    <Label className="text-xs">Module</Label>
                                    <div className="flex items-center gap-2">
                                        <Select onValueChange={(value) => updateParams("moduleIndex", Number(value))} value={String(operator.params.moduleIndex ?? 0)}>
                                            <SelectTrigger className="flex-1">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="0">No Module</SelectItem>
                                                {operator.availableModules.map((moduleIdx) => {
                                                    const moduleData = operator.moduleData?.find((m) => m.index === moduleIdx);
                                                    const typeName = moduleData?.typeName1 ?? (moduleIdx === 1 ? "X" : moduleIdx === 2 ? "Y" : "D");
                                                    const moduleName = moduleData?.uniEquipName ?? `Module ${typeName}`;
                                                    return (
                                                        <SelectItem key={`module-${moduleIdx}`} value={String(moduleIdx)}>
                                                            {moduleName}
                                                        </SelectItem>
                                                    );
                                                })}
                                            </SelectContent>
                                        </Select>

                                        {/* Module Level Selector (only when a module is selected) */}
                                        {(operator.params.moduleIndex ?? 0) > 0 && (
                                            <Select onValueChange={(val) => updateParams("moduleLevel", Number(val))} value={String(operator.params.moduleLevel ?? 3)}>
                                                <SelectTrigger className="w-20">
                                                    <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    {[1, 2, 3].map((level) => (
                                                        <SelectItem key={`module-level-${level}`} value={String(level)}>
                                                            Lv.{level}
                                                        </SelectItem>
                                                    ))}
                                                </SelectContent>
                                            </Select>
                                        )}
                                    </div>
                                </div>
                            )}

                            {/* Potential - Desktop: Image button grid with tooltips */}
                            <div className="hidden space-y-2 md:block">
                                <Label className="text-xs">Potential</Label>
                                <TooltipProvider delayDuration={200}>
                                    <div className="flex flex-wrap items-center gap-1.5">
                                        {Array.from({ length: 6 }, (_, idx) => {
                                            // potentialRanks array has 5 items (indices 0-4) for potentials 2-6
                                            // Potential 1 (idx=0) has no bonus, so no description
                                            const potentialDescription = idx > 0 ? operator.potentialRanks?.[idx - 1]?.Description : undefined;
                                            return (
                                                // biome-ignore lint/suspicious/noArrayIndexKey: Static array of potentials
                                                <Tooltip key={`potential-${idx}`}>
                                                    <TooltipTrigger asChild>
                                                        <button
                                                            className={cn("flex h-8 w-8 items-center justify-center rounded-md border transition-all", (operator.params.potential ?? 1) === idx + 1 ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                                            onClick={() => updateParams("potential", idx + 1)}
                                                            type="button"
                                                        >
                                                            <Image alt={`Potential ${idx + 1}`} height={22} src={`/api/cdn/upk/arts/potential_hub/potential_${idx}.png`} unoptimized width={22} />
                                                        </button>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p className="font-medium">Potential {idx + 1}</p>
                                                        {potentialDescription ? <p className="text-muted-foreground text-xs">{potentialDescription}</p> : idx === 0 && <p className="text-muted-foreground text-xs">Base potential</p>}
                                                    </TooltipContent>
                                                </Tooltip>
                                            );
                                        })}
                                    </div>
                                </TooltipProvider>
                            </div>

                            {/* Potential - Mobile: Select with images and descriptions */}
                            <div className="space-y-2 md:hidden">
                                <Label className="text-xs">Potential</Label>
                                <Select onValueChange={(val) => updateParams("potential", Number(val))} value={String(operator.params.potential ?? 1)}>
                                    <SelectTrigger>
                                        <SelectValue>
                                            {(() => {
                                                const currentPot = operator.params.potential ?? 1;
                                                // potentialRanks[0] = Pot 2, potentialRanks[4] = Pot 6
                                                const description = currentPot > 1 ? operator.potentialRanks?.[currentPot - 2]?.Description : undefined;
                                                return (
                                                    <div className="flex items-center gap-2">
                                                        <Image alt={`Potential ${currentPot}`} className="h-5 w-5" height={20} src={`/api/cdn/upk/arts/potential_hub/potential_${currentPot - 1}.png`} unoptimized width={20} />
                                                        <span className="truncate">
                                                            Pot {currentPot}
                                                            {description && `: ${description}`}
                                                        </span>
                                                    </div>
                                                );
                                            })()}
                                        </SelectValue>
                                    </SelectTrigger>
                                    <SelectContent>
                                        {Array.from({ length: 6 }, (_, idx) => {
                                            // potentialRanks[0] = Pot 2, so for idx (0-5), description is at idx-1 for idx > 0
                                            const potentialDescription = idx > 0 ? operator.potentialRanks?.[idx - 1]?.Description : undefined;
                                            return (
                                                // biome-ignore lint/suspicious/noArrayIndexKey: Static array of potentials
                                                <SelectItem key={`potential-select-${idx}`} value={String(idx + 1)}>
                                                    <div className="flex items-center gap-2">
                                                        <Image alt={`Potential ${idx + 1}`} className="h-5 w-5 shrink-0" height={20} src={`/api/cdn/upk/arts/potential_hub/potential_${idx}.png`} unoptimized width={20} />
                                                        <span>
                                                            Pot {idx + 1}
                                                            {potentialDescription && `: ${potentialDescription}`}
                                                        </span>
                                                    </div>
                                                </SelectItem>
                                            );
                                        })}
                                    </SelectContent>
                                </Select>
                            </div>

                            {/* Trust - 0-200% with input */}
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-1">
                                        <Heart className="h-3.5 w-3.5 text-muted-foreground" />
                                        <Label className="text-xs">Trust</Label>
                                    </div>
                                    <div className="flex items-center gap-0.5 rounded-sm bg-accent px-2 py-0.5">
                                        <Input
                                            className="h-5 w-12 border-none bg-transparent p-0 text-center font-mono text-foreground text-sm shadow-none focus-visible:ring-0"
                                            max={200}
                                            min={0}
                                            onChange={(e) => {
                                                const val = Number.parseInt(e.target.value, 10);
                                                if (!Number.isNaN(val)) {
                                                    updateParams("trust", Math.max(0, Math.min(val, 200)));
                                                }
                                            }}
                                            type="number"
                                            value={operator.params.trust ?? 0}
                                        />
                                        <span className="font-mono text-foreground text-sm">%</span>
                                    </div>
                                </div>
                                <Slider
                                    className="w-full"
                                    max={200}
                                    min={0}
                                    onValueChange={(values) => {
                                        const raw = values[0] ?? 0;
                                        const rounded = Math.round(raw / 20) * 20;
                                        updateParams("trust", rounded);
                                    }}
                                    step={1}
                                    value={[operator.params.trust ?? 0]}
                                />
                            </div>

                            {/* Elite Level - image button grid */}
                            <div className="space-y-2">
                                <Label className="text-xs">Elite Level</Label>
                                <div className="flex items-center gap-2">
                                    {Array.from({ length: operator.maxPromotion + 1 }, (_, idx) => (
                                        <button
                                            className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", (operator.params.promotion ?? operator.maxPromotion) === idx ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                            // biome-ignore lint/suspicious/noArrayIndexKey: Static array of promotion phases
                                            key={`elite-${idx}`}
                                            onClick={() => updateParams("promotion", idx)}
                                            type="button"
                                        >
                                            <Image alt={`Elite ${idx}`} className="icon-theme-aware" height={24} src={`/api/cdn/upk/arts/elite_hub/elite_${idx}.png`} unoptimized width={24} />
                                        </button>
                                    ))}
                                </div>
                            </div>

                            {/* Level - slider + input */}
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <Label className="text-xs">Level</Label>
                                    <div className="flex items-center gap-1 rounded-sm bg-accent px-2 py-0.5">
                                        <Input
                                            className="h-5 w-8 border-none bg-transparent p-0 text-center font-mono text-foreground text-sm shadow-none focus-visible:ring-0"
                                            max={getMaxLevel()}
                                            min={1}
                                            onChange={(e) => {
                                                const val = Number.parseInt(e.target.value, 10);
                                                if (!Number.isNaN(val)) {
                                                    updateParams("level", Math.max(1, Math.min(val, getMaxLevel())));
                                                }
                                            }}
                                            type="number"
                                            value={operator.params.level ?? getMaxLevel()}
                                        />
                                        <span className="font-mono text-muted-foreground text-sm">/</span>
                                        <span className="font-mono text-foreground text-sm">{getMaxLevel()}</span>
                                    </div>
                                </div>
                                <Slider
                                    className="w-full"
                                    max={getMaxLevel()}
                                    min={1}
                                    onValueChange={(values) => {
                                        const raw = values[0] ?? 1;
                                        const rounded = Math.max(1, Math.round(raw / 10) * 10);
                                        updateParams("level", rounded);
                                    }}
                                    step={1}
                                    value={[operator.params.level ?? getMaxLevel()]}
                                />
                            </div>
                        </div>

                        {/* Advanced Options Section */}
                        <Disclosure onOpenChange={setIsAdvancedOpen} open={isAdvancedOpen}>
                            <DisclosureTrigger>
                                <div className="mt-4 flex w-full items-center gap-2 rounded-md border border-border/50 bg-muted/30 px-3 py-2 text-left text-muted-foreground text-sm hover:bg-muted/50">
                                    <Settings2 className="h-4 w-4" />
                                    <span className="font-medium">Advanced Options</span>
                                    <ChevronDown className="ml-auto h-4 w-4 transition-transform duration-200 group-data-[state=open]:rotate-180" />
                                </div>
                            </DisclosureTrigger>
                            <DisclosureContent>
                                <div className="mt-3 space-y-6">
                                    {/* Conditionals Section */}
                                    <div className="space-y-3">
                                        <Label className="text-muted-foreground text-xs uppercase tracking-wide">Conditionals</Label>
                                        <div className="grid grid-cols-2 gap-x-4 gap-y-2 md:grid-cols-3">
                                            {/* Master Toggle */}
                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="col-span-2 flex items-center justify-between rounded-md border border-primary/30 bg-primary/5 px-3 py-2 md:col-span-3">
                                                            <Label className="cursor-pointer font-medium text-sm" htmlFor="allCond">
                                                                All Conditionals
                                                            </Label>
                                                            <Switch checked={operator.params.allCond ?? true} id="allCond" onCheckedChange={updateAllCond} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Master toggle for all conditional bonuses</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            {/* Individual Conditionals */}
                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
                                                            <Label className="cursor-pointer text-xs" htmlFor="traitDamage">
                                                                Trait
                                                            </Label>
                                                            <Switch checked={operator.params.conditionals?.traitDamage ?? true} disabled={!(operator.params.allCond ?? true)} id="traitDamage" onCheckedChange={(v) => updateConditional("traitDamage", v)} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Trait passive damage bonuses</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
                                                            <Label className="cursor-pointer text-xs" htmlFor="talentDamage">
                                                                Talent 1
                                                            </Label>
                                                            <Switch checked={operator.params.conditionals?.talentDamage ?? true} disabled={!(operator.params.allCond ?? true)} id="talentDamage" onCheckedChange={(v) => updateConditional("talentDamage", v)} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Primary talent bonuses</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
                                                            <Label className="cursor-pointer text-xs" htmlFor="talent2Damage">
                                                                Talent 2
                                                            </Label>
                                                            <Switch checked={operator.params.conditionals?.talent2Damage ?? true} disabled={!(operator.params.allCond ?? true)} id="talent2Damage" onCheckedChange={(v) => updateConditional("talent2Damage", v)} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Secondary talent bonuses</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
                                                            <Label className="cursor-pointer text-xs" htmlFor="skillDamage">
                                                                Skill
                                                            </Label>
                                                            <Switch checked={operator.params.conditionals?.skillDamage ?? true} disabled={!(operator.params.allCond ?? true)} id="skillDamage" onCheckedChange={(v) => updateConditional("skillDamage", v)} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Skill passive effects</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>

                                            <TooltipProvider delayDuration={200}>
                                                <Tooltip>
                                                    <TooltipTrigger asChild>
                                                        <div className="flex items-center justify-between rounded-md border border-border px-3 py-2">
                                                            <Label className="cursor-pointer text-xs" htmlFor="moduleDamage">
                                                                Module
                                                            </Label>
                                                            <Switch checked={operator.params.conditionals?.moduleDamage ?? true} disabled={!(operator.params.allCond ?? true)} id="moduleDamage" onCheckedChange={(v) => updateConditional("moduleDamage", v)} />
                                                        </div>
                                                    </TooltipTrigger>
                                                    <TooltipContent>
                                                        <p>Module passive bonuses</p>
                                                    </TooltipContent>
                                                </Tooltip>
                                            </TooltipProvider>
                                        </div>
                                    </div>

                                    {/* External Buffs Section */}
                                    <div className="space-y-3">
                                        <Label className="text-muted-foreground text-xs uppercase tracking-wide">External Buffs</Label>
                                        <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="buffAtk">
                                                    ATK %
                                                </Label>
                                                <div className="flex items-center gap-1">
                                                    <Input
                                                        className="h-8 w-full font-mono text-sm"
                                                        id="buffAtk"
                                                        max={500}
                                                        min={0}
                                                        onChange={(e) => {
                                                            const val = Number.parseFloat(e.target.value);
                                                            if (!Number.isNaN(val)) {
                                                                updateBuff("atk", Math.max(0, Math.min(val / 100, 5)));
                                                            }
                                                        }}
                                                        placeholder="0"
                                                        type="number"
                                                        value={Math.round((operator.params.buffs?.atk ?? 0) * 100) || ""}
                                                    />
                                                    <span className="text-muted-foreground text-xs">%</span>
                                                </div>
                                            </div>

                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="buffFlatAtk">
                                                    Flat ATK
                                                </Label>
                                                <Input
                                                    className="h-8 w-full font-mono text-sm"
                                                    id="buffFlatAtk"
                                                    max={5000}
                                                    min={0}
                                                    onChange={(e) => {
                                                        const val = Number.parseInt(e.target.value, 10);
                                                        if (!Number.isNaN(val)) {
                                                            updateBuff("flatAtk", Math.max(0, Math.min(val, 5000)));
                                                        }
                                                    }}
                                                    placeholder="0"
                                                    type="number"
                                                    value={operator.params.buffs?.flatAtk || ""}
                                                />
                                            </div>

                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="buffAspd">
                                                    ASPD
                                                </Label>
                                                <Input
                                                    className="h-8 w-full font-mono text-sm"
                                                    id="buffAspd"
                                                    max={500}
                                                    min={0}
                                                    onChange={(e) => {
                                                        const val = Number.parseInt(e.target.value, 10);
                                                        if (!Number.isNaN(val)) {
                                                            updateBuff("aspd", Math.max(0, Math.min(val, 500)));
                                                        }
                                                    }}
                                                    placeholder="0"
                                                    type="number"
                                                    value={operator.params.buffs?.aspd || ""}
                                                />
                                            </div>

                                            <div className="space-y-1">
                                                <TooltipProvider delayDuration={200}>
                                                    <Tooltip>
                                                        <TooltipTrigger asChild>
                                                            <Label className="cursor-help text-xs" htmlFor="buffFragile">
                                                                Fragile %
                                                            </Label>
                                                        </TooltipTrigger>
                                                        <TooltipContent>
                                                            <p>Enemy damage taken increase (e.g., Suzuran S3)</p>
                                                        </TooltipContent>
                                                    </Tooltip>
                                                </TooltipProvider>
                                                <div className="flex items-center gap-1">
                                                    <Input
                                                        className="h-8 w-full font-mono text-sm"
                                                        id="buffFragile"
                                                        max={100}
                                                        min={0}
                                                        onChange={(e) => {
                                                            const val = Number.parseFloat(e.target.value);
                                                            if (!Number.isNaN(val)) {
                                                                updateBuff("fragile", Math.max(0, Math.min(val / 100, 1)));
                                                            }
                                                        }}
                                                        placeholder="0"
                                                        type="number"
                                                        value={Math.round((operator.params.buffs?.fragile ?? 0) * 100) || ""}
                                                    />
                                                    <span className="text-muted-foreground text-xs">%</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    {/* Enemy Shred Section */}
                                    <div className="space-y-3">
                                        <Label className="text-muted-foreground text-xs uppercase tracking-wide">Enemy Shred</Label>
                                        <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="shredDef">
                                                    DEF %
                                                </Label>
                                                <div className="flex items-center gap-1">
                                                    <Input
                                                        className="h-8 w-full font-mono text-sm"
                                                        id="shredDef"
                                                        max={100}
                                                        min={0}
                                                        onChange={(e) => {
                                                            const val = Number.parseInt(e.target.value, 10);
                                                            if (!Number.isNaN(val)) {
                                                                updateShred("def", Math.max(0, Math.min(val, 100)));
                                                            }
                                                        }}
                                                        placeholder="0"
                                                        type="number"
                                                        value={operator.params.shred?.def || ""}
                                                    />
                                                    <span className="text-muted-foreground text-xs">%</span>
                                                </div>
                                            </div>

                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="shredDefFlat">
                                                    DEF Flat
                                                </Label>
                                                <Input
                                                    className="h-8 w-full font-mono text-sm"
                                                    id="shredDefFlat"
                                                    max={2000}
                                                    min={0}
                                                    onChange={(e) => {
                                                        const val = Number.parseInt(e.target.value, 10);
                                                        if (!Number.isNaN(val)) {
                                                            updateShred("defFlat", Math.max(0, Math.min(val, 2000)));
                                                        }
                                                    }}
                                                    placeholder="0"
                                                    type="number"
                                                    value={operator.params.shred?.defFlat || ""}
                                                />
                                            </div>

                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="shredRes">
                                                    RES %
                                                </Label>
                                                <div className="flex items-center gap-1">
                                                    <Input
                                                        className="h-8 w-full font-mono text-sm"
                                                        id="shredRes"
                                                        max={100}
                                                        min={0}
                                                        onChange={(e) => {
                                                            const val = Number.parseInt(e.target.value, 10);
                                                            if (!Number.isNaN(val)) {
                                                                updateShred("res", Math.max(0, Math.min(val, 100)));
                                                            }
                                                        }}
                                                        placeholder="0"
                                                        type="number"
                                                        value={operator.params.shred?.res || ""}
                                                    />
                                                    <span className="text-muted-foreground text-xs">%</span>
                                                </div>
                                            </div>

                                            <div className="space-y-1">
                                                <Label className="text-xs" htmlFor="shredResFlat">
                                                    RES Flat
                                                </Label>
                                                <Input
                                                    className="h-8 w-full font-mono text-sm"
                                                    id="shredResFlat"
                                                    max={100}
                                                    min={0}
                                                    onChange={(e) => {
                                                        const val = Number.parseInt(e.target.value, 10);
                                                        if (!Number.isNaN(val)) {
                                                            updateShred("resFlat", Math.max(0, Math.min(val, 100)));
                                                        }
                                                    }}
                                                    placeholder="0"
                                                    type="number"
                                                    value={operator.params.shred?.resFlat || ""}
                                                />
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </DisclosureContent>
                        </Disclosure>
                    </div>
                </DisclosureContent>
            </Disclosure>
        </div>
    );
}
